mod alter_table;
mod column;
mod table_constraint;
mod table_owner;
mod table_rls;

use super::{Column, SchemaId, Table};
use anyhow::anyhow;
use debug_ignore::DebugIgnore;
use pg_query::{protobuf::CreateStmt, NodeEnum};
use std::collections::BTreeMap;

impl TryFrom<&CreateStmt> for Table {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateStmt) -> Result<Self, Self::Error> {
        let range_var = stmt
            .relation
            .as_ref()
            .ok_or_else(|| anyhow!("no relation"))?;

        let id = SchemaId::from(range_var);
        let mut columns = BTreeMap::new();
        for node in stmt.table_elts.iter().filter_map(|n| n.node.as_ref()) {
            if let NodeEnum::ColumnDef(ref column) = node {
                let column = Column::try_from(column.as_ref())?;
                columns.insert(column.name.clone(), column);
            }
        }
        let node = NodeEnum::CreateStmt(stmt.clone());
        Ok(Self {
            id,
            columns,
            node: DebugIgnore(node),
        })
    }
}
