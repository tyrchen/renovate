#[cfg(feature = "cli")]
mod cli;
mod config;
mod macros;
mod parser;
mod repo;
mod utils;

use anyhow::Result;
use async_trait::async_trait;
use config::RenovateOutputConfig;
use std::path::PathBuf;

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

pub trait SqlDiffer {
    type Delta: MigrationPlanner;
    /// find the schema change
    fn diff(&self, remote: &Self) -> Result<Option<Self::Delta>>;
}

pub trait MigrationPlanner {
    type Migration: ToString;
    /// generate schema migration
    fn plan(&self) -> Vec<Self::Migration>;
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
