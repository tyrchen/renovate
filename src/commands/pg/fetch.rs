use super::{Args, CommandExecutor};
use crate::{utils::load_config, RemoteRepo};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct PgFetchCommand {}

#[async_trait]
impl CommandExecutor for PgFetchCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let config = load_config().await?;
        let repo = RemoteRepo::new(&config.url);
        repo.fetch().await?;
        Ok(())
    }
}
