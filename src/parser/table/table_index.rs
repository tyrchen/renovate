use crate::{
    parser::{RelationId, TableIndex},
    NodeItem,
};
use pg_query::{protobuf::IndexStmt, NodeEnum, NodeRef};

impl NodeItem for TableIndex {
    type Inner = IndexStmt;
    fn id(&self) -> String {
        self.id.name.clone()
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::IndexStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create index statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP INDEX {}", self.id.name);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop index statement"),
        }
    }
}

impl TryFrom<&IndexStmt> for TableIndex {
    type Error = anyhow::Error;
    fn try_from(stmt: &IndexStmt) -> Result<Self, Self::Error> {
        let id = get_id(stmt);
        let node = pg_query::NodeEnum::IndexStmt(Box::new(stmt.clone()));
        Ok(Self { id, node })
    }
}

fn get_id(stmt: &IndexStmt) -> RelationId {
    let name = stmt.idxname.clone();
    assert!(stmt.relation.is_some());
    let schema_id = stmt.relation.as_ref().unwrap().into();
    RelationId { name, schema_id }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn index_should_parse() {
        let sql = "CREATE INDEX foo ON bar (baz)";
        let index: TableIndex = sql.parse().unwrap();
        assert_eq!(index.id.name, "foo");
        assert_eq!(index.id.schema_id.schema, "public");
        assert_eq!(index.id.schema_id.name, "bar");
    }

    #[test]
    fn unchanged_index_should_return_none() {
        let sql1 = "CREATE INDEX foo ON bar (baz)";
        let sql2 = "CREATE INDEX foo ON bar (baz)";
        let old: TableIndex = sql1.parse().unwrap();
        let new: TableIndex = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap();
        assert!(diff.is_none());
    }

    #[test]
    fn changed_index_should_generate_migration() {
        let sql1 = "CREATE INDEX foo ON bar (baz)";
        let sql2 = "CREATE INDEX foo ON bar (ooo)";
        let old: TableIndex = sql1.parse().unwrap();
        let new: TableIndex = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let migrations = diff.plan().unwrap();
        assert_eq!(migrations[0], "DROP INDEX foo");
        assert_eq!(migrations[1], "CREATE INDEX foo ON bar USING btree (ooo)");
    }
}
