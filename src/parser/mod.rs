mod alter_table;
mod function;
mod index;
mod privilege;
mod sequence;
mod table;
mod table_constraint;
mod table_owner;
mod table_rls;
mod trigger;
mod utils;
mod view;

use debug_ignore::DebugIgnore;
use pg_query::{
    protobuf::{ConstrType, GrantTargetType, ObjectType},
    NodeEnum,
};
use std::collections::{BTreeMap, BTreeSet};

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

    // schema level objects
    pub types: BTreeMap<String, BTreeMap<String, DataType>>,
    pub tables: BTreeMap<String, BTreeMap<String, Table>>,
    pub views: BTreeMap<String, BTreeMap<String, View>>,
    pub functions: BTreeMap<String, BTreeMap<String, Function>>,

    // database level objects
    pub triggers: BTreeMap<String, Trigger>,
    pub privileges: BTreeMap<String, Privilege>,

    // table level objects
    pub table_indexes: BTreeMap<SchemaId, BTreeMap<String, TableIndex>>,
    pub table_constraints: BTreeMap<SchemaId, BTreeMap<String, TableConstraint>>,
    pub table_policies: BTreeMap<SchemaId, Vec<TablePolicy>>,
    pub table_rls: BTreeMap<SchemaId, TableRls>,
    pub table_owners: BTreeMap<SchemaId, TableOwner>,
    pub table_sequences: BTreeMap<SchemaId, BTreeMap<String, TableSequence>>,

    // internal data structures
    _sequences: BTreeMap<String, Sequence>,
    _table_sequences: BTreeMap<SchemaId, SequenceInfo>,
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
    pub constraints: Vec<ConstraintInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub id: RelationId,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableSequence {
    pub id: RelationId,
    pub seq: Sequence,
    pub info: SequenceInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceInfo {
    pub id: RelationId,
    pub column: String,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintInfo {
    pub name: String,
    pub con_type: ConstrType,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableConstraint {
    pub id: RelationId,
    pub info: ConstraintInfo,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Privilege {
    pub id: String,
    pub target_type: GrantTargetType,
    pub object_type: ObjectType,
    pub privileges: BTreeMap<String, SinglePriv>,
    pub grantee: String,
    pub grant: bool,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinglePriv {
    pub name: String,
    pub cols: BTreeSet<String>,
}

/// Index for table or material view
#[derive(Debug, Clone, PartialEq)]
pub struct TableIndex {
    pub id: RelationId,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    pub id: SchemaId,
    pub node: DebugIgnore<NodeEnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TablePolicy {
    pub id: SchemaId,
    pub node: DebugIgnore<NodeEnum>,
}

/// Struct to capture all alter table statements
#[derive(Debug, Clone, PartialEq)]
pub struct AlterTable {
    pub id: SchemaId,
    // for sql from pg_dump, only one action is used
    pub action: AlterTableAction,
    pub node: DebugIgnore<NodeEnum>,
}

/// Supported alter table actions
#[derive(Debug, Clone, PartialEq)]
pub enum AlterTableAction {
    Constraint(Box<ConstraintInfo>),
    Rls,
    Owner(String),
}

/// Struct to capture `ALTER TABLE ENABLE ROW LEVEL SECURITY;`
#[derive(Debug, Clone, PartialEq)]
pub struct TableRls {
    pub id: SchemaId,
    pub node: DebugIgnore<NodeEnum>,
}

/// Struct to capture `ALTER TABLE OWNER TO new_owner;`
#[derive(Debug, Clone, PartialEq)]
pub struct TableOwner {
    pub id: SchemaId,
    pub owner: String,
    pub node: DebugIgnore<NodeEnum>,
}
