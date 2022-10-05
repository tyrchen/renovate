mod macros;
pub mod parsec;

use super::ConstraintInfo;
use itertools::Itertools;
use pg_query::{Node, NodeEnum};

pub fn node_to_embed_constraint(node: &Node) -> Option<ConstraintInfo> {
    match &node.node {
        Some(NodeEnum::Constraint(v)) => ConstraintInfo::try_from(v.as_ref()).ok(),
        _ => None,
    }
}

pub fn get_node_str(n: &Node) -> Option<&str> {
    match n.node.as_ref() {
        Some(NodeEnum::String(s)) => Some(s.str.as_str()),
        _ => None,
    }
}

pub fn get_type_name(nodes: &[Node]) -> String {
    nodes.iter().filter_map(get_node_str).join(".")
}

#[allow(dead_code)]
pub fn drain_where<T, Pred: Fn(&T) -> bool>(source: Vec<T>, pred: Pred) -> (Vec<T>, Vec<T>) {
    let mut orig: Vec<T> = Vec::with_capacity(source.len());
    let mut drained: Vec<T> = Vec::new();

    for v in source.into_iter() {
        if pred(&v) {
            drained.push(v);
        } else {
            orig.push(v);
        }
    }
    (orig, drained)
}
