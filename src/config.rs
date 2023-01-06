use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlformat::{FormatOptions, Indent};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenovateConfig {
    /// The url of the database to connect to
    pub url: String,
    /// The output config
    #[serde(default)]
    pub output: RenovateOutputConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenovateOutputConfig {
    #[serde(default)]
    pub(crate) layout: Layout,
    #[serde(default = "default_path")]
    pub(crate) path: PathBuf,
    #[serde(default = "default_format")]
    pub(crate) format: Option<RenovateFormatConfig>,
}

/// Layout of the output files when saving the schema
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// Default layout. Each schema has its own directory, with each file for a type of objects.
    #[default]
    Normal,
    /// All objects are in a single file.
    Flat,
    /// Each type has its own directory under the schema directory.
    Nested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenovateFormatConfig {
    /// Controls the type and length of indentation to use. Default 4.
    #[serde(default = "default_indent")]
    indent: u8,
    /// When set, changes reserved keywords to ALL CAPS. Defaults to false.
    #[serde(default = "default_uppercase")]
    uppercase: bool,
    /// Controls the number of line breaks after a query. Default 2.
    #[serde(default = "default_lines")]
    lines_between_queries: u8,
}

impl Default for RenovateFormatConfig {
    fn default() -> Self {
        Self {
            indent: default_indent(),
            uppercase: default_uppercase(),
            lines_between_queries: default_lines(),
        }
    }
}

impl From<RenovateFormatConfig> for FormatOptions {
    fn from(config: RenovateFormatConfig) -> Self {
        Self {
            indent: Indent::Spaces(config.indent),
            uppercase: config.uppercase,
            lines_between_queries: config.lines_between_queries,
        }
    }
}

impl RenovateConfig {
    pub fn new(url: String, path: impl Into<PathBuf>) -> Self {
        Self {
            url,
            output: RenovateOutputConfig::new(path),
        }
    }

    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read configuration: {}", path.display()))?;
        let config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse configuration:\n{}", content))?;
        Ok(config)
    }

    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let content = serde_yaml::to_string(&self)
            .with_context(|| format!("Failed to serialize configuration: {:?}", self))?;
        fs::write(path, content)
            .await
            .with_context(|| format!("Failed to write configuration: {}", path.display()))?;
        Ok(())
    }
}

impl RenovateOutputConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
}

impl Default for RenovateOutputConfig {
    fn default() -> Self {
        Self {
            layout: Layout::default(),
            path: default_path(),
            format: default_format(),
        }
    }
}

fn default_format() -> Option<RenovateFormatConfig> {
    Some(RenovateFormatConfig::default())
}

fn default_path() -> PathBuf {
    PathBuf::from(".")
}

fn default_indent() -> u8 {
    4
}

fn default_uppercase() -> bool {
    true
}

fn default_lines() -> u8 {
    2
}
