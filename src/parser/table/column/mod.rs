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
use std::{collections::BTreeSet, fmt};

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
        let sql = format!("ALTER TABLE ONLY {} ADD COLUMN {}", self.id.schema_id, self);
        Ok(sql)
    }

    fn default_str(&self) -> Option<String> {
        self.default.as_ref().map(|v| v.to_string())
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

    fn rename(self, item: &Self::SqlNode, new: Self) -> anyhow::Result<Vec<String>> {
        if self.type_name == new.type_name
            && self.nullable == new.nullable
            && self.default == new.default
            && self.constraints == new.constraints
        {
            return Ok(vec![format!(
                "ALTER TABLE ONLY {} RENAME COLUMN {} TO {}",
                item.id, self.id.name, new.id.name
            )]);
        }
        Ok(vec![])
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
                    format!("SET {}", v)
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

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fragments = vec![self.id.name.clone(), self.type_name.clone()];
        if !self.nullable {
            fragments.push("NOT NULL".to_owned());
        }
        if let Some(default) = self.default_str() {
            fragments.push(default);
        }
        for constraint in &self.constraints {
            fragments.push(constraint.to_string());
        }

        write!(f, "{}", fragments.join(" "))
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
        assert_eq!(
            plan[0],
            "ALTER TABLE ONLY public.foo ADD COLUMN tags text[]"
        );
    }

    #[test]
    fn simple_table_rename_column_should_work() {
        let s1 = "CREATE TABLE foo (name varchar(256))";
        let s2 = "CREATE TABLE foo (name1 varchar(256))";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            "ALTER TABLE ONLY public.foo RENAME COLUMN name TO name1"
        );
    }

    #[test]
    fn table_rename_column_should_work() {
        let s1 = "CREATE TABLE public.todos (
            title text NOT NULL,
            completed boolean,
            id bigint DEFAULT nextval('public.todos_id_seq'::regclass) NOT NULL,
            CONSTRAINT todos_title_check1 CHECK (length(title) > 5)
        )";
        let s2 = "CREATE TABLE public.todos (
            title text NOT NULL,
            completed1 boolean,
            id bigint DEFAULT nextval('public.todos_id_seq'::regclass) NOT NULL,
            CONSTRAINT todos_title_check1 CHECK (length(title) > 5)
        )";
        let t1: Table = s1.parse().unwrap();
        let t2: Table = s2.parse().unwrap();
        let diff = t1.diff(&t2).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0],
            "ALTER TABLE ONLY public.todos RENAME COLUMN completed TO completed1"
        );
    }
}
