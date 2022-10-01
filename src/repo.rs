use crate::{
    parser::{Constraint, Function, Index, Table, Trigger, View},
    utils::ignore_file,
    DatabaseSchema, LocalRepo, RemoteRepo, SchemaLoader,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use glob::glob;
use pg_query::NodeRef;
use std::path::PathBuf;
use tokio::fs;

/// intermediate representation for local and remote repo
#[derive(Debug, Clone)]
struct SqlRepo(String);

#[async_trait]
impl SchemaLoader for LocalRepo {
    async fn load(&self) -> Result<DatabaseSchema> {
        // load all the .sql files in subdirectories except the "_meta" directory
        let files = glob("**/*.sql")?
            .filter_map(Result::ok)
            .filter(|p| ignore_file(p, "_"))
            .collect::<Vec<PathBuf>>();

        // concatenate all the sql files into one string
        let mut sql = String::with_capacity(16 * 1024);
        for file in files {
            let content = fs::read_to_string(file).await?;
            sql.push_str(&content);
        }

        SqlRepo(sql).load().await
    }
}

#[async_trait]
impl SchemaLoader for RemoteRepo {
    /// run pg_dump us async process and get the output sql
    async fn load(&self) -> anyhow::Result<crate::DatabaseSchema> {
        let output = async_process::Command::new("pg_dump")
            .arg("-s")
            .arg(&self.url)
            .output()
            .await?
            .stdout;
        let sql = String::from_utf8(output)?;
        SqlRepo(sql).load().await
    }
}

macro_rules! map_insert_schema {
    ($map:expr, $item:ident) => {
        $map.entry($item.id.schema.clone())
            .or_insert(Default::default())
            .insert($item.id.name.clone(), $item);
    };
}

macro_rules! map_insert_relation {
    ($map:expr, $item:ident) => {
        $map.entry($item.id.schema_id.clone())
            .or_insert(Default::default())
            .insert($item.id.name.clone(), $item);
    };
}

macro_rules! map_insert {
    ($map:expr, $item:ident) => {
        $map.insert($item.id.clone(), $item);
    };
}

#[async_trait]
impl SchemaLoader for SqlRepo {
    async fn load(&self) -> Result<DatabaseSchema> {
        let result = pg_query::parse(&self.0)?;
        let nodes = result.protobuf.nodes();
        let mut data = DatabaseSchema::default();

        for (node, _, _) in nodes {
            match node {
                NodeRef::CreateStmt(table) => {
                    let item = Table::from(table);
                    map_insert_schema!(data.tables, item);
                }
                NodeRef::ViewStmt(view) => {
                    let item = View::from(view);
                    map_insert_schema!(data.views, item);
                }
                NodeRef::CreateTableAsStmt(mview) => {
                    let item = View::from(mview);
                    map_insert_schema!(data.views, item);
                }
                NodeRef::CreateFunctionStmt(func) => {
                    let item = Function::from(func);
                    map_insert_schema!(data.functions, item);
                }
                NodeRef::CreateTrigStmt(trig) => {
                    let item = Trigger::from(trig);
                    map_insert!(data.triggers, item);
                }
                NodeRef::AlterTableStmt(alter) => {
                    if let Ok(item) = Constraint::try_from(alter) {
                        map_insert_relation!(data.constraints, item);
                    } else {
                        todo!("alter table");
                    }
                }
                NodeRef::IndexStmt(index) => {
                    let item = Index::from(index);
                    map_insert_relation!(data.indexes, item);
                }
                NodeRef::GrantStmt(_grant) => {
                    todo!()
                }
                NodeRef::CommentStmt(_comment) => {
                    todo!()
                }
                NodeRef::CreateExtensionStmt(_ext) => {
                    todo!()
                }
                NodeRef::CreateSchemaStmt(_schema) => {
                    todo!()
                }
                NodeRef::CreateSeqStmt(_seq) => {
                    todo!()
                }
                NodeRef::CreateForeignTableStmt(_table) => {
                    todo!()
                }
                NodeRef::CreateForeignServerStmt(_server) => {
                    todo!()
                }
                NodeRef::CreateFdwStmt(_fdw) => {
                    todo!()
                }
                NodeRef::CreatePolicyStmt(_policy) => {
                    todo!()
                }
                _ => return Err(anyhow!(format!("Unsupported top level node: {:?}", node))),
            }
        }
        Ok(data)
    }
}

impl LocalRepo {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl RemoteRepo {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl Default for LocalRepo {
    fn default() -> Self {
        Self::new(".")
    }
}
