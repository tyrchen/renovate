use std::collections::BTreeSet;

use crate::{
    parser::{
        utils::{get_type_name, node_to_embed_constraint},
        Column, RelationId, SchemaId, Table,
    },
    DeltaItem,
};
use anyhow::anyhow;
use pg_query::protobuf::{ColumnDef, ConstrType};

impl TryFrom<(SchemaId, ColumnDef)> for Column {
    type Error = anyhow::Error;
    fn try_from((id, column): (SchemaId, ColumnDef)) -> Result<Self, Self::Error> {
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
            id: RelationId::new_with(id, name),
            type_name,
            nullable,
            constraints,
        })
    }
}

impl Column {
    fn generate_add_sql(self) -> anyhow::Result<String> {
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            self.id.schema_id, self.id.name, self.type_name
        );
        if !self.nullable {
            sql.push_str(" NOT NULL");
        }
        // for constraint in self.constraints {
        //     sql.push_str(&format!(" {}", constraint.generate_sql()?));
        // }
        Ok(sql)
    }
}

impl DeltaItem for Column {
    type SqlNode = Table;
    fn drop(self, item: &Self::SqlNode) -> anyhow::Result<Vec<String>> {
        let sql = format!("ALTER TABLE {} DROP COLUMN {}", item.id, self.id.name);

        Ok(vec![sql])
    }

    fn create(self, _item: &Self::SqlNode) -> anyhow::Result<Vec<String>> {
        let sql = self.generate_add_sql()?;
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
