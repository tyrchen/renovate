use super::{AlterTable, AlterTableAction, SchemaId, TableRls};
use debug_ignore::DebugIgnore;
use pg_query::NodeEnum;

impl TryFrom<AlterTable> for TableRls {
    type Error = anyhow::Error;
    fn try_from(AlterTable { id, action, node }: AlterTable) -> Result<Self, Self::Error> {
        match action {
            AlterTableAction::Rls => Ok(TableRls::new(id, node)),
            _ => anyhow::bail!("not an owner change"),
        }
    }
}

impl TableRls {
    fn new(id: SchemaId, node: DebugIgnore<NodeEnum>) -> Self {
        Self { id, node }
    }
}