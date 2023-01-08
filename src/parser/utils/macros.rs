use crate::{
    parser::{
        AlterTable, CompositeType, EnumType, Function, MatView, Privilege, Sequence, Table,
        TableConstraint, TableIndex, TableOwner, TableRls, TableSequence, Trigger, View,
    },
    MigrationPlanner, MigrationResult, NodeDiff, NodeItem,
};
use anyhow::Context;
use pg_query::NodeRef;
use std::str::FromStr;

macro_rules! def_display {
    ($($name:ident),*) => {
        $(impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let sql = self.node().deparse().map_err(|_| std::fmt::Error)?;
                write!(f, "{}", sql)
            }
        })*
    };
}

macro_rules! def_simple_planner {
    ($($name:ident),*) => {
        $(impl MigrationPlanner for NodeDiff<$name> {
            type Migration = String;

            fn drop(&self) -> MigrationResult<Self::Migration> {
                if let Some(old) = &self.old {
                    let sql = old.revert()?.deparse()?;
                    Ok(vec![sql])
                } else {
                    Ok(vec![])
                }
            }

            fn create(&self) -> MigrationResult<Self::Migration> {
                if let Some(new) = &self.new {
                    let sql = new.to_string();
                    Ok(vec![sql])
                } else {
                    Ok(vec![])
                }
            }

            fn alter(&self) -> MigrationResult<Self::Migration> {
                Ok(vec![])
            }
        })*
    };
}

macro_rules! def_from_str {
    ($name:ident, $node_name:ident) => {
        impl FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> anyhow::Result<Self> {
                let parsed = pg_query::parse(s)
                    .with_context(|| format!("Failed to parse {}: {}", stringify!($name), s))?;
                let node = parsed.protobuf.nodes()[0].0;
                match node {
                    NodeRef::$node_name(stmt) => Self::try_from(stmt),
                    _ => anyhow::bail!("not a {}: {}", stringify!($name), s),
                }
            }
        }
    };
    ($name:ident) => {
        impl FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> anyhow::Result<Self> {
                let parsed = pg_query::parse(s).with_context(|| {
                    format!(
                        "Failed to parse {} for alter table: {}",
                        stringify!($name),
                        s
                    )
                })?;
                let node = parsed.protobuf.nodes()[0].0;
                match node {
                    NodeRef::AlterTableStmt(stmt) => AlterTable::try_from(stmt)?.try_into(),
                    _ => anyhow::bail!("not a {}: {}", stringify!($name), s),
                }
            }
        }
    };
}

def_display!(
    CompositeType,
    EnumType,
    Function,
    MatView,
    Privilege,
    Sequence,
    Table,
    TableConstraint,
    TableIndex,
    TableOwner,
    TableRls,
    TableSequence,
    Trigger,
    View
);

def_simple_planner!(
    CompositeType,
    Function,
    MatView,
    Sequence,
    TableConstraint,
    TableIndex,
    TableOwner,
    TableRls,
    TableSequence,
    Trigger,
    View
);

def_from_str!(CompositeType, CompositeTypeStmt);
def_from_str!(EnumType, CreateEnumStmt);
def_from_str!(Function, CreateFunctionStmt);
def_from_str!(MatView, CreateTableAsStmt);
def_from_str!(Sequence, CreateSeqStmt);
def_from_str!(Table, CreateStmt);
def_from_str!(TableConstraint);
def_from_str!(TableIndex, IndexStmt);
def_from_str!(TableOwner);
def_from_str!(TableRls);
def_from_str!(TableSequence);
def_from_str!(Trigger, CreateTrigStmt);
def_from_str!(Privilege, GrantStmt);
def_from_str!(View, ViewStmt);
