use crate::{
    map_insert_relation, map_insert_schema,
    parser::{
        AlterTable, AlterTableAction, Function, MatView, Table, TableConstraint, TableIndex,
        TableOwner, TableRls, Trigger, View,
    },
    utils::ignore_file,
    DatabaseSchema, LocalRepo, RemoteRepo, SchemaLoader,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use glob::glob;
use pg_query::{NodeEnum, NodeRef};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

/// intermediate representation for local and remote repo
#[derive(Debug, Clone)]
struct SqlRepo(String);

#[async_trait]
impl SchemaLoader for LocalRepo {
    async fn load(&self) -> Result<DatabaseSchema> {
        // load all the .sql files in subdirectories except the "_meta" directory
        let glob_path = self.path.join("**/*.sql");
        let files = glob(glob_path.as_os_str().to_str().unwrap())?
            .filter_map(Result::ok)
            .filter(|p| ignore_file(p, "_"))
            .collect::<Vec<PathBuf>>();

        // concatenate all the sql files into one string
        let mut sql = String::with_capacity(16 * 1024);
        for file in files {
            let content = fs::read_to_string(file.as_path())
                .await
                .with_context(|| format!("Failed to read file: {:?}", file))?;
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
        let result = pg_query::parse(&self.0).with_context(|| "Failed to parse SQL statements")?;
        let nodes = result.protobuf.nodes();
        let mut data = DatabaseSchema::default();

        for (node, _, _) in nodes {
            match node {
                NodeRef::CreateStmt(table) => {
                    let item = Table::try_from(table).with_context(|| {
                        let sql = NodeEnum::CreateStmt(table.clone()).deparse();
                        format!("Failed to convert: {:?}", sql)
                    })?;
                    map_insert_schema!(data.tables, item);
                }
                NodeRef::ViewStmt(view) => {
                    let item = View::try_from(view)?;
                    map_insert_schema!(data.views, item);
                }
                NodeRef::CreateTableAsStmt(mview) => {
                    let item = MatView::try_from(mview)?;
                    map_insert_schema!(data.mviews, item);
                }
                NodeRef::CreateFunctionStmt(func) => {
                    let item = Function::try_from(func).with_context(|| {
                        let sql = NodeEnum::CreateFunctionStmt(func.clone()).deparse();
                        format!("Failed to convert: {:?}", sql)
                    })?;
                    map_insert_schema!(data.functions, item);
                }
                NodeRef::CreateTrigStmt(trig) => {
                    let item = Trigger::try_from(trig)?;
                    data.triggers.insert(item.id.name.clone(), item);
                }
                NodeRef::AlterTableStmt(alter) => {
                    let alter_table = AlterTable::try_from(alter)?;
                    match &alter_table.action {
                        AlterTableAction::Constraint(_) => {
                            let constraint = TableConstraint::try_from(alter_table)?;
                            map_insert_relation!(data.table_constraints, constraint);
                        }
                        AlterTableAction::Rls => {
                            let rls = TableRls::try_from(alter_table)?;
                            data.table_rls.insert(rls.id.clone(), rls);
                        }
                        AlterTableAction::Owner(_) => {
                            let owner = TableOwner::try_from(alter_table)?;
                            data.table_owners.insert(owner.id.clone(), owner);
                        }
                    }
                }
                NodeRef::IndexStmt(index) => {
                    let item = TableIndex::try_from(index)?;
                    map_insert_relation!(data.table_indexes, item);
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
                    info!("ignoring schema");
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
                _ => {
                    info!("unhandled node: {:?}", node.deparse());
                }
            }
        }
        Ok(data)
    }
}
