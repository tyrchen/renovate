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
        Some(n) => node_enum_to_string(n),
        _ => None,
    }
}

pub fn node_enum_to_string(node: &NodeEnum) -> Option<String> {
    match node {
        NodeEnum::String(s) => Some(s.str.clone()),
        NodeEnum::Integer(i) => Some(i.ival.to_string()),
        NodeEnum::AConst(a) => a.val.as_ref().and_then(|v| match &v.node {
            Some(NodeEnum::String(s)) => Some(format!("'{}'", s.str)),
            Some(NodeEnum::Integer(i)) => Some(i.ival.to_string()),
            _ => None,
        }),
        NodeEnum::FuncCall(f) => {
            let fname = f.funcname.iter().filter_map(node_to_string).join(".");
            let args = f.args.iter().filter_map(node_to_string).join(", ");
            Some(format!("{}({})", fname, args))
        }
        NodeEnum::AExpr(e) => {
            let left = e.lexpr.as_deref().and_then(node_to_string);
            let right = e.rexpr.as_deref().and_then(node_to_string);
            let op = e.name.iter().filter_map(node_to_string).join(".");
            match (left, right) {
                (Some(l), Some(r)) => Some(format!("{} {} {}", l, op, r)),
                _ => None,
            }
        }
        NodeEnum::TypeCast(c) => {
            let arg = c.arg.as_deref().and_then(node_to_string);
            let typname = c
                .type_name
                .as_ref()
                .map(|t| t.names.iter().filter_map(node_to_string).join("."));
            match (arg, typname) {
                (Some(a), Some(t)) => Some(format!("{}::{}", a, t)),
                _ => None,
            }
        }
        NodeEnum::TypeName(t) => {
            let typname = t.names.iter().filter_map(node_to_string).join(".");
            let typmod = t.typmods.iter().filter_map(node_to_string).join("");
            Some(format!("{}({})", typname, typmod))
        }
        NodeEnum::ColumnRef(c) => {
            let fields = c.fields.iter().filter_map(node_to_string).join(",");
            Some(fields)
        }
        NodeEnum::AArrayExpr(a) => {
            let elements = a.elements.iter().filter_map(node_to_string).join(",");
            Some(format!("[{}]", elements))
        }
        _ => None,
    }
}
