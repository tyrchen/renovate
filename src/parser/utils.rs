use crate::config::RenovateFormatConfig;

use super::{EmbedConstraint, SchemaId};
use anyhow::Result;
use console::{style, Style};
use pg_query::{
    protobuf::{ConstrType, RangeVar, TypeName},
    Node, NodeEnum,
};
use serde::Deserialize;
use similar::{ChangeTag, TextDiff};
use std::fmt::{self, Write};

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

pub fn node_to_embed_constraint(node: &Node) -> Option<EmbedConstraint> {
    match &node.node {
        Some(NodeEnum::Constraint(v)) => EmbedConstraint::try_from(v.as_ref()).ok(),
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

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub fn create_diff(old: &NodeEnum, new: &NodeEnum) -> Result<String> {
    let format = RenovateFormatConfig::default().into();

    let old = sqlformat::format(&old.deparse()?, &Default::default(), format);
    let new = sqlformat::format(&new.deparse()?, &Default::default(), format);

    diff_text(&old, &new)
}

/// generate the diff between two strings. TODO: this is just for console output for now
fn diff_text(text1: &str, text2: &str) -> Result<String> {
    let mut output = String::new();
    let diff = TextDiff::from_lines(text1, text2);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(&mut output, "{:-^1$}", "-", 80)?;
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(
                    &mut output,
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )?;
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(&mut output, "{}", s.apply_to(value).underlined().on_black())?;
                    } else {
                        write!(&mut output, "{}", s.apply_to(value))?;
                    }
                }
                if change.missing_newline() {
                    writeln!(&mut output)?;
                }
            }
        }
    }

    Ok(output)
}
