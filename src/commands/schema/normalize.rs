use super::{git_commit, Args, CommandExecutor};
use crate::{utils::load_config, DatabaseRepo, LocalRepo, SchemaLoader, SqlSaver};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct SchemaNormalizeCommand {}

#[async_trait]
impl CommandExecutor for SchemaNormalizeCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let config = load_config().await?;

        git_commit("commit schema changes before nomalization")?;

        let local_repo = LocalRepo::new(&config.output.path);
        let schema = local_repo.load().await?;
        let sql = schema.sql(true);

        let repo = DatabaseRepo::new(&config);
        let schema = repo.normalize(&sql).await?;
        schema.save(&config.output).await?;

        git_commit("commit schema changes after nomalization")?;

        Ok(())
    }
}
