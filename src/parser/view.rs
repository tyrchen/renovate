use super::{SchemaId, View};
use crate::{DiffItem, MigrationPlanner, MigrationResult, NodeDiff};
use anyhow::Context;
use pg_query::{
    protobuf::{CreateTableAsStmt, ViewStmt},
    NodeEnum, NodeRef,
};
use std::str::FromStr;

impl DiffItem for View {
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }
}

impl FromStr for View {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parsed = pg_query::parse(s).with_context(|| format!("Failed to parse view: {}", s))?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::ViewStmt(stmt) => Self::try_from(stmt),
            NodeRef::CreateTableAsStmt(stmt) => Self::try_from(stmt),
            _ => anyhow::bail!("not a view: {}", s),
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

impl TryFrom<&CreateTableAsStmt> for View {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateTableAsStmt) -> Result<Self, Self::Error> {
        let id = get_mview_id(stmt);
        let node = NodeEnum::CreateTableAsStmt(Box::new(stmt.clone()));
        Ok(Self { id, node })
    }
}

impl MigrationPlanner for NodeDiff<View> {
    type Migration = String;

    fn drop(&self) -> MigrationResult<Self::Migration> {
        if let Some(old) = &self.old {
            match old.node() {
                NodeEnum::ViewStmt(_) => {
                    let sql = format!("DROP VIEW {};", old.id);
                    Ok(vec![sql])
                }
                NodeEnum::CreateTableAsStmt(_) => {
                    let sql = format!("DROP MATERIALIZED VIEW {};", old.id);
                    Ok(vec![sql])
                }
                _ => anyhow::bail!("not a view or materialized view"),
            }
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

fn get_view_id(stmt: &ViewStmt) -> SchemaId {
    assert!(stmt.view.is_some());
    stmt.view.as_ref().unwrap().into()
}

fn get_mview_id(stmt: &CreateTableAsStmt) -> SchemaId {
    assert!(stmt.into.is_some());
    let into = stmt.into.as_ref().unwrap();
    assert!(into.rel.is_some());
    into.rel.as_ref().unwrap().into()
}

#[cfg(test)]
mod tests {
    use crate::Differ;

    use super::*;

    #[test]
    fn view_should_parse() {
        let sql = "CREATE VIEW foo AS SELECT 1;";
        let view: View = sql.parse().unwrap();
        assert_eq!(view.id.to_string(), "public.foo");
    }

    #[test]
    fn mview_should_parse() {
        let sql = "CREATE MATERIALIZED VIEW foo.bar AS SELECT 1;";
        let view: View = sql.parse().unwrap();
        assert_eq!(view.id.to_string(), "foo.bar");
    }

    #[test]
    fn test_view_migration() {
        let sql1 = "CREATE VIEW foo AS SELECT 1;";
        let sql2 = "CREATE VIEW foo AS SELECT 2;";
        let old: View = sql1.parse().unwrap();
        let new: View = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let migrations = diff.plan().unwrap();
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0], "DROP VIEW public.foo;");
        assert_eq!(migrations[1], "CREATE VIEW foo AS SELECT 2;");
    }

    #[test]
    fn test_mview_migration() {
        let sql1 = "CREATE MATERIALIZED VIEW foo AS SELECT 1;";
        let sql2 = "CREATE MATERIALIZED VIEW foo AS SELECT 2;";
        let old: View = sql1.parse().unwrap();
        let new: View = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let migrations = diff.plan().unwrap();
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0], "DROP MATERIALIZED VIEW public.foo;");
        assert_eq!(migrations[1], "CREATE MATERIALIZED VIEW foo AS SELECT 2;");
    }
}
