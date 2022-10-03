use std::str::FromStr;

use super::{AlterTable, AlterTableAction, ConstraintInfo, RelationId, SchemaId, TableConstraint};
use debug_ignore::DebugIgnore;
use pg_query::{NodeEnum, NodeRef};

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
    fn new(id: SchemaId, info: ConstraintInfo, node: DebugIgnore<NodeEnum>) -> Self {
        let id = RelationId::new_with(id, info.name.clone());
        Self { id, info, node }
    }
}

// impl TryFrom<(SchemaId, &AlterTableStmt, &PgConstraint)> for Constraint {
//     type Error = anyhow::Error;
//     fn try_from(
//         (id, alter, constraint): (SchemaId, &AlterTableStmt, &PgConstraint),
//     ) -> Result<Self, Self::Error> {
//         let name = constraint.conname.clone();
//         let con_type = ConstrType::from_i32(constraint.contype).unwrap();
//         let id = RelationId::new_with(id, name);
//         let node = NodeEnum::AlterTableStmt(alter.clone());
//         Ok(Self {
//             id,
//             con_type,
//             node: DebugIgnore(node),
//         })
//     }
// }
