mod composite_type;
mod enum_type;
mod function;
mod mview;
mod privilege;
mod sequence;
mod table;
mod utils;
mod view;

use derivative::Derivative;
use indexmap::IndexMap;
use pg_query::{
    protobuf::{ConstrType, GrantTargetType, ObjectType},
    NodeEnum,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub schemas: BTreeSet<String>,

    // schema level objects
    pub extensions: BTreeMap<String, BTreeMap<String, Extension>>,
    pub composite_types: BTreeMap<String, BTreeMap<String, CompositeType>>,
    pub enum_types: BTreeMap<String, BTreeMap<String, EnumType>>,
    pub sequences: BTreeMap<String, BTreeMap<String, Sequence>>,
    pub tables: BTreeMap<String, BTreeMap<String, Table>>,
    pub views: BTreeMap<String, BTreeMap<String, View>>,
    pub mviews: BTreeMap<String, BTreeMap<String, MatView>>,
    pub functions: BTreeMap<String, BTreeMap<String, Function>>,

    // database level objects
    pub privileges: BTreeMap<String, BTreeSet<Privilege>>,

    // table level objects
    pub table_indexes: BTreeMap<SchemaId, BTreeMap<String, TableIndex>>,
    pub table_constraints: BTreeMap<SchemaId, BTreeMap<String, TableConstraint>>,
    pub table_sequences: BTreeMap<SchemaId, BTreeMap<String, TableSequence>>,
    pub table_triggers: BTreeMap<SchemaId, BTreeMap<String, Trigger>>,
    pub table_policies: BTreeMap<SchemaId, BTreeMap<String, TablePolicy>>,
    pub table_rls: BTreeMap<SchemaId, TableRls>,
    pub table_owners: BTreeMap<SchemaId, TableOwner>,

    // internal data structures
    _table_sequences: BTreeMap<SchemaId, SequenceInfo>,
}

/// Postgres schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub name: String,
    pub types: BTreeMap<String, CompositeType>,
    pub tables: BTreeMap<String, Table>,
    pub views: BTreeMap<String, View>,
    pub functions: BTreeMap<String, Function>,
}

/// Trigger defined in the database
#[derive(Derivative, Clone)]
#[derivative(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Trigger {
    pub id: RelationId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Composite type defined in the schema
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct CompositeType {
    pub id: SchemaId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Enum type defined in the schema
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct EnumType {
    pub id: SchemaId,
    pub items: BTreeSet<String>,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

/// Table defined in the schema
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq)]
pub struct Table {
    pub id: SchemaId,
    pub columns: IndexMap<String, Column>,
    pub constraints: IndexMap<String, ConstraintInfo>,

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

/// Materialized View defined in the schema
#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct MatView {
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
#[derive(Derivative, Debug, Clone, PartialOrd, Ord)]
#[derivative(PartialEq, Eq)]
pub struct FunctionArg {
    #[derivative(PartialEq = "ignore")]
    pub name: String,
    pub data_type: String,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct Column {
    pub id: RelationId,
    pub type_name: String,
    pub nullable: bool,
    pub default: Option<ConstraintInfo>,
    pub constraints: BTreeSet<ConstraintInfo>,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct Sequence {
    pub id: SchemaId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct TableSequence {
    pub id: RelationId,
    #[derivative(Debug = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub node: NodeEnum,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceInfo {
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
#[derivative(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Privilege {
    pub id: String,
    pub target_type: GrantTargetType,
    pub object_type: ObjectType,
    pub privileges: BTreeMap<String, SinglePriv>,
    pub grantee: String,
    pub grant: bool,
    #[derivative(
        Debug = "ignore",
        PartialOrd = "ignore",
        Ord = "ignore",
        Hash = "ignore"
    )]
    pub node: NodeEnum,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub id: RelationId,
    pub cmd_name: String,
    pub permissive: bool,
    pub roles: BTreeSet<String>,
    pub qual: Option<String>,
    pub with_check: Option<String>,
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
    Sequence(Box<SequenceInfo>),
    Unsupported,
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
