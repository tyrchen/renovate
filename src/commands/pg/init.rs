use crate::{RemoteRepo, RenovateConfig, SchemaLoader, SqlSaver};

use super::{Args, CommandExecutor};
use clap_utils::prelude::*;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct PgInitCommand {
    pub url: String,
    #[clap(short, long, value_parser, default_value = ".")]
    pub project_path: PathBuf,
}

#[async_trait]
impl CommandExecutor for PgInitCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let config = RenovateConfig::new(self.url.clone(), self.project_path.clone());

        let repo = RemoteRepo::new(self.url.clone());
        let schema = repo.load().await?;

        schema.save(&config.output).await?;
        config.save(self.project_path.join("renovate.yml")).await?;

        Ok(())
    }
}
