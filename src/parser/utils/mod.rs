pub mod parsec;

use super::{ConstraintInfo, SchemaId};
use anyhow::Result;
use pg_query::{
    protobuf::{ConstrType, RangeVar, TypeName},
    Node, NodeEnum,
};
use serde::Deserialize;

impl From<&RangeVar> for SchemaId {
    fn from(v: &RangeVar) -> Self {
        let schema_name = if v.schemaname.is_empty() {
            "public"
        } else {
            v.schemaname.as_str()
        };
        Self::new(schema_name, &v.relname)
    }
}

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

pub fn get_type_name(data_type: Option<&TypeName>) -> Option<String> {
    data_type.map(|t| {
        t.names
            .iter()
            .filter_map(get_node_str)
            .collect::<Vec<_>>()
            .join(".")
    })
}

#[allow(dead_code)]
pub fn serialize_node<S>(node: &NodeEnum, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&node.deparse().unwrap())
}

#[allow(dead_code)]
pub fn deserialize_node<'de, D>(deserializer: D) -> Result<NodeEnum, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(pg_query::parse(&s).unwrap().protobuf.nodes()[0].0.to_enum())
}

#[allow(dead_code)]
pub fn serialize_constr_type<S>(con_type: &ConstrType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("{:?}", con_type.clone()))
}

#[allow(dead_code)]
pub fn deserialize_constr_type<'de, D>(deserializer: D) -> Result<ConstrType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let v = match s.as_str() {
        "ConstrNull" => ConstrType::ConstrNull,
        "ConstrNotNull" => ConstrType::ConstrNotnull,
        "ConstrDefault" => ConstrType::ConstrDefault,
        "ConstrIdentity" => ConstrType::ConstrIdentity,
        "ConstrGenerated" => ConstrType::ConstrGenerated,
        "ConstrCheck" => ConstrType::ConstrCheck,
        "ConstrPrimary" => ConstrType::ConstrPrimary,
        "ConstrUnique" => ConstrType::ConstrUnique,
        "ConstrExclusion" => ConstrType::ConstrExclusion,
        "ConstrForeign" => ConstrType::ConstrForeign,
        "ConstrAttrDeferrable" => ConstrType::ConstrAttrDeferrable,
        "ConstrAttrNotDeferrable" => ConstrType::ConstrAttrNotDeferrable,
        "ConstrAttrDeferred" => ConstrType::ConstrAttrDeferred,
        "ConstrAttrImmediate" => ConstrType::ConstrAttrImmediate,
        _ => ConstrType::Undefined,
    };
    Ok(v)
}