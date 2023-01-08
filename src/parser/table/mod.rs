mod alter_table;
mod column;
mod table_constraint;
mod table_index;
mod table_owner;
mod table_policy;
mod table_rls;
mod table_sequence;
mod table_trigger;

use super::{Column, ConstraintInfo, SchemaId, Table};
use crate::{MigrationPlanner, MigrationResult, NodeDelta, NodeDiff, NodeItem};
use indexmap::IndexMap;
use pg_query::{protobuf::CreateStmt, NodeEnum, NodeRef};

impl NodeItem for Table {
    type Inner = CreateStmt;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn type_name(&self) -> &'static str {
        "table"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::CreateStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create table statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP TABLE {}", self.id);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop statement"),
        }
    }
}

impl TryFrom<&CreateStmt> for Table {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateStmt) -> Result<Self, Self::Error> {
        let id = SchemaId::from(stmt.relation.as_ref());
        let (columns, constraints) = parse_nodes(id.clone(), stmt)?;
        let node = NodeEnum::CreateStmt(stmt.clone());
        Ok(Self {
            id,
            columns,
            constraints,
            node,
        })
    }
}

impl MigrationPlanner for NodeDiff<Table> {
    type Migration = String;

    fn drop(&self) -> MigrationResult<Self::Migration> {
        if let Some(old) = &self.old {
            let sqls = vec![old.revert()?.deparse()?];
            Ok(sqls)
        } else {
            Ok(vec![])
        }
    }

    fn create(&self) -> MigrationResult<Self::Migration> {
        if let Some(new) = &self.new {
            let sqls = vec![new.node.deparse()?];
            Ok(sqls)
        } else {
            Ok(vec![])
        }
    }

    fn alter(&self) -> MigrationResult<Self::Migration> {
        match (&self.old, &self.new) {
            (Some(old), Some(new)) => {
                let delta =
                    NodeDelta::create(old.columns.iter().collect(), new.columns.iter().collect());
                let mut migrations = delta.plan(old)?;
                let delta = NodeDelta::create(
                    old.constraints.iter().collect(),
                    new.constraints.iter().collect(),
                );
                migrations.extend(delta.plan(old)?);
                Ok(migrations)
            }
            _ => Ok(vec![]),
        }
    }
}

fn parse_nodes(
    id: SchemaId,
    stmt: &CreateStmt,
) -> anyhow::Result<(IndexMap<String, Column>, IndexMap<String, ConstraintInfo>)> {
    let mut columns = IndexMap::new();
    let mut constraints = IndexMap::new();

    for node in stmt.table_elts.iter().filter_map(|n| n.node.as_ref()) {
        match node {
            NodeEnum::ColumnDef(col) => {
                let column = Column::try_from((id.clone(), col.as_ref().clone()))?;
                columns.insert(column.id.name.to_string(), column);
            }
            NodeEnum::Constraint(constraint) => {
                let constraint = ConstraintInfo::try_from(constraint.as_ref())?;
                constraints.insert(constraint.name.clone(), constraint);
            }
            _ => {}
        }
    }
    Ok((columns, constraints))
}

#[cfg(test)]
mod tests {
    use pg_query::protobuf::ConstrType;

    use crate::Differ;

    use super::*;

    #[test]
    fn test_parse_and_to_string() {
        let sql = "CREATE TABLE foo (id int PRIMARY KEY, name text NOT NULL UNIQUE)";
        let table: Table = sql.parse().unwrap();
        let sql1 = table.node.deparse().unwrap();
        assert_eq!(sql, sql1);
    }

    #[test]
    fn table_should_be_parsed_correctly() {
        let sql =
            "CREATE TABLE foo (id serial not null primary key, name text default random_name(), CHECK (check_name(name)))";
        let table: Table = sql.parse().unwrap();
        assert_eq!(table.id.to_string(), "public.foo");
        assert_eq!(table.columns.len(), 2);
        let col = table.columns.get("id").unwrap();
        assert_eq!(col.id.name, "id");
        assert_eq!(col.type_name, "serial");

        assert_eq!(col.constraints.len(), 1);
        let constraints: Vec<_> = col.constraints.iter().collect();
        let cons = constraints.get(0).unwrap();
        assert_eq!(cons.con_type, ConstrType::ConstrPrimary);
        assert!(!col.nullable);

        let col = table.columns.get("name").unwrap();
        assert_eq!(col.id.name, "name");
        assert_eq!(col.type_name, "text");
        assert!(col.nullable);
        assert_eq!(col.constraints.len(), 0);

        let cons = col.default.as_ref().unwrap();
        assert_eq!(cons.con_type, ConstrType::ConstrDefault);

        let constraints: Vec<_> = table.constraints.iter().collect();
        assert_eq!(constraints.len(), 1);
        let (_, cons) = constraints.get(0).unwrap();
        assert_eq!(cons.con_type, ConstrType::ConstrCheck);
    }

    #[test]
    fn table_should_generate_valid_plan() {
        let s1 =
            "CREATE TABLE foo (id serial not null primary key, name text default random_name())";
        let s2 = "CREATE TABLE foo (id serial not null primary key, name text default random_name(), email text, constraint c1 CHECK (check_name(name)))";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "ALTER TABLE public.foo ADD COLUMN email text");
        assert_eq!(
            plan[1],
            "ALTER TABLE public.foo ADD CONSTRAINT c1 CHECK (check_name(name))"
        );
    }

    #[test]
    fn same_table_should_generate_valid_plan() {
        let s1 = "CREATE TABLE public.todos (title text, completed boolean)";
        let s2 = "CREATE TABLE public.todos (title text, completed boolean)";
        let old: Table = s1.parse().unwrap();
        let new: Table = s2.parse().unwrap();
        let diff = old.diff(&new).unwrap();
        assert!(diff.is_none());
    }
}
