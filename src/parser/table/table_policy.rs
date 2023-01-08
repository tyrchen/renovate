use crate::{
    parser::{utils::node_to_string, RelationId, TablePolicy},
    NodeItem,
};
use pg_query::{protobuf::CreatePolicyStmt, NodeEnum, NodeRef};

impl NodeItem for TablePolicy {
    type Inner = CreatePolicyStmt;
    fn id(&self) -> String {
        self.id.name.clone()
    }

    fn type_name(&self) -> &'static str {
        "policy"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::CreatePolicyStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create policy statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP POLICY {} On {}", self.id.name, self.id.schema_id);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop index statement"),
        }
    }
}

impl TryFrom<&CreatePolicyStmt> for TablePolicy {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreatePolicyStmt) -> Result<Self, Self::Error> {
        let id = get_id(stmt);
        let cmd_name = stmt.cmd_name.clone();
        let permissive = stmt.permissive;
        let roles = stmt.roles.iter().filter_map(node_to_string).collect();
        let qual = stmt.qual.as_deref().and_then(node_to_string);
        let with_check = stmt.with_check.as_deref().and_then(node_to_string);
        let node = NodeEnum::CreatePolicyStmt(Box::new(stmt.clone()));
        Ok(Self {
            id,
            cmd_name,
            permissive,
            roles,
            qual,
            with_check,
            node,
        })
    }
}

fn get_id(stmt: &CreatePolicyStmt) -> RelationId {
    let name = stmt.policy_name.clone();
    assert!(stmt.table.is_some());
    let schema_id = stmt.table.as_ref().unwrap().into();
    RelationId { name, schema_id }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn policy_should_parse() {
        let sql = "CREATE POLICY baz ON foo.bar FOR ALL USING(username = CURRENT_USER) WITH CHECK (username = CURRENT_USER)";
        let p: TablePolicy = sql.parse().unwrap();
        assert_eq!(p.id.name, "baz");
        assert_eq!(p.id.schema_id.schema, "foo");
        assert_eq!(p.id.schema_id.name, "bar");
    }

    #[test]
    fn unchanged_policy_should_return_none() {
        let sql1 = "CREATE POLICY foo ON bar FOR ALL TO postgres USING(true)";
        let sql2 = "CREATE POLICY foo ON bar FOR ALL TO postgres USING(true)";
        let old: TablePolicy = sql1.parse().unwrap();
        let new: TablePolicy = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap();
        assert!(diff.is_none());
    }

    #[test]
    fn changed_policy_should_generate_migration() {
        let sql1 = "CREATE POLICY foo ON bar FOR ALL TO postgres USING(true)";
        let sql2 = "CREATE POLICY foo ON bar FOR SELECT TO postgres USING(true)";
        let old: TablePolicy = sql1.parse().unwrap();
        let new: TablePolicy = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let migrations = diff.plan().unwrap();
        assert_eq!(migrations[0], "DROP POLICY foo ON public.bar");
        assert_eq!(
            migrations[1],
            "CREATE POLICY foo ON bar FOR SELECT TO postgres USING (true) "
        );
    }
}
