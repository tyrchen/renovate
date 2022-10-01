use crate::{
    parser::{Constraint, Function, Index, RawDatabaseSchema, Table, Trigger, View},
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

#[async_trait]
impl SchemaLoader for SqlRepo {
    async fn load(&self) -> Result<DatabaseSchema> {
        let result = pg_query::parse(&self.0)?;
        let nodes = result.protobuf.nodes();
        let mut data = RawDatabaseSchema::default();

        for (node, _, _) in nodes {
            match node {
                NodeRef::CreateStmt(table) => {
                    let table = Table::from(table);
                    data.tables.insert(table.id.clone(), table);
                }
                NodeRef::ViewStmt(view) => {
                    let view = View::from(view);
                    data.views.insert(view.id.clone(), view);
                }
                NodeRef::CreateTableAsStmt(mview) => {
                    let view = View::from(mview);
                    data.views.insert(view.id.clone(), view);
                }
                NodeRef::CreateFunctionStmt(func) => {
                    let func = Function::from(func);
                    data.functions.insert(func.id.clone(), func);
                }
                NodeRef::CreateTrigStmt(trig) => {
                    let trigger = Trigger::from(trig);
                    data.triggers.insert(trigger.id.clone(), trigger);
                }
                NodeRef::AlterTableStmt(alter) => {
                    if let Ok(constraint) = Constraint::try_from(alter) {
                        data.constraints.insert(constraint.id.clone(), constraint);
                    } else {
                        todo!("alter table");
                    }
                }
                NodeRef::IndexStmt(index) => {
                    let index = Index::from(index);
                    data.indexes.insert(index.id.clone(), index);
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
        Ok(data.into())
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
