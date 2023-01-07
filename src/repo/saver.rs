use crate::{
    config::{RenovateFormatConfig, RenovateOutputConfig},
    parser::SchemaId,
    DatabaseSchema, LocalRepo, MigrationPlanner, NodeDiff, NodeItem, SqlSaver,
};
use anyhow::Result;
use async_trait::async_trait;
use std::{collections::BTreeMap, fmt, hash::Hash, path::Path, str::FromStr};
use tokio::fs;

#[async_trait]
impl SqlSaver for DatabaseSchema {
    async fn save(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        use crate::config::Layout;

        // remove all existing sql files in the local repo
        let local_repo = LocalRepo::new(&config.path);
        for file in local_repo.files()? {
            fs::remove_file(file).await?;
        }

        match config.layout {
            Layout::Normal => self.normal(config).await,
            Layout::Flat => self.flat(config).await,
            Layout::Nested => self.nested(config).await,
        }
    }
}

impl DatabaseSchema {
    pub async fn flat(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        let content = self.to_string();
        let filename = config.path.join("all.sql");
        Self::write(filename, &content, config.format).await?;
        Ok(())
    }

    pub async fn nested(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        write_schema_files(&self.composite_types, "types", "01", vec![], config).await?;
        write_schema_files(&self.enum_types, "enums", "02", vec![], config).await?;

        write_schema_files(&self.sequences, "sequences", "03", vec![], config).await?;
        write_schema_files(
            &self.tables,
            "tables",
            "04",
            self.table_embedded_resources(),
            config,
        )
        .await?;

        write_schema_files(&self.views, "views", "08", vec![], config).await?;
        write_schema_files(&self.mviews, "mviews", "09", vec![], config).await?;
        write_schema_files(&self.functions, "functions", "10", vec![], config).await?;

        write_single_file(&self.triggers, "triggers", "11", config).await?;
        write_single_file(&self.privileges, "privileges", "12", config).await?;

        Ok(())
    }

    pub async fn normal(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        write_schema_file(&self.composite_types, "types", "01", vec![], config).await?;
        write_schema_file(&self.enum_types, "enums", "02", vec![], config).await?;

        write_schema_file(&self.sequences, "sequences", "03", vec![], config).await?;
        write_schema_file(
            &self.tables,
            "tables",
            "04",
            self.table_embedded_resources(),
            config,
        )
        .await?;

        write_schema_file(&self.views, "views", "08", vec![], config).await?;
        write_schema_file(&self.mviews, "mviews", "09", vec![], config).await?;
        write_schema_file(&self.functions, "functions", "10", vec![], config).await?;

        write_single_file(&self.triggers, "triggers", "11", config).await?;
        write_single_file(&self.privileges, "privileges", "12", config).await?;

        Ok(())
    }

    async fn write(
        filename: impl AsRef<Path>,
        content: &str,
        format: Option<RenovateFormatConfig>,
    ) -> anyhow::Result<()> {
        if let Some(format) = format {
            let content = sqlformat::format(content, &Default::default(), format.into());
            fs::write(filename, content).await?;
        } else {
            fs::write(filename, content).await?;
        };

        Ok(())
    }

    fn table_embedded_resources(&self) -> Vec<BTreeMap<SchemaId, BTreeMap<String, String>>> {
        vec![
            convert(&self.table_sequences),
            convert(&self.table_constraints),
            convert(&self.table_indexes),
        ]
    }
}

impl fmt::Display for DatabaseSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        // join_nested_items(&self.extensions, &mut result);
        join_nested_items(&self.composite_types, &mut result);
        join_nested_items(&self.enum_types, &mut result);
        join_nested_items(&self.sequences, &mut result);
        join_nested_items(&self.tables, &mut result);
        join_nested_items(&self.table_sequences, &mut result);
        join_nested_items(&self.table_constraints, &mut result);
        join_nested_items(&self.table_indexes, &mut result);
        join_nested_items(&self.views, &mut result);
        join_nested_items(&self.mviews, &mut result);
        join_nested_items(&self.functions, &mut result);

        result.push_str(&join_items(&self.triggers));
        result.push_str(&join_items(&self.privileges));

        write!(f, "{}", result)
    }
}

async fn write_schema_files<T>(
    source: &BTreeMap<String, BTreeMap<String, T>>,
    name: &str,
    prefix: &str,
    embedded_sources: Vec<BTreeMap<SchemaId, BTreeMap<String, String>>>,
    config: &RenovateOutputConfig,
) -> Result<()>
where
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    for (schema, items) in source {
        let path = config.path.join(schema);
        fs::create_dir_all(&path).await?;
        for (n, content) in items {
            let p = path.join(name);
            fs::create_dir_all(&p).await?;
            let filename = p.join(format!("{}_{}.sql", prefix, n));
            let item_content = format!("{};\n\n", content.to_string());
            let content = if embedded_sources.is_empty() {
                item_content
            } else {
                format!(
                    "{}{}",
                    item_content,
                    join_embedded_sources(SchemaId::new(schema, n), &embedded_sources)
                )
            };

            DatabaseSchema::write(&filename, &content, config.format).await?;
        }
    }
    Ok(())
}

fn join_embedded_sources(
    id: SchemaId,
    embedded_sources: &[BTreeMap<SchemaId, BTreeMap<String, String>>],
) -> String {
    let mut result = String::new();
    for source in embedded_sources {
        if let Some(items) = source.get(&id) {
            result.push_str(&join_items(items));
        }
    }
    result
}

async fn write_schema_file<T>(
    source: &BTreeMap<String, BTreeMap<String, T>>,
    name: &str,
    prefix: &str,
    embedded_sources: Vec<BTreeMap<SchemaId, BTreeMap<String, String>>>,
    config: &RenovateOutputConfig,
) -> Result<()>
where
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    for (schema, items) in source {
        let path = config.path.join(schema);
        fs::create_dir_all(&path).await?;
        let mut content = String::new();
        for (n, item) in items {
            let item_content = format!("{};\n\n", item.to_string());
            let s = if embedded_sources.is_empty() {
                item_content
            } else {
                format!(
                    "{}{}",
                    item_content,
                    join_embedded_sources(SchemaId::new(schema, n), &embedded_sources)
                )
            };

            content.push_str(&s);
        }

        let filename = path.join(format!("{}_{}.sql", prefix, name));
        DatabaseSchema::write(&filename, &content, config.format).await?;
    }

    Ok(())
}

async fn write_single_file<T>(
    source: &BTreeMap<String, T>,
    name: &str,
    prefix: &str,
    config: &RenovateOutputConfig,
) -> Result<()>
where
    T: NodeItem,
{
    let content = join_items(source);
    if !content.is_empty() {
        let path = config.path.join(format!("{}_{}.sql", prefix, name));
        DatabaseSchema::write(&path, &content, config.format).await?;
    }
    Ok(())
}

fn join_items<T>(source: &BTreeMap<String, T>) -> String
where
    T: ToString,
{
    let mut dest = String::new();
    for v in source.values() {
        dest.push_str(v.to_string().as_str());
        dest.push_str(";\n\n");
    }
    dest
}

fn join_nested_items<K, T>(source: &BTreeMap<K, BTreeMap<String, T>>, dest: &mut String)
where
    K: Hash + Eq + Ord,
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    for items in source.values() {
        dest.push_str(&join_items(items));
    }
}

fn convert<T>(
    source: &BTreeMap<SchemaId, BTreeMap<String, T>>,
) -> BTreeMap<SchemaId, BTreeMap<String, String>>
where
    T: NodeItem + Clone + PartialEq + Eq + 'static,
{
    source
        .clone()
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                v.into_iter()
                    .map(|(k1, v1)| (k1, v1.to_string()))
                    .collect::<BTreeMap<String, String>>(),
            )
        })
        .collect::<BTreeMap<SchemaId, _>>()
}
