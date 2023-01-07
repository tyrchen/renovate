use crate::parser::ConstraintInfo;
use itertools::Itertools;
use pg_query::{Node, NodeEnum};

pub fn node_to_embed_constraint(node: &Node) -> Option<ConstraintInfo> {
    match &node.node {
        Some(NodeEnum::Constraint(v)) => ConstraintInfo::try_from(v.as_ref()).ok(),
        _ => None,
    }
}

pub fn node_to_string(node: &Node) -> Option<String> {
    match &node.node {
        Some(NodeEnum::String(s)) => Some(s.str.clone()),
        Some(NodeEnum::Integer(i)) => Some(i.ival.to_string()),
        Some(NodeEnum::AConst(a)) => a.val.as_ref().and_then(|v| match &v.node {
            Some(NodeEnum::String(s)) => Some(format!("'{}'", s.str)),
            Some(NodeEnum::Integer(i)) => Some(i.ival.to_string()),
            _ => None,
        }),
        Some(NodeEnum::FuncCall(f)) => {
            let fname = f.funcname.iter().filter_map(node_to_string).join(".");
            let args = f.args.iter().filter_map(node_to_string).join(", ");
            Some(format!("{}({})", fname, args))
        }
        _ => None,
    }
}
