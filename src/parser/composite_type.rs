use super::CompositeType;
use crate::NodeItem;
use pg_query::{protobuf::CompositeTypeStmt, NodeEnum, NodeRef};

impl NodeItem for CompositeType {
    type Inner = CompositeTypeStmt;
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::CompositeTypeStmt(stmt) => Ok(stmt),
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

impl TryFrom<&CompositeTypeStmt> for CompositeType {
    type Error = anyhow::Error;
    fn try_from(stmt: &CompositeTypeStmt) -> Result<Self, Self::Error> {
        let id = stmt.typevar.as_ref().into();
        let node = NodeEnum::CompositeTypeStmt(stmt.clone());
        Ok(Self { id, node })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn composite_type_should_parse() {
        let sql = "CREATE TYPE foo AS (a int, b text)";
        let composite_type: CompositeType = sql.parse().unwrap();
        assert_eq!(composite_type.id.to_string(), "public.foo");
    }

    #[test]
    fn composite_type_should_generate_drop_create_plan() {
        let sql1 = "CREATE TYPE foo AS (a int, b text)";
        let sql2 = "CREATE TYPE foo AS (a int, b text, c text)";
        let old: CompositeType = sql1.parse().unwrap();
        let new: CompositeType = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].to_string(), "DROP TYPE public.foo");
        assert_eq!(
            plan[1].to_string(),
            "CREATE TYPE foo AS (a int, b text, c text)"
        );
    }
}
