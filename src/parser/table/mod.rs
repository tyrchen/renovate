mod alter_table;
mod column;
mod table_constraint;
mod table_owner;
mod table_rls;

use crate::{DiffItem, MigrationPlanner, NodeDelta, NodeDiff};

use super::{Column, SchemaId, Table};
use anyhow::Context;
use pg_query::{protobuf::CreateStmt, NodeEnum, NodeRef};
use std::{collections::BTreeMap, str::FromStr};

impl DiffItem for Table {
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }
}

impl FromStr for Table {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parsed = pg_query::parse(s).with_context(|| format!("Failed to parse table: {}", s))?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::CreateStmt(stmt) => Self::try_from(stmt),
            _ => anyhow::bail!("not a table: {}", s),
        }
    }
}

impl TryFrom<&CreateStmt> for Table {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateStmt) -> Result<Self, Self::Error> {
        let id = SchemaId::from(stmt.relation.as_ref());
        let columns = get_columns(stmt)?;
        let node = NodeEnum::CreateStmt(stmt.clone());
        Ok(Self { id, columns, node })
    }
}

impl MigrationPlanner for NodeDiff<Table> {
    type Migration = String;

    fn drop(&self) -> anyhow::Result<Option<Self::Migration>> {
        if let Some(old) = &self.old {
            Ok(Some(format!("DROP TABLE {};", old.id)))
        } else {
            Ok(None)
        }
    }

    fn create(&self) -> anyhow::Result<Option<Self::Migration>> {
        if let Some(new) = &self.new {
            Ok(Some(format!("{};", new.node.deparse()?)))
        } else {
            Ok(None)
        }
    }

    fn alter(&self) -> anyhow::Result<Option<Vec<Self::Migration>>> {
        match (&self.old, &self.new) {
            (Some(old), Some(new)) => {
                let delta = NodeDelta::calculate(&old.columns, &new.columns);
                let mut migrations = Vec::new();
                for removed in delta.removed {
                    let sql = gen_table_sql(&old.node, Some(removed), false)?;
                    migrations.push(format!("{};", sql));
                }

                for added in delta.added {
                    let sql = gen_table_sql(&old.node, Some(added), true)?;
                    migrations.push(format!("{};", sql));
                }

                for (v1, v2) in delta.changed {
                    let sql = gen_table_sql(&old.node, Some(v1), false)?;
                    migrations.push(format!("{};", sql));
                    let sql = gen_table_sql(&new.node, Some(v2), true)?;
                    migrations.push(format!("{};", sql));
                }
                Ok(Some(migrations))
            }
            _ => Ok(None),
        }
    }
}

fn get_columns(stmt: &CreateStmt) -> anyhow::Result<BTreeMap<String, Column>> {
    let mut columns = BTreeMap::new();

    for node in stmt.table_elts.iter().filter_map(|n| n.node.as_ref()) {
        if let NodeEnum::ColumnDef(ref column) = node {
            let column = Column::try_from(column.as_ref())?;
            columns.insert(column.name.clone(), column);
        }
    }
    Ok(columns)
}

fn gen_table_sql(
    _stmt: &NodeEnum,
    _column: Option<Column>,
    _grant: bool,
) -> anyhow::Result<String> {
    todo!()
}
