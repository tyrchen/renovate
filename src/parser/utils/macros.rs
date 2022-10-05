use crate::{
    parser::{
        Function, MatView, Privilege, Table, TableConstraint, TableIndex, TableOwner, TableRls,
        Trigger, View,
    },
    NodeItem,
};

macro_rules! def_display {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let sql = self.node().deparse().map_err(|_| std::fmt::Error)?;
                write!(f, "{}", sql)
            }
        }
    };
}

def_display!(Table);
def_display!(Privilege);
def_display!(Function);
def_display!(View);
def_display!(MatView);
def_display!(Trigger);
def_display!(TableConstraint);
def_display!(TableIndex);
def_display!(TableOwner);
def_display!(TableRls);

macro_rules! def_simple_planner {
    ($name:ident) => {
        impl crate::MigrationPlanner for crate::NodeDiff<$name> {
            type Migration = String;

            fn drop(&self) -> crate::MigrationResult<Self::Migration> {
                if let Some(old) = &self.old {
                    let sql = old.revert()?.deparse()?;
                    Ok(vec![sql])
                } else {
                    Ok(vec![])
                }
            }

            fn create(&self) -> crate::MigrationResult<Self::Migration> {
                if let Some(new) = &self.new {
                    let sql = new.to_string();
                    Ok(vec![sql])
                } else {
                    Ok(vec![])
                }
            }

            fn alter(&self) -> crate::MigrationResult<Self::Migration> {
                Ok(vec![])
            }
        }
    };
}

def_simple_planner!(Function);
def_simple_planner!(View);
def_simple_planner!(MatView);
def_simple_planner!(Trigger);
def_simple_planner!(TableIndex);
def_simple_planner!(TableOwner);
def_simple_planner!(TableRls);
def_simple_planner!(TableConstraint);
