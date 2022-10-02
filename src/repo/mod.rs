mod loader;
mod saver;

use crate::{LocalRepo, RemoteRepo};
use std::path::PathBuf;

impl LocalRepo {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl RemoteRepo {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl Default for LocalRepo {
    fn default() -> Self {
        Self::new(".")
    }
}