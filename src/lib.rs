#[cfg(feature = "cli")]
mod cli;
mod macros;
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

#[derive(Debug, Default, Clone, Copy)]
pub enum Layout {
    /// Default layout. Each schema has its own directory, with each file for a type of objects.
    #[default]
    Normal,
    /// All objects are in a single file.
    Flat,
    /// Each type has its own directory under the schema directory.
    Nested,
}

#[async_trait]
pub trait SqlSaver: SqlFormatter {
    /// store data to sql files in the given directory
    async fn save(&self, path: &Path, layout: Layout) -> Result<()>;
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
