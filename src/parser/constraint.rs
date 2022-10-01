use super::Constraint;
use pg_query::protobuf::AlterTableStmt;

impl TryFrom<&AlterTableStmt> for Constraint {
    type Error = anyhow::Error;
    fn try_from(_stmt: &AlterTableStmt) -> Result<Self, Self::Error> {
        todo!()
    }
}
