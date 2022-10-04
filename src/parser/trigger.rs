use super::{RelationId, Trigger};
use crate::{DiffItem, MigrationPlanner, NodeDiff};
use anyhow::Context;
use pg_query::{protobuf::CreateTrigStmt, NodeEnum, NodeRef};
use std::str::FromStr;

impl DiffItem for Trigger {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }
}

impl FromStr for Trigger {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parsed =
            pg_query::parse(s).with_context(|| format!("Failed to parse trigger: {}", s))?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::CreateTrigStmt(stmt) => Self::try_from(stmt),
            _ => anyhow::bail!("not a trigger: {}", s),
        }
    }
}

impl TryFrom<&CreateTrigStmt> for Trigger {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateTrigStmt) -> Result<Self, Self::Error> {
        let id = stmt.trigname.clone();
        let node = NodeEnum::CreateTrigStmt(Box::new(stmt.clone()));
        Ok(Self { id, node })
    }
}

impl MigrationPlanner for NodeDiff<Trigger> {
    type Migration = String;

    fn drop(&self) -> anyhow::Result<Option<Self::Migration>> {
        if let Some(old) = &self.old {
            Ok(Some(format!("DROP TRIGGER {};", old.id)))
        } else {
            Ok(None)
        }
    }

    fn create(&self) -> anyhow::Result<Option<Self::Migration>> {
        if let Some(new) = &self.new {
            Ok(Some(format!("{};", new.node.deparse()?)))
        } else {
            Ok(None)
        }
    }

    fn alter(&self) -> anyhow::Result<Option<Vec<Self::Migration>>> {
        Ok(None)
    }
}

#[allow(dead_code)]
fn get_id(stmt: &CreateTrigStmt) -> RelationId {
    let name = stmt.trigname.clone();
    assert!(stmt.relation.is_some());
    let schema_id = stmt.relation.as_ref().unwrap().into();

    RelationId::new_with(schema_id, name)
}

#[cfg(test)]
mod tests {
    use crate::Differ;

    use super::*;

    #[test]
    fn trigger_should_parse() {
        let sql = "CREATE TRIGGER test_trigger BEFORE INSERT ON test_table FOR EACH ROW EXECUTE FUNCTION test_function();";
        let trigger: Trigger = sql.parse().unwrap();
        assert_eq!(trigger.id, "test_trigger");
    }

    #[test]
    fn trigger_diff_should_work() {
        let sql1 = "CREATE TRIGGER test_trigger BEFORE INSERT ON test_table FOR EACH ROW EXECUTE FUNCTION test_function();";
        let sql2 = "CREATE TRIGGER test_trigger AFTER INSERT ON test_table FOR EACH ROW EXECUTE FUNCTION test_function();";
        let trigger1: Trigger = sql1.parse().unwrap();
        let trigger2: Trigger = sql2.parse().unwrap();
        let diff = trigger1.diff(&trigger2).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "DROP TRIGGER test_trigger;");
        assert_eq!(plan[1], sql2);
    }
}
