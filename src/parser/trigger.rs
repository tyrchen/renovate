use super::Trigger;
use pg_query::protobuf::CreateTrigStmt;

impl From<&CreateTrigStmt> for Trigger {
    fn from(_stmt: &CreateTrigStmt) -> Self {
        todo!()
    }
}
