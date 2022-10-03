use super::{AlterTable, AlterTableAction, SchemaId};
use crate::parser::ConstraintInfo;
use anyhow::{anyhow, Context};
use debug_ignore::DebugIgnore;
use pg_query::{
    protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType},
    NodeEnum,
};

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

        Ok(Self {
            id,
            action,
            node: DebugIgnore(node),
        })
    }
}

impl TryFrom<&AlterTableCmd> for AlterTableAction {
    type Error = anyhow::Error;
    fn try_from(cmd: &AlterTableCmd) -> Result<Self, Self::Error> {
        let node = cmd.def.as_ref().and_then(|n| n.node.as_ref());
        let node_type =
            AlterTableType::from_i32(cmd.subtype).ok_or_else(|| anyhow!("no subtype"))?;

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
            _ => todo!(),
        }
    }
}
