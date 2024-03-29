use crate::parser::ConstraintInfo;
use itertools::Itertools;
use pg_query::{
    protobuf::{AExprKind, RoleSpecType, SqlValueFunctionOp, TypeName},
    Node, NodeEnum,
};

pub fn node_to_embed_constraint(node: &Node) -> Option<ConstraintInfo> {
    match &node.node {
        Some(NodeEnum::Constraint(v)) => ConstraintInfo::try_from(v.as_ref()).ok(),
        _ => None,
    }
}

pub fn type_name_to_string(n: &TypeName) -> String {
    let typname = n.names.iter().filter_map(node_to_string).join(".");
    let typmod = n.typmods.iter().filter_map(node_to_string).join("");
    let array_bounds = array_bounds_to_string(&n.array_bounds);

    match (typmod.as_str(), array_bounds.as_str()) {
        ("", "") => typname,
        ("", b) => format!("{}{}", typname, b),
        (m, "") => format!("{}({})", typname, m),
        (m, b) => format!("{}({}){}", typname, m, b),
    }
}

pub fn array_bounds_to_string(bounds: &[Node]) -> String {
    bounds
        .iter()
        .filter_map(node_to_string)
        .map(|s| {
            if s == "-1" {
                "[]".to_owned()
            } else {
                format!("[{}]", s)
            }
        })
        .join("")
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
            let op_kind = e.kind();
            match (left, right) {
                (Some(l), Some(r)) => match e.kind() {
                    AExprKind::AexprOp => Some(format!("{} {} {}", l, op, r)),
                    AExprKind::AexprOpAll => Some(format!("{} {} ALL ({})", l, op, r)),
                    AExprKind::AexprOpAny => Some(format!("{} {} ANY ({})", l, op, r)),
                    _ => panic!("Unsupported AExprKind: {:?}", op_kind),
                },
                (l, r) => panic!("Expect left and right to exists. Got {:?} and {:?}", l, r),
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
        NodeEnum::TypeName(t) => Some(type_name_to_string(t)),
        NodeEnum::ColumnRef(c) => {
            let fields = c.fields.iter().filter_map(node_to_string).join(",");
            Some(fields)
        }
        NodeEnum::SqlvalueFunction(f) => match f.op() {
            SqlValueFunctionOp::SvfopCurrentUser => Some("CURRENT_USER".to_owned()),
            SqlValueFunctionOp::SvfopCurrentRole => Some("CURRENT_ROLE".to_owned()),
            op => unimplemented!("Unsupported SqlValueFunctionOp: {:?}", op),
        },
        NodeEnum::AArrayExpr(a) => {
            let elements = a.elements.iter().filter_map(node_to_string).join(",");
            Some(format!("ARRAY [{}]", elements))
        }
        NodeEnum::RoleSpec(r) => match r.roletype() {
            RoleSpecType::RolespecCstring => Some(r.rolename.clone()),
            RoleSpecType::RolespecCurrentUser => Some("CURRENT_USER".to_owned()),
            RoleSpecType::RolespecSessionUser => Some("SESSION_USER".to_owned()),
            RoleSpecType::RolespecPublic => None,
            RoleSpecType::Undefined => None,
        },
        _ => None,
    }
}
