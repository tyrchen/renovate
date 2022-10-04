mod alter_table;
mod column;
mod table_constraint;
mod table_owner;
mod table_rls;

use crate::{DiffItem, MigrationPlanner, MigrationResult, NodeDelta, NodeDiff};

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

    fn drop(&self) -> MigrationResult<Self::Migration> {
        if let Some(old) = &self.old {
            let sqls = gen_table_sql(&old.node, None, false)?;
            Ok(sqls)
        } else {
            Ok(vec![])
        }
    }

    fn create(&self) -> MigrationResult<Self::Migration> {
        if let Some(new) = &self.new {
            let sqls = gen_table_sql(&new.node, None, true)?;
            Ok(sqls)
        } else {
            Ok(vec![])
        }
    }

    fn alter(&self) -> MigrationResult<Self::Migration> {
        match (&self.old, &self.new) {
            (Some(old), Some(new)) => {
                let delta = NodeDelta::calculate(&old.columns, &new.columns);
                delta.plan(&old.node, gen_table_sql, gen_table_sql_for_changes)
            }
            _ => Ok(vec![]),
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
    _add: bool,
) -> anyhow::Result<Vec<String>> {
    todo!()
}

fn gen_table_sql_for_changes(
    _stmt: &NodeEnum,
    _v1: Column,
    _v2: Column,
) -> anyhow::Result<Vec<String>> {
    todo!()
}
