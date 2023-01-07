use crate::parser::{AlterTable, AlterTableAction, SchemaId};
use crate::parser::{ConstraintInfo, SequenceInfo};
use anyhow::{anyhow, Context};
use pg_query::{
    protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType},
    NodeEnum,
};
use tracing::warn;

impl TryFrom<&AlterTableStmt> for AlterTable {
    type Error = anyhow::Error;
    fn try_from(alter: &AlterTableStmt) -> Result<Self, Self::Error> {
        let id = SchemaId::from(alter.relation.as_ref());
        let cmd = alter
            .cmds
            .iter()
            .filter_map(|n| n.node.as_ref())
            .next()
            .ok_or_else(|| anyhow!("no commands"))?;

        let action = match cmd {
            NodeEnum::AlterTableCmd(ref cmd) => AlterTableAction::try_from(cmd.as_ref())?,
            _ => anyhow::bail!("not an alter table command"),
        };

        let node = NodeEnum::AlterTableStmt(alter.clone());

        Ok(Self { id, action, node })
    }
}

impl TryFrom<&AlterTableCmd> for AlterTableAction {
    type Error = anyhow::Error;
    fn try_from(cmd: &AlterTableCmd) -> Result<Self, Self::Error> {
        let node = cmd.def.as_ref().and_then(|n| n.node.as_ref());
        let node_type = cmd.subtype();

        match (node_type, node) {
            (AlterTableType::AtAddConstraint, Some(NodeEnum::Constraint(constraint))) => {
                let item = ConstraintInfo::try_from(constraint.as_ref()).with_context(|| {
                    let sql = NodeEnum::Constraint(constraint.clone()).deparse();
                    format!("Failed to convert: {:?}", sql)
                })?;
                Ok(Self::Constraint(Box::new(item)))
            }
            (AlterTableType::AtChangeOwner, None) => {
                let owner = cmd.newowner.as_ref().ok_or_else(|| anyhow!("no owner"))?;
                Ok(Self::Owner(owner.rolename.clone()))
            }
            (AlterTableType::AtEnableRowSecurity, None) => Ok(Self::Rls),
            (AlterTableType::AtColumnDefault, Some(n)) => {
                let info = SequenceInfo {
                    column: cmd.name.clone(),
                    node: n.clone(),
                };
                Ok(Self::Sequence(Box::new(info)))
            }
            (ty, node) => {
                warn!("unhandled alter table action: {:?} {:?}", ty, node);
                Ok(Self::Unsupported)
            }
        }
    }
}
