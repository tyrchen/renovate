use super::Table;
use pg_query::protobuf::CreateStmt;

impl From<&CreateStmt> for Table {
    fn from(_stmt: &CreateStmt) -> Self {
        todo!()
    }
}
