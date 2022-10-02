use crate::{DatabaseSchema, Layout, SqlSaver};
use async_trait::async_trait;
use std::{collections::BTreeMap, path::Path};
use tokio::fs;

macro_rules! join_items {
    ($source:expr, $dest:ident) => {{
        for (_k, v) in $source {
            $dest.push_str(&v);
        }
    }};
    ($source:expr) => {{
        let mut dest = String::new();
        for (_k, v) in $source {
            dest.push_str(&v);
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
    ($source:expr, $name:expr, $path:expr) => {{
        let content = join_items!($source);
        let path = $path.join(format!("{}.sql", $name));
        fs::write(
            path,
            sqlformat::format(&content, &Default::default(), Default::default()),
        )
        .await?;
    }};
}

macro_rules! write_schema_file {
    ($source:expr, $name:expr, $path:expr) => {{
        for (schema, items) in $source {
            let path = $path.join(&schema);
            fs::create_dir_all(&path).await?;
            write_single_file!(items, $name, &path);
        }
    }};
}

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
    async fn save(&self, path: &Path, layout: Layout) -> anyhow::Result<()> {
        let saver = SchemaSaver::try_from(self)?;
        match layout {
            Layout::Normal => saver.normal(path).await,
            Layout::Flat => saver.flat(path).await,
            Layout::Nested => saver.nested(path).await,
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
    pub async fn flat(&self, path: &Path) -> anyhow::Result<()> {
        let content = self.to_string();
        fs::write(path, content).await?;
        Ok(())
    }

    pub async fn nested(&self, path: &Path) -> anyhow::Result<()> {
        write_single_file!(&self.triggers, "triggers", path);
        write_single_file!(&self.privileges, "privileges", path);

        Ok(())
    }

    pub async fn normal(&self, path: &Path) -> anyhow::Result<()> {
        write_schema_file!(&self.types, "types", path);
        write_schema_file!(&self.tables, "tables", path);
        write_schema_file!(&self.views, "views", path);
        write_schema_file!(&self.functions, "functions", path);

        write_single_file!(&self.triggers, "triggers", path);
        write_single_file!(&self.privileges, "privileges", path);
        Ok(())
    }
}

impl ToString for SchemaSaver {
    /// combine all strings into one
    fn to_string(&self) -> String {
        let mut result = String::new();

        join_nested_items!(&self.types, result);
        join_nested_items!(&self.tables, result);
        join_nested_items!(&self.views, result);
        join_nested_items!(&self.functions, result);

        join_items!(&self.triggers, result);
        join_items!(&self.privileges, result);

        result
    }
}
