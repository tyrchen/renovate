#[cfg(feature = "cli")]
mod cli;
mod config;
mod macros;
mod parser;
mod repo;
mod types;
mod utils;

use anyhow::Result;
use async_trait::async_trait;
use config::RenovateOutputConfig;
use pg_query::NodeEnum;
use std::{collections::BTreeSet, path::PathBuf};

pub use config::RenovateConfig;
pub use parser::DatabaseSchema;

#[async_trait]
pub trait SchemaLoader {
    /// Load the sql file(s) to a DatabaseSchema
    async fn load(&self) -> Result<DatabaseSchema>;
}

#[async_trait]
pub trait SqlSaver {
    /// store data to sql files in the given directory
    async fn save(&self, config: &RenovateOutputConfig) -> Result<()>;
}

/// Object for SqlDiff<T> must satisfy DiffItem trait
pub trait DiffItem {
    fn id(&self) -> String;
    fn node(&self) -> &NodeEnum;
}

pub trait NodeItem {
    type Inner;
    /// Unique id for the object
    fn id(&self) -> String;
    /// get node for the item
    fn node(&self) -> &NodeEnum;
    /// get the inner item
    fn inner(&self) -> Result<&Self::Inner>;
    /// convert by mapping inner item with optional data and generate a new node
    fn map<F, T>(&self, f: F, data: Option<T>) -> Result<NodeEnum>
    where
        F: Fn(&Self::Inner, Option<T>) -> Result<NodeEnum>,
    {
        f(self.inner()?, data)
    }
    /// revert the node. For example, a `GRANT xxx` will become `REVOKE xxx`
    fn revert(&self) -> Result<NodeEnum>;
}

/// Record the old/new for a schema object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDiff<T> {
    pub old: Option<T>,
    pub new: Option<T>,
    pub diff: String,
}

/// Record the changes for a schema object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDelta<T> {
    pub added: BTreeSet<T>,
    pub removed: BTreeSet<T>,
    pub changed: BTreeSet<(T, T)>,
}

/// Diffing two objects to get deltas
pub trait Differ {
    type Diff: MigrationPlanner;
    /// find the schema change
    fn diff(&self, remote: &Self) -> Result<Option<Self::Diff>>;
}

pub type MigrationResult<T> = Result<Vec<T>>;
pub trait MigrationPlanner {
    type Migration: ToString;

    /// generate drop planner
    fn drop(&self) -> MigrationResult<Self::Migration>;
    /// generate create planner
    fn create(&self) -> MigrationResult<Self::Migration>;
    /// generate alter planner
    fn alter(&self) -> MigrationResult<Self::Migration>;

    /// if alter return Some, use the result for migration directly; otherwise, use drop/create for migration
    fn plan(&self) -> Result<Vec<Self::Migration>> {
        // if alter result is available, use that for migration
        let items = self.alter()?;
        if !items.is_empty() {
            return Ok(items);
        }

        // otherwise, we do drop/create for migration
        let drop = self.drop()?;
        let create = self.create()?;
        Ok([drop, create].into_iter().flatten().collect())
    }
}

/// A trait for the diff object to generate proper migration sql
pub trait DeltaItem {
    /// The node which will be used to generated the final SQL
    type SqlNode: NodeItem;
    /// generate sql for drop
    fn drop(self, node: &Self::SqlNode) -> Result<Vec<String>>;
    /// generate sql for create
    fn create(self, node: &Self::SqlNode) -> Result<Vec<String>>;
    /// generate sql for alter
    fn alter(self, node: &Self::SqlNode, remote: Self) -> Result<Vec<String>>;
}

#[async_trait]
pub trait MigrationExecutor {
    /// execute the migration
    async fn execute(&self) -> Result<()>;
}

/// Local repository
#[derive(Debug, Clone)]
pub struct LocalRepo {
    pub path: PathBuf,
}

/// Remote repository
#[derive(Debug, Clone)]
pub struct RemoteRepo {
    pub url: String,
}
