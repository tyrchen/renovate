use super::{confirm, git_commit, Args, CommandExecutor};
use crate::{utils::load_config, DatabaseRepo};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct SchemaFetchCommand {}

#[async_trait]
impl CommandExecutor for SchemaFetchCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let config = load_config().await?;
        let repo = DatabaseRepo::new(&config);

        if confirm("This will overwrite the local schema files. Continue?") {
            git_commit("commit schema changes before fetching")?;
            repo.fetch().await?;
        }
        Ok(())
    }
}
