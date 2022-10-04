use crate::parser::{
    AlterTable, AlterTableAction, ConstraintInfo, RelationId, SchemaId, TableConstraint,
};
use pg_query::{
    protobuf::{ConstrType, Constraint as PgConstraint},
    NodeEnum, NodeRef,
};
use std::str::FromStr;

impl FromStr for TableConstraint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parsed = pg_query::parse(s)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::AlterTableStmt(stmt) => AlterTable::try_from(stmt)?.try_into(),
            _ => anyhow::bail!("not a constraint: {}", s),
        }
    }
}

impl TryFrom<AlterTable> for TableConstraint {
    type Error = anyhow::Error;
    fn try_from(AlterTable { id, action, node }: AlterTable) -> Result<Self, Self::Error> {
        match action {
            AlterTableAction::Constraint(info) => Ok(TableConstraint::new(id, *info, node)),
            _ => anyhow::bail!("not an add constraint"),
        }
    }
}

impl TableConstraint {
    fn new(id: SchemaId, info: ConstraintInfo, node: NodeEnum) -> Self {
        let id = RelationId::new_with(id, info.name.clone());
        Self { id, info, node }
    }
}

impl TryFrom<&PgConstraint> for ConstraintInfo {
    type Error = anyhow::Error;
    fn try_from(constraint: &PgConstraint) -> Result<Self, Self::Error> {
        let con_type = ConstrType::from_i32(constraint.contype).unwrap();
        let node = NodeEnum::Constraint(Box::new(constraint.clone()));
        let name = constraint.conname.clone();
        Ok(Self {
            name,
            con_type,
            node,
        })
    }
}
