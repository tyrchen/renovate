use super::View;
use pg_query::protobuf::{CreateTableAsStmt, ViewStmt};

impl From<&ViewStmt> for View {
    fn from(_stmt: &ViewStmt) -> Self {
        todo!()
    }
}

impl From<&CreateTableAsStmt> for View {
    fn from(_stmt: &CreateTableAsStmt) -> Self {
        todo!()
    }
}
