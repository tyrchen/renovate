use super::{Constraint, EmbedConstraint, RelationId, SchemaId};
use debug_ignore::DebugIgnore;
use pg_query::{
    protobuf::{AlterTableStmt, ConstrType, Constraint as PgConstraint},
    NodeEnum,
};

impl TryFrom<(SchemaId, &AlterTableStmt, &PgConstraint)> for Constraint {
    type Error = anyhow::Error;
    fn try_from(
        (id, alter, constraint): (SchemaId, &AlterTableStmt, &PgConstraint),
    ) -> Result<Self, Self::Error> {
        let name = constraint.conname.clone();
        let con_type = ConstrType::from_i32(constraint.contype).unwrap();
        let id = RelationId::new_with(id, name);
        let node = NodeEnum::AlterTableStmt(alter.clone());
        Ok(Self {
            id,
            con_type,
            node: DebugIgnore(node),
        })
    }
}

impl TryFrom<&PgConstraint> for EmbedConstraint {
    type Error = anyhow::Error;
    fn try_from(constraint: &PgConstraint) -> Result<Self, Self::Error> {
        let con_type = ConstrType::from_i32(constraint.contype).unwrap();
        let node = NodeEnum::Constraint(Box::new(constraint.clone()));
        Ok(Self {
            con_type,
            node: DebugIgnore(node),
        })
    }
}
