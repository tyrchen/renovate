use crate::{
    parser::{Constraint, Function, Index, SchemaId, Table, Trigger, View},
    utils::ignore_file,
    DatabaseSchema, LocalRepo, RemoteRepo, SchemaLoader,
};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use glob::glob;
use pg_query::{protobuf::AlterTableType, NodeEnum, NodeRef};
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
                    let item = View::from(view);
                    map_insert_schema!(data.views, item);
                }
                NodeRef::CreateTableAsStmt(mview) => {
                    let item = View::from(mview);
                    map_insert_schema!(data.views, item);
                }
                NodeRef::CreateFunctionStmt(func) => {
                    let item = Function::try_from(func).with_context(|| {
                        let sql = NodeEnum::CreateFunctionStmt(func.clone()).deparse();
                        format!("Failed to convert: {:?}", sql)
                    })?;
                    map_insert_schema!(data.functions, item);
                }
                NodeRef::CreateTrigStmt(trig) => {
                    let item = Trigger::from(trig);
                    map_insert!(data.triggers, item);
                }
                NodeRef::AlterTableStmt(alter) => {
                    let range_var = alter
                        .relation
                        .as_ref()
                        .ok_or_else(|| anyhow!("no relation"))?;

                    let id = SchemaId::from(range_var);
                    let cmd = alter
                        .cmds
                        .iter()
                        .filter_map(|n| n.node.as_ref())
                        .next()
                        .ok_or_else(|| anyhow!("no commands"))?;
                    match cmd {
                        NodeEnum::AlterTableCmd(ref cmd) => {
                            match AlterTableType::from_i32(cmd.as_ref().subtype) {
                                Some(AlterTableType::AtAddConstraint) => {
                                    let node = cmd
                                        .def
                                        .as_ref()
                                        .ok_or_else(|| anyhow!("no def"))?
                                        .node
                                        .as_ref()
                                        .ok_or_else(|| anyhow!("no node"))?;
                                    match node {
                                        NodeEnum::Constraint(constraint) => {
                                            let item = Constraint::try_from((
                                                id,
                                                alter,
                                                constraint.as_ref(),
                                            ))
                                            .with_context(|| {
                                                let sql = NodeEnum::Constraint(constraint.clone())
                                                    .deparse();
                                                format!("Failed to convert: {:?}", sql)
                                            })?;
                                            map_insert_relation!(data.constraints, item);
                                        }
                                        _ => {
                                            return Err(anyhow!("unknown constraint: {:?}", node));
                                        }
                                    }
                                }
                                Some(AlterTableType::AtAddIndex) => todo!(),
                                Some(AlterTableType::AtAddColumn) => todo!(),
                                Some(AlterTableType::AtAddIndexConstraint) => todo!(),
                                _ => todo!(),
                            }
                        }
                        _ => return Err(anyhow!("unknown command")),
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
