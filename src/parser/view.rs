use super::{SchemaId, View};
use crate::NodeItem;
use pg_query::{protobuf::ViewStmt, NodeEnum, NodeRef};

impl NodeItem for View {
    type Inner = ViewStmt;
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn type_name(&self) -> &'static str {
        "view"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::ViewStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create view statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP VIEW {}", self.id);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop index statement"),
        }
    }
}

impl TryFrom<&ViewStmt> for View {
    type Error = anyhow::Error;
    fn try_from(stmt: &ViewStmt) -> Result<Self, Self::Error> {
        let id = get_view_id(stmt);
        let node = NodeEnum::ViewStmt(Box::new(stmt.clone()));
        Ok(Self { id, node })
    }
}

fn get_view_id(stmt: &ViewStmt) -> SchemaId {
    assert!(stmt.view.is_some());
    stmt.view.as_ref().unwrap().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn view_should_parse() {
        let sql = "CREATE VIEW foo AS SELECT 1";
        let view: View = sql.parse().unwrap();
        assert_eq!(view.id.to_string(), "public.foo");
    }

    #[test]
    fn test_view_migration() {
        let sql1 = "CREATE VIEW foo AS SELECT 1";
        let sql2 = "CREATE VIEW foo AS SELECT 2";
        let old: View = sql1.parse().unwrap();
        let new: View = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let migrations = diff.plan().unwrap();
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0], "DROP VIEW public.foo");
        assert_eq!(migrations[1], "CREATE VIEW foo AS SELECT 2");
    }
}
