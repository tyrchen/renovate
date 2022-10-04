use super::{RelationId, TableIndex};
use crate::{DiffItem, MigrationPlanner, MigrationResult, NodeDiff};
use anyhow::Context;
use pg_query::{protobuf::IndexStmt, NodeEnum, NodeRef};
use std::str::FromStr;

impl DiffItem for TableIndex {
    fn id(&self) -> String {
        self.id.name.clone()
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }
}

impl FromStr for TableIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parsed = pg_query::parse(s).with_context(|| format!("Failed to parse index: {}", s))?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::IndexStmt(stmt) => Self::try_from(stmt),
            _ => anyhow::bail!("not an index: {}", s),
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

impl MigrationPlanner for NodeDiff<TableIndex> {
    type Migration = String;

    fn drop(&self) -> MigrationResult<Self::Migration> {
        if let Some(old) = &self.old {
            let sql = format!("DROP INDEX {};", old.id.name);
            Ok(vec![sql])
        } else {
            Ok(vec![])
        }
    }

    fn create(&self) -> MigrationResult<Self::Migration> {
        if let Some(new) = &self.new {
            let sql = format!("{};", new.node.deparse()?);
            Ok(vec![sql])
        } else {
            Ok(vec![])
        }
    }

    fn alter(&self) -> MigrationResult<Self::Migration> {
        Ok(vec![])
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
    use crate::Differ;

    use super::*;

    #[test]
    fn index_should_parse() {
        let sql = "CREATE INDEX foo ON bar (baz);";
        let index: TableIndex = sql.parse().unwrap();
        assert_eq!(index.id.name, "foo");
        assert_eq!(index.id.schema_id.schema, "public");
        assert_eq!(index.id.schema_id.name, "bar");
    }

    #[test]
    fn unchanged_index_should_return_none() {
        let sql1 = "CREATE INDEX foo ON bar (baz);";
        let sql2 = "CREATE INDEX foo ON bar (baz);";
        let old: TableIndex = sql1.parse().unwrap();
        let new: TableIndex = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap();
        assert!(diff.is_none());
    }

    #[test]
    fn changed_index_should_generate_migration() {
        let sql1 = "CREATE INDEX foo ON bar (baz);";
        let sql2 = "CREATE INDEX foo ON bar (ooo);";
        let old: TableIndex = sql1.parse().unwrap();
        let new: TableIndex = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let migrations = diff.plan().unwrap();
        assert_eq!(migrations[0], "DROP INDEX foo;");
        assert_eq!(migrations[1], "CREATE INDEX foo ON bar USING btree (ooo);");
    }
}
