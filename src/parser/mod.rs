mod function;
mod index;
mod privilege;
mod sequence;
mod table;
mod trigger;
mod utils;
mod view;

use derivative::Derivative;
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub name: String,
    pub types: BTreeMap<String, DataType>,
    pub tables: BTreeMap<String, Table>,
    pub views: BTreeMap<String, View>,
    pub functions: BTreeMap<String, Function>,
}

/// Trigger defined in the database
#[derive(Derivative, Clone)]
#[derivative(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Trigger {
    pub id: String,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Data type defined in the schema
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct DataType {
    pub id: SchemaId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Table defined in the schema
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct Table {
    pub id: SchemaId,
    pub columns: BTreeMap<String, Column>,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// View defined in the schema
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct View {
    pub id: SchemaId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Function defined in the schema
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct Function {
    pub id: SchemaId,
    pub args: Vec<FunctionArg>,
    pub returns: String,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Function defined in the schema
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionArg {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Column {
    pub name: String,
    pub type_name: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub constraints: Vec<ConstraintInfo>,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct Sequence {
    pub id: RelationId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSequence {
    pub id: RelationId,
    pub seq: Sequence,
    pub info: SequenceInfo,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceInfo {
    pub id: RelationId,
    pub column: String,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstraintInfo {
    pub name: String,
    pub con_type: ConstrType,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct TableConstraint {
    pub id: RelationId,
    pub info: ConstraintInfo,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct Privilege {
    pub id: String,
    pub target_type: GrantTargetType,
    pub object_type: ObjectType,
    pub privileges: BTreeMap<String, SinglePriv>,
    pub grantee: String,
    pub grant: bool,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SinglePriv {
    pub name: String,
    pub cols: BTreeSet<String>,
}

/// Index for table or material view
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct TableIndex {
    pub id: RelationId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct Extension {
    pub id: SchemaId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct TablePolicy {
    pub id: SchemaId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Struct to capture all alter table statements
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct AlterTable {
    pub id: SchemaId,
    // for sql from pg_dump, only one action is used
    pub action: AlterTableAction,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Supported alter table actions
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlterTableAction {
    Constraint(Box<ConstraintInfo>),
    Rls,
    Owner(String),
}

/// Struct to capture `ALTER TABLE ENABLE ROW LEVEL SECURITY;`
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct TableRls {
    pub id: SchemaId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Struct to capture `ALTER TABLE OWNER TO new_owner;`
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct TableOwner {
    pub id: SchemaId,
    pub owner: String,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}
