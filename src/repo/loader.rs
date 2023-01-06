use crate::{
    map_insert_relation, map_insert_schema,
    parser::{
        AlterTable, AlterTableAction, CompositeType, EnumType, Function, MatView, Privilege, Table,
        TableConstraint, TableIndex, TableOwner, TableRls, Trigger, View,
    },
    utils::ignore_file,
    DatabaseSchema, LocalRepo, RemoteRepo, SchemaLoader,
};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use glob::glob;
use pg_query::NodeRef;
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

        // parse the sql to see if the syntax is correct
        let ret = pg_query::parse(&sql)?;
        let sql = ret.deparse()?;

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
            .await?;

        if !output.status.success() {
            bail!("{}", String::from_utf8(output.stderr)?);
        }

        let sql = String::from_utf8(output.stdout)?;
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
                NodeRef::CompositeTypeStmt(stmt) => {
                    let item: CompositeType = stmt.try_into()?;
                    map_insert_schema!(data.composite_types, item);
                }
                NodeRef::CreateEnumStmt(stmt) => {
                    let item: EnumType = stmt.try_into()?;
                    map_insert_schema!(data.enum_types, item);
                }
                NodeRef::CreateStmt(stmt) => {
                    let item: Table = stmt.try_into()?;
                    map_insert_schema!(data.tables, item);
                }
                NodeRef::ViewStmt(stmt) => {
                    let item: View = stmt.try_into()?;
                    map_insert_schema!(data.views, item);
                }
                NodeRef::CreateTableAsStmt(stmt) => {
                    let item: MatView = stmt.try_into()?;
                    map_insert_schema!(data.mviews, item);
                }
                NodeRef::CreateFunctionStmt(stmt) => {
                    let item: Function = stmt.try_into()?;
                    map_insert_schema!(data.functions, item);
                }
                NodeRef::CreateTrigStmt(stmt) => {
                    let item: Trigger = stmt.try_into()?;
                    data.triggers.insert(item.id.name.clone(), item);
                }
                NodeRef::AlterTableStmt(stmt) => {
                    let item: AlterTable = stmt.try_into()?;
                    match &item.action {
                        AlterTableAction::Constraint(_) => {
                            let constraint: TableConstraint = item.try_into()?;
                            map_insert_relation!(data.table_constraints, constraint);
                        }
                        AlterTableAction::Rls => {
                            let rls: TableRls = item.try_into()?;
                            data.table_rls.insert(rls.id.clone(), rls);
                        }
                        AlterTableAction::Owner(_) => {
                            let owner: TableOwner = item.try_into()?;
                            data.table_owners.insert(owner.id.clone(), owner);
                        }
                        _ => {
                            info!("ignore alter table action: {:?}", item.action);
                        }
                    }
                }
                NodeRef::IndexStmt(index) => {
                    let item: TableIndex = index.try_into()?;
                    map_insert_relation!(data.table_indexes, item);
                }
                NodeRef::GrantStmt(grant) => {
                    let item: Privilege = grant.try_into()?;
                    data.privileges.insert(item.id.clone(), item);
                }
                NodeRef::CommentStmt(_comment) => {
                    info!("ignore comment");
                }
                NodeRef::CreateExtensionStmt(_ext) => {
                    info!("ignore extension");
                }
                NodeRef::CreateSchemaStmt(_schema) => {
                    info!("ignoring schema");
                }
                NodeRef::CreateSeqStmt(_seq) => {
                    info!("ignore seq for now");
                }
                NodeRef::CreateForeignTableStmt(_table) => {
                    info!("ignore foreign table for now");
                }
                NodeRef::CreateForeignServerStmt(_server) => {
                    info!("ignore foreign server for now");
                }
                NodeRef::CreateFdwStmt(_fdw) => {
                    info!("ignore fwd for now");
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
