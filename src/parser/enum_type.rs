use super::{utils::node_to_string, EnumType};
use crate::{MigrationPlanner, MigrationResult, NodeDiff, NodeItem};
use itertools::Itertools;
use pg_query::{protobuf::CreateEnumStmt, NodeEnum, NodeRef};

impl NodeItem for EnumType {
    type Inner = CreateEnumStmt;
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn type_name(&self) -> &'static str {
        "enum"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::CreateEnumStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create view statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP TYPE {}", self.id);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop type statement"),
        }
    }
}

impl TryFrom<&CreateEnumStmt> for EnumType {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateEnumStmt) -> Result<Self, Self::Error> {
        let id = stmt
            .type_name
            .iter()
            .filter_map(node_to_string)
            .join(".")
            .parse()?;
        let node = NodeEnum::CreateEnumStmt(stmt.clone());
        let items = stmt.vals.iter().filter_map(node_to_string).collect();
        Ok(Self { id, items, node })
    }
}

impl MigrationPlanner for NodeDiff<EnumType> {
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
                let added = new.items.difference(&old.items).collect::<Vec<_>>();
                let removed = old.items.difference(&new.items).collect::<Vec<_>>();
                if removed.is_empty() {
                    let migrations = added
                        .iter()
                        .map(|s| format!("ALTER TYPE {} ADD VALUE '{}'", old.id, s))
                        .collect();
                    return Ok(migrations);
                }

                if removed.len() == added.len() && removed.len() == 1 {
                    let sql = format!(
                        "ALTER TYPE {} RENAME VALUE '{}' TO '{}'",
                        old.id,
                        removed.get(0).unwrap(),
                        added.get(0).unwrap()
                    );
                    return Ok(vec![sql]);
                }

                if atty::is(atty::Stream::Stdout) {
                    println!("WARNING: recreate enum type {} because of incompatible changes. Be CAUTIOUS this migration might fail if you referenced the type in other places.", old.id);
                }
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn enum_type_should_parse() {
        let sql = "CREATE TYPE enum_type AS ENUM ('a', 'b', 'c')";
        let enum_type: EnumType = sql.parse().unwrap();
        assert_eq!(enum_type.id.to_string(), "public.enum_type");
    }

    #[test]
    fn composite_type_should_generate_drop_create_plan() {
        let sql1 = "CREATE TYPE enum_type AS ENUM ('a', 'b', 'c')";
        let sql2 = "CREATE TYPE enum_type AS ENUM ('a', 'b', 'c', 'd', 'e')";
        let old: EnumType = sql1.parse().unwrap();
        let new: EnumType = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "ALTER TYPE public.enum_type ADD VALUE 'd'");
        assert_eq!(plan[1], "ALTER TYPE public.enum_type ADD VALUE 'e'");
    }
}
