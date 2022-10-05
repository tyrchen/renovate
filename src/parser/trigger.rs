use super::{RelationId, Trigger};
use crate::{MigrationPlanner, MigrationResult, NodeDiff, NodeItem};
use anyhow::Context;
use pg_query::{protobuf::CreateTrigStmt, NodeEnum, NodeRef};
use std::str::FromStr;

impl NodeItem for Trigger {
    type Inner = CreateTrigStmt;

    fn id(&self) -> String {
        self.id.name.clone()
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::CreateTrigStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create trigger statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP TRIGGER {} on {}", self.id.name, self.id.schema_id);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop index statement"),
        }
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
        let name = stmt.trigname.clone();
        let schema_id = stmt.relation.as_ref().into();
        let id = RelationId::new_with(schema_id, name);
        let node = NodeEnum::CreateTrigStmt(Box::new(stmt.clone()));
        Ok(Self { id, node })
    }
}

impl MigrationPlanner for NodeDiff<Trigger> {
    type Migration = String;

    fn drop(&self) -> MigrationResult<Self::Migration> {
        if let Some(old) = &self.old {
            let sql = old.revert()?.deparse()?;
            Ok(vec![sql])
        } else {
            Ok(vec![])
        }
    }

    fn create(&self) -> MigrationResult<Self::Migration> {
        if let Some(new) = &self.new {
            let sql = new.node.deparse()?;
            Ok(vec![sql])
        } else {
            Ok(vec![])
        }
    }

    fn alter(&self) -> MigrationResult<Self::Migration> {
        Ok(vec![])
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
        let sql = "CREATE TRIGGER test_trigger BEFORE INSERT ON test_table FOR EACH ROW EXECUTE FUNCTION test_function()";
        let trigger: Trigger = sql.parse().unwrap();
        assert_eq!(trigger.id.name, "test_trigger");
        assert_eq!(trigger.id.schema_id.to_string(), "public.test_table");
    }

    #[test]
    fn trigger_diff_should_work() {
        let sql1 = "CREATE TRIGGER test_trigger BEFORE INSERT ON test_table FOR EACH ROW EXECUTE FUNCTION test_function()";
        let sql2 = "CREATE TRIGGER test_trigger AFTER INSERT ON test_table FOR EACH ROW EXECUTE FUNCTION test_function()";
        let trigger1: Trigger = sql1.parse().unwrap();
        let trigger2: Trigger = sql2.parse().unwrap();
        let diff = trigger1.diff(&trigger2).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "DROP TRIGGER test_trigger ON public.test_table");
        assert_eq!(plan[1], sql2);
    }
}
