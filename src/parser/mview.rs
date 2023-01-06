use super::{MatView, SchemaId};
use crate::NodeItem;
use pg_query::{protobuf::CreateTableAsStmt, NodeEnum, NodeRef};

impl NodeItem for MatView {
    type Inner = CreateTableAsStmt;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn type_name(&self) -> &'static str {
        "materialized view"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::CreateTableAsStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create trigger statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP MATERIALIZED VIEW {}", self.id);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop index statement"),
        }
    }
}

impl TryFrom<&CreateTableAsStmt> for MatView {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateTableAsStmt) -> Result<Self, Self::Error> {
        let id = get_mview_id(stmt);
        let node = NodeEnum::CreateTableAsStmt(Box::new(stmt.clone()));
        Ok(Self { id, node })
    }
}

fn get_mview_id(stmt: &CreateTableAsStmt) -> SchemaId {
    assert!(stmt.into.is_some());
    let into = stmt.into.as_ref().unwrap();
    assert!(into.rel.is_some());
    into.rel.as_ref().unwrap().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn mview_should_parse() {
        let sql = "CREATE MATERIALIZED VIEW foo.bar AS SELECT 1";
        let view: MatView = sql.parse().unwrap();
        assert_eq!(view.id.to_string(), "foo.bar");
    }

    #[test]
    fn test_mview_migration() {
        let sql1 = "CREATE MATERIALIZED VIEW foo AS SELECT 1";
        let sql2 = "CREATE MATERIALIZED VIEW foo AS SELECT 2";
        let old: MatView = sql1.parse().unwrap();
        let new: MatView = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let migrations = diff.plan().unwrap();
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0], "DROP MATERIALIZED VIEW public.foo");
        assert_eq!(migrations[1], "CREATE MATERIALIZED VIEW foo AS SELECT 2");
    }
}
