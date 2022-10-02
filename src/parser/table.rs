use std::collections::BTreeMap;

use super::{
    utils::{get_type_name, node_to_embed_constraint},
    Column, SchemaId, Table,
};
use anyhow::anyhow;
use pg_query::{
    protobuf::{ColumnDef, CreateStmt},
    NodeEnum,
};

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
        Ok(Self { id, columns, node })
    }
}

impl TryFrom<&ColumnDef> for Column {
    type Error = anyhow::Error;
    fn try_from(column: &ColumnDef) -> Result<Self, Self::Error> {
        let name = column.colname.clone();
        let data_type = column
            .type_name
            .as_ref()
            .ok_or_else(|| anyhow!("no data type"))?;
        let type_name = get_type_name(data_type);
        // let type_modifier = get_type_mod(data_type);
        let nullable = !column.is_not_null;
        let default = column
            .raw_default
            .as_ref()
            .map(|n| n.node.as_ref().unwrap().deparse().unwrap());
        let constraints = column
            .constraints
            .iter()
            .filter_map(node_to_embed_constraint)
            .collect();
        Ok(Self {
            name,
            type_name,
            nullable,
            default,
            constraints,
        })
    }
}
