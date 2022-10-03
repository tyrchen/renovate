use crate::{
    config::{RenovateFormatConfig, RenovateOutputConfig},
    DatabaseSchema, SqlSaver,
};
use async_trait::async_trait;
use std::{collections::BTreeMap, fmt, path::Path};
use tokio::fs;

macro_rules! join_items {
    ($source:expr, $dest:ident) => {{
        for (_k, v) in $source {
            $dest.push_str(&v);
            $dest.push_str(";\n\n");
        }
    }};
    ($source:expr) => {{
        let mut dest = String::new();
        for (_k, v) in $source {
            dest.push_str(&v);
            dest.push_str(";\n\n");
        }
        dest
    }};
}
macro_rules! join_nested_items {
    ($source:expr, $dest:ident) => {{
        for (_k, items) in $source {
            join_items!(items, $dest);
        }
    }};
}

macro_rules! write_single_file {
    ($source:expr, $name:literal, $config:ident) => {{
        let content = join_items!($source);
        if !content.is_empty() {
            let path = $config.path.join(format!("{}.sql", $name));
            SchemaSaver::write(&path, &content, $config.format).await?;
        }
    }};
}

macro_rules! write_schema_file {
    ($source:expr, $name:literal, $config:ident) => {{
        for (schema, items) in $source {
            let path = $config.path.join(&schema);
            fs::create_dir_all(&path).await?;
            let content = join_items!(items);
            let filename = path.join(format!("{}.sql", $name));
            SchemaSaver::write(&filename, &content, $config.format).await?;
        }
    }};
}

macro_rules! write_schema_files {
    ($source:expr, $name:literal, $config:ident) => {{
        for (schema, items) in $source {
            let path = $config.path.join(&schema);
            fs::create_dir_all(&path).await?;
            for (n, content) in items {
                let p = path.join($name);
                fs::create_dir_all(&p).await?;
                let filename = p.join(format!("{}.sql", n));
                let content = format!("{};\n", content);
                SchemaSaver::write(&filename, &content, $config.format).await?;
            }
        }
    }};
}

/// Temporary struct to hold the data for the generated SQLs
struct SchemaSaver {
    pub types: BTreeMap<String, BTreeMap<String, String>>,
    pub tables: BTreeMap<String, BTreeMap<String, String>>,
    pub views: BTreeMap<String, BTreeMap<String, String>>,
    pub functions: BTreeMap<String, BTreeMap<String, String>>,
    pub triggers: BTreeMap<String, String>,
    pub privileges: BTreeMap<String, String>,
}

macro_rules! collect_schema_items {
    ($source:expr) => {{
        let mut dest = BTreeMap::new();
        for (schema, items) in $source {
            let result = items
                .into_iter()
                .map(|(name, ty)| (name.clone(), ty.node.deparse().unwrap()))
                .collect::<BTreeMap<_, _>>();
            dest.insert(schema.clone(), result);
        }
        dest
    }};
}

macro_rules! collect_db_items {
    ($source:expr) => {{
        let mut dest = BTreeMap::new();
        for (k, v) in $source {
            dest.insert(k.clone(), v.node.deparse()?);
        }
        dest
    }};
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
        let types = collect_schema_items!(&schema.types);
        let tables = collect_schema_items!(&schema.tables);
        let views = collect_schema_items!(&schema.views);
        let functions = collect_schema_items!(&schema.functions);

        let triggers = collect_db_items!(&schema.triggers);
        let privileges = collect_db_items!(&schema.privileges);

        Ok(Self {
            types,
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
        write_schema_files!(&self.types, "types", config);
        write_schema_files!(&self.tables, "tables", config);
        write_schema_files!(&self.views, "views", config);
        write_schema_files!(&self.functions, "functions", config);

        write_single_file!(&self.triggers, "triggers", config);
        write_single_file!(&self.privileges, "privileges", config);

        Ok(())
    }

    pub async fn normal(&self, config: &RenovateOutputConfig) -> anyhow::Result<()> {
        write_schema_file!(&self.types, "types", config);
        write_schema_file!(&self.tables, "tables", config);
        write_schema_file!(&self.views, "views", config);
        write_schema_file!(&self.functions, "functions", config);

        write_single_file!(&self.triggers, "triggers", config);
        write_single_file!(&self.privileges, "privileges", config);

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

        join_nested_items!(&self.types, result);
        join_nested_items!(&self.tables, result);
        join_nested_items!(&self.views, result);
        join_nested_items!(&self.functions, result);

        join_items!(&self.triggers, result);
        join_items!(&self.privileges, result);

        write!(f, "{}", result)
    }
}
