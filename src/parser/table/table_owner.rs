use crate::{
    parser::{AlterTable, AlterTableAction, SchemaId, TableOwner},
    NodeItem,
};
use pg_query::{protobuf::AlterTableStmt, NodeEnum, NodeRef};

impl NodeItem for TableOwner {
    type Inner = AlterTableStmt;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn type_name(&self) -> &'static str {
        "table owner"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::AlterTableStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a alter table statement"),
        }
    }

    /// we don't know what the old owner is, so we can only revert to session_user
    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("ALTER TABLE {} OWNER TO session_user", self.id);
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::AlterTableStmt(stmt) => Ok(NodeEnum::AlterTableStmt(stmt.clone())),
            _ => anyhow::bail!("not a alter table owner to statement"),
        }
    }
}

impl TryFrom<AlterTable> for TableOwner {
    type Error = anyhow::Error;
    fn try_from(AlterTable { id, action, node }: AlterTable) -> Result<Self, Self::Error> {
        match action {
            AlterTableAction::Owner(owner) => Ok(TableOwner::new(id, owner, node)),
            _ => anyhow::bail!("not an owner change"),
        }
    }
}

impl TableOwner {
    fn new(id: SchemaId, owner: String, node: NodeEnum) -> Self {
        Self { id, owner, node }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn table_owner_to_should_parse() {
        let sql = "ALTER TABLE foo OWNER TO bar";
        let parsed: TableOwner = sql.parse().unwrap();
        assert_eq!(parsed.id.name, "foo");
        assert_eq!(parsed.owner, "bar");
    }

    #[test]
    fn table_owner_to_should_revert() {
        let sql = "ALTER TABLE foo OWNER TO bar";
        let parsed: TableOwner = sql.parse().unwrap();
        let reverted = parsed.revert().unwrap().deparse().unwrap();
        assert_eq!(reverted, "ALTER TABLE public.foo OWNER TO SESSION_USER");
    }

    #[test]
    fn table_owner_to_should_generate_drop_create_migration() {
        let sql1 = "ALTER TABLE foo OWNER TO bar";
        let sql2 = "ALTER TABLE foo OWNER TO baz";
        let old: TableOwner = sql1.parse().unwrap();
        let new: TableOwner = sql2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "ALTER TABLE public.foo OWNER TO SESSION_USER");
        assert_eq!(plan[1], sql2);
    }
}
