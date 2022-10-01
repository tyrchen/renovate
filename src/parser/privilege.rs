use super::Privilege;
use pg_query::protobuf::GrantStmt;

impl From<&GrantStmt> for Privilege {
    fn from(_stmt: &GrantStmt) -> Self {
        todo!()
    }
}
