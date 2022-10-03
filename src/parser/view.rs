use super::{SchemaId, View};
use debug_ignore::DebugIgnore;
use pg_query::{
    protobuf::{CreateTableAsStmt, ViewStmt},
    NodeEnum,
};

impl TryFrom<&ViewStmt> for View {
    type Error = anyhow::Error;
    fn try_from(stmt: &ViewStmt) -> Result<Self, Self::Error> {
        let id = get_id(stmt);
        let node = NodeEnum::ViewStmt(Box::new(stmt.clone()));
        Ok(Self {
            id,
            node: DebugIgnore(node),
        })
    }
}

impl From<&CreateTableAsStmt> for View {
    fn from(_stmt: &CreateTableAsStmt) -> Self {
        todo!()
    }
}

fn get_id(stmt: &ViewStmt) -> SchemaId {
    assert!(stmt.view.is_some());
    stmt.view.as_ref().unwrap().into()
}
