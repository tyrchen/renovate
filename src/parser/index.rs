use super::Index;
use pg_query::protobuf::IndexStmt;

impl From<&IndexStmt> for Index {
    fn from(_stmt: &IndexStmt) -> Self {
        todo!()
    }
}
