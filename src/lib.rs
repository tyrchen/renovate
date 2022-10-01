#[cfg(feature = "cli")]
mod cli;
mod parser;
mod repo;
mod utils;

use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub use parser::DatabaseSchema;

#[async_trait]
pub trait SchemaLoader {
    /// Load the sql file(s) to a DatabaseSchema
    async fn load(&self) -> Result<DatabaseSchema>;
}

#[async_trait]
pub trait SqlSaver: ToString + SqlFormatter {
    /// store data to sql files in the given directory
    async fn save(&self, path: impl AsRef<Path>) -> Result<()>;
}

pub trait SqlFormatter {
    /// format the sql to a pretty string
    fn format(&self) -> String;
}

pub trait SqlDiffer {
    type Delta: ToString + MigrationPlanner;
    /// find the schema change
    fn diff(&self, remote: &Self) -> Vec<Self::Delta>;
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
