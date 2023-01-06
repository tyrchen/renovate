use super::{utils::get_type_name, EnumType};
use crate::NodeItem;
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
        let id = get_type_name(&stmt.type_name).parse()?;
        let node = NodeEnum::CreateEnumStmt(stmt.clone());
        Ok(Self { id, node })
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
        let sql2 = "CREATE TYPE enum_type AS ENUM ('a', 'b', 'c', 'd')";
        let old: EnumType = sql1.parse().unwrap();
        let new: EnumType = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "DROP TYPE public.enum_type");
        assert_eq!(
            plan[1],
            "CREATE TYPE enum_type AS ENUM ('a', 'b', 'c', 'd')"
        );
    }
}
