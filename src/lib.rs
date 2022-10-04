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

/// Record the old/new for a schema object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDiff<T>
where
    T: DiffItem,
{
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
    type Delta: MigrationPlanner;
    /// find the schema change
    fn diff(&self, remote: &Self) -> Result<Option<Self::Delta>>;
}

pub trait MigrationPlanner {
    type Migration: ToString;

    /// generate drop sql
    fn drop(&self) -> Result<Option<Self::Migration>>;
    /// generate create sql
    fn create(&self) -> Result<Option<Self::Migration>>;
    /// generate alter sql
    fn alter(&self) -> Result<Option<Vec<Self::Migration>>>;

    /// if alter return Some, use the result for migration directly; otherwise, use drop/create for migration
    fn plan(&self) -> Result<Vec<Self::Migration>> {
        if let Some(items) = self.alter()? {
            return Ok(items);
        }

        let drop = self.drop()?;
        let create = self.create()?;
        Ok([drop, create].into_iter().flatten().collect())
    }
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
