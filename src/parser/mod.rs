mod constraint;
mod function;
mod index;
mod privilege;
mod table;
mod trigger;
mod view;

use anyhow::Result;
use pg_query::NodeEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaId {
    pub schema: String,
    pub name: String,
}
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RelationId {
    pub schema_id: SchemaId,
    pub name: String,
}

/// All the parsed information about a database
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseSchema {
    pub extensions: BTreeMap<String, BTreeMap<String, Extension>>,
    pub types: BTreeMap<String, BTreeMap<String, DataType>>,
    pub tables: BTreeMap<String, BTreeMap<String, Table>>,
    pub sequences: BTreeMap<SchemaId, BTreeMap<String, Sequence>>,
    pub views: BTreeMap<String, BTreeMap<String, View>>,
    pub functions: BTreeMap<String, BTreeMap<String, Function>>,
    pub indexes: BTreeMap<SchemaId, BTreeMap<String, Index>>,
    pub constraints: BTreeMap<SchemaId, BTreeMap<String, Constraint>>,
    pub triggers: BTreeMap<String, Trigger>,
    pub privileges: BTreeMap<String, Privilege>,
    pub policies: BTreeMap<String, BTreeMap<String, Vec<Policy>>>,

    _sequences: BTreeMap<String, BTreeMap<String, Sequence>>,
}

// #[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
// pub struct DatabaseSchema {
//     pub schemas: BTreeMap<String, Schema>,
//     pub triggers: BTreeMap<String, Trigger>,
//     pub privileges: BTreeMap<String, Privilege>,
//     pub foreign_keys: BTreeMap<String, Constraint>,
// }

/// Postgres schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub types: BTreeMap<String, DataType>,
    pub tables: BTreeMap<String, Table>,
    pub views: BTreeMap<String, View>,
    pub functions: BTreeMap<String, Function>,
}

/// Trigger defined in the database
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    // for trigger definition, if it changed we will just drop and recreate it
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

/// Data type defined in the schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataType {
    pub id: SchemaId,
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

/// Table defined in the schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub id: SchemaId,
    pub columns: BTreeMap<String, Column>,
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

/// View defined in the schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct View {
    pub id: SchemaId,
    // for view definition, if it changed we will just drop and recreate it
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

/// Function defined in the schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub id: SchemaId,
    // for function definition, if it changed we will just drop and recreate it
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    pub id: RelationId,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sequence {
    pub id: RelationId,
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub id: RelationId,
    pub sql: String,
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Privilege {
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Index {
    pub id: RelationId,
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Extension {
    pub id: SchemaId,
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Policy {
    pub id: SchemaId,
    #[serde(
        serialize_with = "serialize_node",
        deserialize_with = "deserialize_node"
    )]
    pub node: NodeEnum,
}

fn serialize_node<S>(node: &NodeEnum, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&node.deparse().unwrap())
}

fn deserialize_node<'de, D>(deserializer: D) -> Result<NodeEnum, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(pg_query::parse(&s).unwrap().protobuf.nodes()[0].0.to_enum())
}
