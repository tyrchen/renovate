use std::collections::VecDeque;

use super::{utils::get_type_name, Function, SchemaId};
use debug_ignore::DebugIgnore;
use itertools::Itertools;
use pg_query::{protobuf::CreateFunctionStmt, NodeEnum};

impl TryFrom<&CreateFunctionStmt> for Function {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateFunctionStmt) -> Result<Self, Self::Error> {
        let mut name = stmt
            .funcname
            .iter()
            .map(|n| n.deparse())
            .collect::<Result<VecDeque<_>, _>>()?;
        let id = SchemaId::new(name.pop_front().unwrap(), name.into_iter().join("."));
        let args = stmt
            .parameters
            .iter()
            .map(|n| n.deparse())
            .collect::<Result<Vec<_>, _>>()?;
        let returns = stmt
            .return_type
            .iter()
            .map(get_type_name)
            .collect::<Vec<_>>()
            .join(",");
        let node = NodeEnum::CreateFunctionStmt(stmt.clone());
        Ok(Self {
            id,
            args,
            returns,
            node: DebugIgnore(node),
        })
    }
}
