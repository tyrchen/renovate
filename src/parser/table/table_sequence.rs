use crate::{
    parser::{AlterTable, AlterTableAction, RelationId, SchemaId, SequenceInfo, TableSequence},
    NodeItem,
};
use pg_query::{protobuf::AlterTableStmt, NodeEnum, NodeRef};

impl NodeItem for TableSequence {
    type Inner = AlterTableStmt;
    fn id(&self) -> String {
        self.id.name.clone()
    }

    fn type_name(&self) -> &'static str {
        "table sequence"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match self.node() {
            NodeEnum::AlterTableStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create index statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!(
            "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT",
            self.id.schema_id, self.id.name
        );
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::AlterTableStmt(stmt) => Ok(NodeEnum::AlterTableStmt(stmt.clone())),
            _ => anyhow::bail!("not a alter table statement"),
        }
    }
}

impl TryFrom<AlterTable> for TableSequence {
    type Error = anyhow::Error;
    fn try_from(AlterTable { id, action, node }: AlterTable) -> Result<Self, Self::Error> {
        match action {
            AlterTableAction::Sequence(info) => Ok(TableSequence::new(id, *info, node)),
            _ => anyhow::bail!("not an add constraint"),
        }
    }
}

impl TableSequence {
    fn new(id: SchemaId, info: SequenceInfo, node: NodeEnum) -> Self {
        let id = RelationId::new_with(id, info.column);
        Self { id, node }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Differ, MigrationPlanner};

    #[test]
    fn alter_table_set_default_sequence_should_parse() {
        let sql = "ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass)";
        let parsed: TableSequence = sql.parse().unwrap();
        assert_eq!(parsed.id.schema_id.to_string(), "public.users");
        assert_eq!(parsed.id.name, "id");
    }

    #[test]
    fn alter_table_set_default_sequence_should_revert() {
        let sql = "ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass)";
        let parsed: TableSequence = sql.parse().unwrap();
        let reverted = parsed.revert().unwrap().deparse().unwrap();
        assert_eq!(
            reverted,
            "ALTER TABLE public.users ALTER COLUMN id DROP DEFAULT"
        );
    }

    #[test]
    fn alter_table_set_default_sequence_migration_should_drop_and_create() {
        let sql1 = "ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass)";
        let sql2 = "ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq1'::regclass)";
        let old: TableSequence = sql1.parse().unwrap();
        let new: TableSequence = sql2.parse().unwrap();
        let diff = Differ::diff(&old, &new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(
            plan[0],
            "ALTER TABLE public.users ALTER COLUMN id DROP DEFAULT"
        );
        assert_eq!(
            plan[1],
            "ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq1'::regclass)"
        );
    }
}
