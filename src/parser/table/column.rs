use std::collections::BTreeSet;

use crate::{
    parser::{
        utils::{get_type_name, node_to_embed_constraint},
        Column, Table,
    },
    DeltaItem,
};
use anyhow::anyhow;
use pg_query::{
    protobuf::{ColumnDef, ConstrType},
    NodeEnum,
};

impl TryFrom<&ColumnDef> for Column {
    type Error = anyhow::Error;
    fn try_from(column: &ColumnDef) -> Result<Self, Self::Error> {
        let name = column.colname.clone();

        let type_nodes = &column
            .type_name
            .as_ref()
            .ok_or_else(|| anyhow!("no data type"))?
            .names;
        let type_name = get_type_name(type_nodes);
        // let type_modifier = get_type_mod(data_type);

        let constraints: BTreeSet<_> = column
            .constraints
            .iter()
            .filter_map(node_to_embed_constraint)
            .collect();

        let nullable = !constraints
            .iter()
            .any(|c| c.con_type == ConstrType::ConstrNotnull);

        Ok(Self {
            name,
            type_name,
            nullable,
            constraints,
        })
    }
}

impl Column {
    fn generate_change(self, _item: &Table) -> anyhow::Result<NodeEnum> {
        todo!()
    }
}

impl DeltaItem for Column {
    type SqlNode = Table;
    fn drop(self, item: &Self::SqlNode) -> anyhow::Result<Vec<String>> {
        let node = self.generate_change(item)?;
        let sql = format!("{};", node.deparse()?);
        Ok(vec![sql])
    }

    fn create(self, item: &Self::SqlNode) -> anyhow::Result<Vec<String>> {
        let node = self.generate_change(item)?;
        let sql = format!("{};", node.deparse()?);
        Ok(vec![sql])
    }

    fn alter(self, item: &Self::SqlNode, remote: Self) -> anyhow::Result<Vec<String>> {
        let mut migrations = vec![];
        let sql = self.drop(item)?;
        migrations.extend(sql);
        let sql = remote.create(item)?;
        migrations.extend(sql);
        Ok(migrations)
    }
}
