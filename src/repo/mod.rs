mod applier;
pub mod git;
mod loader;
mod saver;

use crate::{DatabaseRepo, LocalRepo, RenovateConfig, SqlLoader};
use std::path::PathBuf;

impl LocalRepo {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl DatabaseRepo {
    pub fn new(config: &RenovateConfig) -> Self {
        Self {
            url: config.url.clone(),
            remote_url: config.remote_url.clone(),
        }
    }

    pub fn new_with(url: String) -> Self {
        Self {
            url: url.clone(),
            remote_url: url,
        }
    }
}

impl Default for LocalRepo {
    fn default() -> Self {
        Self::new(".")
    }
}

impl SqlLoader {
    pub fn new(sql: impl Into<String>) -> Self {
        Self(sql.into())
    }
}
