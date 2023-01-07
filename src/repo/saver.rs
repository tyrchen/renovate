use crate::{
    config::{RenovateFormatConfig, RenovateOutputConfig},
    DatabaseSchema, MigrationPlanner, NodeDiff, NodeItem, SqlSaver,
};
use anyhow::Result;
use async_trait::async_trait;
use std::{collections::BTreeMap, fmt, hash::Hash, path::Path, str::FromStr};
use tokio::fs;

#[async_trait]
impl SqlSaver for DatabaseSchema {
    async fn save(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        use crate::config::Layout;

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
        write_schema_files(&self.composite_types, "types", config).await?;
        write_schema_files(&self.enum_types, "enums", config).await?;

        write_schema_files(&self.sequences, "sequences", config).await?;
        write_schema_files(&self.tables, "tables", config).await?;
        write_schema_files(&self.table_constraints, "constraints", config).await?;
        write_schema_files(&self.table_indexes, "indexes", config).await?;
        write_schema_files(&self.views, "views", config).await?;
        write_schema_files(&self.mviews, "mviews", config).await?;
        write_schema_files(&self.functions, "functions", config).await?;

        write_single_file(&self.triggers, "triggers", config).await?;
        write_single_file(&self.privileges, "privileges", config).await?;

        Ok(())
    }

    pub async fn normal(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        write_schema_file(&self.composite_types, "types", config).await?;
        write_schema_file(&self.enum_types, "enums", config).await?;

        write_schema_file(&self.sequences, "sequences", config).await?;
        write_schema_file(&self.tables, "tables", config).await?;
        write_schema_file(&self.table_constraints, "constraints", config).await?;
        write_schema_file(&self.table_indexes, "indexes", config).await?;
        write_schema_file(&self.views, "views", config).await?;
        write_schema_file(&self.mviews, "mviews", config).await?;
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

impl fmt::Display for DatabaseSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        // join_nested_items(&self.extensions, &mut result);
        join_nested_items(&self.composite_types, &mut result);
        join_nested_items(&self.enum_types, &mut result);
        join_nested_items(&self.sequences, &mut result);
        join_nested_items(&self.tables, &mut result);
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

async fn write_schema_files<K, T>(
    source: &BTreeMap<K, BTreeMap<String, T>>,
    name: &str,
    config: &RenovateOutputConfig,
) -> Result<()>
where
    K: Hash + Eq + Ord + ToString + Clone + 'static,
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    for (schema, items) in source {
        let schema = schema.to_string();
        let schema = schema.split('.').next().unwrap();

        let path = config.path.join(schema);
        fs::create_dir_all(&path).await?;
        for (n, content) in items {
            let p = path.join(name);
            fs::create_dir_all(&p).await?;
            let filename = p.join(format!("{}.sql", n));
            let content = format!("{};\n", content.to_string());
            DatabaseSchema::write(&filename, &content, config.format).await?;
        }
    }
    Ok(())
}

async fn write_schema_file<K, T>(
    source: &BTreeMap<K, BTreeMap<String, T>>,
    name: &str,
    config: &RenovateOutputConfig,
) -> Result<()>
where
    K: Hash + Eq + Ord + ToString + Clone + 'static,
    T: NodeItem + Clone + FromStr<Err = anyhow::Error> + PartialEq + Eq + 'static,
    NodeDiff<T>: MigrationPlanner<Migration = String>,
{
    for (schema, items) in source {
        let schema = schema.to_string();
        let schema = schema.split('.').next().unwrap();
        let path = config.path.join(schema);
        fs::create_dir_all(&path).await?;
        let content = join_items(items);
        let filename = path.join(format!("{}.sql", name));
        DatabaseSchema::write(&filename, &content, config.format).await?;
    }

    Ok(())
}

async fn write_single_file<T>(
    source: &BTreeMap<String, T>,
    name: &str,
    config: &RenovateOutputConfig,
) -> Result<()>
where
    T: NodeItem,
{
    let content = join_items(source);
    if !content.is_empty() {
        let path = config.path.join(format!("{}.sql", name));
        DatabaseSchema::write(&path, &content, config.format).await?;
    }
    Ok(())
}

fn join_items<T>(source: &BTreeMap<String, T>) -> String
where
    T: NodeItem,
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
