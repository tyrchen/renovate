use crate::{utils::load_config, GitRepo, LocalRepo, RemoteRepo, SchemaLoader, SqlSaver};

use super::{Args, CommandExecutor};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct SchemaNormalizeCommand {}

#[async_trait]
impl CommandExecutor for SchemaNormalizeCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        // {
        //     let repo = GitRepo::open(".")?;
        //     if repo.is_dirty() {
        //         repo.commit("commit schema changes before nomalization")?;
        //     }
        // }
        let config = load_config().await?;
        let local_repo = LocalRepo::new(&config.output.path);
        let repo = RemoteRepo::new(&config.url);
        let schema = repo.normalize(&local_repo.load_sql().await?).await?;
        schema.save(&config.output).await
    }
}
