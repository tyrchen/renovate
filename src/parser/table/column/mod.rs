mod constraint_info;

use crate::{
    parser::{
        utils::{node_to_embed_constraint, type_name_to_string},
        Column, RelationId, SchemaId, Table,
    },
    DeltaItem,
};
use pg_query::{
    protobuf::{ColumnDef, ConstrType},
    NodeEnum,
};
use std::collections::BTreeSet;

impl TryFrom<(SchemaId, ColumnDef)> for Column {
    type Error = anyhow::Error;
    fn try_from((id, column): (SchemaId, ColumnDef)) -> Result<Self, Self::Error> {
        let name = column.colname.clone();

        let type_name = type_name_to_string(column.type_name.as_ref().unwrap());

        let mut constraints = BTreeSet::new();

        let all_constraints: Vec<_> = column
            .constraints
            .iter()
            .filter_map(node_to_embed_constraint)
            .collect();

        let mut nullable = true;
        let mut default = None;
        for constraint in all_constraints {
            match constraint.con_type {
                ConstrType::ConstrNotnull => {
                    nullable = false;
                }
                ConstrType::ConstrDefault => {
                    default = Some(constraint);
                }
                _ => {
                    constraints.insert(constraint);
                }
            }
        }

        Ok(Self {
            id: RelationId::new_with(id, name),
            type_name,
            nullable,
            constraints,
            default,
            node: NodeEnum::ColumnDef(Box::new(column)),
        })
    }
}

impl Column {
    pub(super) fn generate_add_sql(self) -> anyhow::Result<String> {
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            self.id.schema_id, self.id.name, self.type_name
        );
        if !self.nullable {
            sql.push_str(" NOT NULL ");
        }
        if let Some(default) = self.default.as_ref() {
            sql.push_str(default.generate_sql()?.as_str());
        }
        for constraint in self.constraints {
            sql.push_str(&constraint.generate_sql()?);
        }
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

    fn alter(self, item: &Self::SqlNode, new: Self) -> anyhow::Result<Vec<String>> {
        assert_eq!(self.id, new.id);
        let mut migrations = vec![];
        let mut commands = vec![];

        if self.type_name != new.type_name {
            commands.push(format!(
                "ALTER COLUMN {} TYPE {}",
                new.id.name, new.type_name
            ));
        }

        if self.nullable != new.nullable {
            let nullable = format!(
                "ALTER COLUMN {} {}",
                new.id.name,
                if new.nullable {
                    "DROP NOT NULL"
                } else {
                    "SET NOT NULL"
                }
            );
            commands.push(nullable);
        }

        if self.default != new.default {
            let default = format!(
                "ALTER COLUMN {} {}",
                new.id.name,
                if let Some(v) = new.default {
                    format!("SET {}", v.generate_sql()?)
                } else {
                    "DROP DEFAULT".to_string()
                }
            );
            commands.push(default);
        }

        if !commands.is_empty() {
            let sql = format!("ALTER TABLE {} {}", item.id, commands.join(", "));
            migrations.push(sql);
        }

        Ok(migrations)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Differ, MigrationPlanner};

    use super::*;

    #[test]
    fn table_add_column_with_default_function_should_work() {
        let s1 = "CREATE TABLE foo (name text)";
        let s2 = "CREATE TABLE foo (name text default random_name())";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            "ALTER TABLE public.foo ALTER COLUMN name SET DEFAULT random_name()"
        );
    }

    #[test]
    fn table_add_column_with_default_value_should_work() {
        let s1 = "CREATE TABLE foo (name text)";
        let s2 = "CREATE TABLE foo (name text default '')";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            "ALTER TABLE public.foo ALTER COLUMN name SET DEFAULT ''"
        );
    }

    #[test]
    fn table_change_column_type_should_work() {
        let s1 = "CREATE TABLE foo (name varchar(128))";
        let s2 = "CREATE TABLE foo (name varchar(256))";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            "ALTER TABLE public.foo ALTER COLUMN name TYPE pg_catalog.varchar(256)"
        );
    }

    #[test]
    fn table_change_column_array_type_should_work() {
        let s1 = "CREATE TABLE foo (name text[][4])";
        let s2 = "CREATE TABLE foo (name varchar(256)[][5])";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            "ALTER TABLE public.foo ALTER COLUMN name TYPE pg_catalog.varchar(256)[][5]"
        );
    }

    #[test]
    fn table_add_column_array_type_should_work() {
        let s1 = "CREATE TABLE foo (name varchar(256))";
        let s2 = "CREATE TABLE foo (name varchar(256), tags text [])";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0], "ALTER TABLE public.foo ADD COLUMN tags text[]");
    }
}
