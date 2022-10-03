mod constraint;
mod function;
mod index;
mod privilege;
mod table;
mod trigger;
mod utils;
mod view;

use debug_ignore::DebugIgnore;
use pg_query::{protobuf::ConstrType, NodeEnum};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaId {
    pub schema: String,
    pub name: String,
}
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationId {
    pub schema_id: SchemaId,
    pub name: String,
}

/// All the parsed information about a database
#[derive(Debug, Default, Clone, PartialEq)]
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

/// Postgres schema
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub name: String,
    pub types: BTreeMap<String, DataType>,
    pub tables: BTreeMap<String, Table>,
    pub views: BTreeMap<String, View>,
    pub functions: BTreeMap<String, Function>,
}

/// Trigger defined in the database
#[derive(Debug, Clone, PartialEq)]
pub struct Trigger {
    pub id: String,
    // for trigger definition, if it changed we will just drop and recreate it
    pub node: DebugIgnore<NodeEnum>,
}

/// Data type defined in the schema
#[derive(Debug, Clone, PartialEq)]
pub struct DataType {
    pub id: SchemaId,
    pub node: DebugIgnore<NodeEnum>,
}

/// Table defined in the schema
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub id: SchemaId,
    pub columns: BTreeMap<String, Column>,
    pub node: DebugIgnore<NodeEnum>,
}

/// View defined in the schema
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub id: SchemaId,
    // for view definition, if it changed we will just drop and recreate it
    pub node: DebugIgnore<NodeEnum>,
}

/// Function defined in the schema
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub id: SchemaId,
    pub args: Vec<FunctionArg>,
    pub returns: String,
    // for function definition, if it changed we will just drop and recreate it
    pub node: DebugIgnore<NodeEnum>,
}

/// Function defined in the schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionArg {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub type_name: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub constraints: Vec<EmbedConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub id: RelationId,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmbedConstraint {
    pub con_type: ConstrType,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub id: RelationId,
    pub con_type: ConstrType,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Privilege {
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub id: RelationId,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    pub id: SchemaId,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    pub id: SchemaId,
    pub node: DebugIgnore<NodeEnum>,
}
