use super::Function;
use pg_query::protobuf::CreateFunctionStmt;

impl From<&CreateFunctionStmt> for Function {
    fn from(_stmt: &CreateFunctionStmt) -> Self {
        todo!()
    }
}
