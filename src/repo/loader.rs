use crate::{
    map_insert_relation, map_insert_schema,
    parser::{
        AlterTable, AlterTableAction, CompositeType, EnumType, Function, MatView, Privilege,
        Sequence, Table, TableConstraint, TableIndex, TableOwner, TablePolicy, TableRls,
        TableSequence, Trigger, View,
    },
    utils::ignore_file,
    DatabaseRepo, DatabaseSchema, LocalRepo, SchemaLoader, SqlLoader,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use glob::glob;
use pg_query::NodeRef;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

#[async_trait]
impl SchemaLoader for LocalRepo {
    async fn load(&self) -> Result<DatabaseSchema> {
        let sql = self.load_sql().await?;
        SqlLoader(sql).load().await
    }

    async fn load_sql(&self) -> Result<String> {
        let files = self.files()?;
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
        Ok(sql)
    }
}

#[async_trait]
impl SchemaLoader for DatabaseRepo {
    /// run pg_dump us async process and get the output sql
    async fn load(&self) -> anyhow::Result<crate::DatabaseSchema> {
        let sql = self.load_sql().await?;
        SqlLoader(sql).load().await
    }

    async fn load_sql(&self) -> anyhow::Result<String> {
        #[cfg(feature = "cli-test")]
        self.init_local_database().await?;
        self.load_sql_string(false).await
    }
}

#[async_trait]
impl SchemaLoader for SqlLoader {
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
                    map_insert_relation!(data.table_triggers, item);
                }
                NodeRef::AlterTableStmt(stmt) => {
                    let item: AlterTable = stmt.try_into()?;
                    match &item.action {
                        AlterTableAction::Constraint(_) => {
                            let constraint: TableConstraint = item.try_into()?;
                            map_insert_relation!(data.table_constraints, constraint);
                        }
                        AlterTableAction::Sequence(_) => {
                            let sequence: TableSequence = item.try_into()?;
                            map_insert_relation!(data.table_sequences, sequence);
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
                    data.privileges
                        .entry(item.id.clone())
                        .or_default()
                        .insert(item);
                }
                NodeRef::CommentStmt(_comment) => {
                    info!("ignore comment");
                }
                NodeRef::CreateExtensionStmt(_ext) => {
                    info!("TODO: extension");
                }
                NodeRef::CreateSchemaStmt(_schema) => {
                    info!("ignore schema creation statement since we already have the schema name");
                }
                NodeRef::CreateSeqStmt(seq) => {
                    let item: Sequence = seq.try_into()?;
                    map_insert_schema!(data.sequences, item);
                }
                NodeRef::CreateForeignTableStmt(_table) => {
                    info!("TODO: foreign table");
                }
                NodeRef::CreateForeignServerStmt(_server) => {
                    info!("TODO: foreign server");
                }
                NodeRef::CreateFdwStmt(_fdw) => {
                    info!("TODO: fwd");
                }
                NodeRef::CreatePolicyStmt(policy) => {
                    let item: TablePolicy = policy.try_into()?;
                    map_insert_relation!(data.table_policies, item);
                }
                _ => {
                    info!("unhandled node: {:?}", node.deparse());
                }
            }
        }
        data.update_schema_names();
        Ok(data)
    }

    async fn load_sql(&self) -> anyhow::Result<String> {
        Ok(self.0.clone())
    }
}

impl LocalRepo {
    // load all the .sql files in subdirectories except the "_meta" directory
    pub fn files(&self) -> Result<Vec<PathBuf>> {
        let glob_path = self.path.join("**/*.sql");
        let mut files = glob(glob_path.as_os_str().to_str().unwrap())?
            .filter_map(Result::ok)
            .filter(|p| ignore_file(p, "_"))
            .collect::<Vec<PathBuf>>();

        files.sort();
        Ok(files)
    }
}
