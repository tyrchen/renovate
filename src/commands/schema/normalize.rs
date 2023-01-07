use std::fs;

use crate::{utils::load_config, GitRepo, LocalRepo, RemoteRepo, SchemaLoader, SqlSaver};

use super::{Args, CommandExecutor};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct SchemaNormalizeCommand {}

#[async_trait]
impl CommandExecutor for SchemaNormalizeCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let config = load_config().await?;
        {
            let repo = GitRepo::open(".")?;
            if repo.is_dirty() {
                repo.commit("commit schema changes before nomalization")?;
            }
        }

        let local_repo = LocalRepo::new(&config.output.path);
        let sql = local_repo.load_sql().await?;
        // remove all existing sql files in the local repo
        for file in local_repo.files()? {
            fs::remove_file(file)?;
        }

        let repo = RemoteRepo::new(&config.url);
        let schema = repo.normalize(&sql).await?;
        schema.save(&config.output).await?;

        {
            let repo = GitRepo::open(".")?;
            if repo.is_dirty() {
                repo.commit("commit schema changes after nomalization")?;
            }
        }

        Ok(())
    }
}
