use crate::{
    config::{RenovateFormatConfig, RenovateOutputConfig},
    DatabaseSchema, NodeItem, SqlSaver,
};
use anyhow::Result;
use async_trait::async_trait;
use std::{collections::BTreeMap, fmt, path::Path};
use tokio::fs;

/// Temporary struct to hold the data for the generated SQLs
struct SchemaSaver {
    pub types: BTreeMap<String, BTreeMap<String, String>>,
    pub sequences: BTreeMap<String, BTreeMap<String, String>>,
    pub tables: BTreeMap<String, BTreeMap<String, String>>,
    pub views: BTreeMap<String, BTreeMap<String, String>>,
    pub functions: BTreeMap<String, BTreeMap<String, String>>,
    pub triggers: BTreeMap<String, String>,
    pub privileges: BTreeMap<String, String>,
}

#[async_trait]
impl SqlSaver for DatabaseSchema {
    async fn save(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        use crate::config::Layout;
        let saver = SchemaSaver::try_from(self)?;

        match config.layout {
            Layout::Normal => saver.normal(config).await,
            Layout::Flat => saver.flat(config).await,
            Layout::Nested => saver.nested(config).await,
        }
    }
}

impl TryFrom<&DatabaseSchema> for SchemaSaver {
    type Error = anyhow::Error;
    fn try_from(schema: &DatabaseSchema) -> Result<Self, Self::Error> {
        let types = collect_schema_items(&schema.composite_types);
        let sequences = collect_schema_items(&schema.sequences);
        let tables = collect_schema_items(&schema.tables);
        let views = collect_schema_items(&schema.views);
        let functions = collect_schema_items(&schema.functions);

        let triggers = collect_db_items(&schema.triggers)?;
        let privileges = collect_db_items(&schema.privileges)?;

        Ok(Self {
            types,
            sequences,
            tables,
            views,
            functions,
            triggers,
            privileges,
        })
    }
}

impl SchemaSaver {
    pub async fn flat(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        let content = self.to_string();
        let filename = config.path.join("all.sql");
        SchemaSaver::write(filename, &content, config.format).await?;
        Ok(())
    }

    pub async fn nested(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        write_schema_files(&self.types, "types", config).await?;

        write_schema_files(&self.sequences, "sequences", config).await?;
        write_schema_files(&self.tables, "tables", config).await?;
        write_schema_files(&self.views, "views", config).await?;
        write_schema_files(&self.functions, "functions", config).await?;

        write_single_file(&self.triggers, "triggers", config).await?;
        write_single_file(&self.privileges, "privileges", config).await?;

        Ok(())
    }

    pub async fn normal(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        write_schema_file(&self.types, "types", config).await?;

        write_schema_file(&self.sequences, "sequences", config).await?;
        write_schema_file(&self.tables, "tables", config).await?;
        write_schema_file(&self.views, "views", config).await?;
        write_schema_file(&self.functions, "functions", config).await?;

        write_single_file(&self.triggers, "triggers", config).await?;
        write_single_file(&self.privileges, "privileges", config).await?;

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
}

impl fmt::Display for SchemaSaver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        join_nested_items(&self.types, &mut result);
        join_nested_items(&self.tables, &mut result);
        join_nested_items(&self.views, &mut result);
        join_nested_items(&self.functions, &mut result);

        result.push_str(&join_items(&self.triggers));
        result.push_str(&join_items(&self.privileges));

        write!(f, "{}", result)
    }
}

fn collect_schema_items<T>(
    source: &BTreeMap<String, BTreeMap<String, T>>,
) -> BTreeMap<String, BTreeMap<String, String>>
where
    T: NodeItem,
{
    let mut dest = BTreeMap::new();
    for (schema, items) in source {
        let result = items
            .iter()
            .map(|(name, ty)| (name.clone(), ty.to_string()))
            .collect::<BTreeMap<_, _>>();
        dest.insert(schema.clone(), result);
    }
    dest
}

fn collect_db_items<T>(source: &BTreeMap<String, T>) -> Result<BTreeMap<String, String>>
where
    T: NodeItem,
{
    let mut dest = BTreeMap::new();
    for (k, v) in source {
        dest.insert(k.clone(), v.node().deparse()?);
    }
    Ok(dest)
}

async fn write_schema_files(
    source: &BTreeMap<String, BTreeMap<String, String>>,
    name: &str,
    config: &RenovateOutputConfig,
) -> Result<()> {
    for (schema, items) in source {
        let path = config.path.join(schema);
        fs::create_dir_all(&path).await?;
        for (n, content) in items {
            let p = path.join(name);
            fs::create_dir_all(&p).await?;
            let filename = p.join(format!("{}.sql", n));
            let content = format!("{};\n", content);
            SchemaSaver::write(&filename, &content, config.format).await?;
        }
    }
    Ok(())
}

async fn write_schema_file(
    source: &BTreeMap<String, BTreeMap<String, String>>,
    name: &str,
    config: &RenovateOutputConfig,
) -> Result<()> {
    for (schema, items) in source {
        let path = config.path.join(schema);
        fs::create_dir_all(&path).await?;
        let content = join_items(items);
        let filename = path.join(format!("{}.sql", name));
        SchemaSaver::write(&filename, &content, config.format).await?;
    }

    Ok(())
}

async fn write_single_file(
    source: &BTreeMap<String, String>,
    name: &str,
    config: &RenovateOutputConfig,
) -> Result<()> {
    let content = join_items(source);
    if !content.is_empty() {
        let path = config.path.join(format!("{}.sql", name));
        SchemaSaver::write(&path, &content, config.format).await?;
    }
    Ok(())
}

fn join_items(source: &BTreeMap<String, String>) -> String {
    let mut dest = String::new();
    for v in source.values() {
        dest.push_str(v);
        dest.push_str(";\n\n");
    }
    dest
}

fn join_nested_items(source: &BTreeMap<String, BTreeMap<String, String>>, dest: &mut String) {
    for items in source.values() {
        dest.push_str(&join_items(items));
    }
}
