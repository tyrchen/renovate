use crate::parser::{
    utils::{get_type_name, node_to_embed_constraint},
    Column,
};
use anyhow::anyhow;
use pg_query::protobuf::ColumnDef;

impl TryFrom<&ColumnDef> for Column {
    type Error = anyhow::Error;
    fn try_from(column: &ColumnDef) -> Result<Self, Self::Error> {
        let name = column.colname.clone();

        let type_name =
            get_type_name(column.type_name.as_ref()).ok_or_else(|| anyhow!("no data type"))?;
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
