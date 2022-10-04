use crate::parser::{AlterTable, AlterTableAction, SchemaId, TableOwner};
use pg_query::NodeEnum;

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
