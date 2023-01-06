use std::path::Path;

use super::Args;
use crate::{commands::CommandExecutor, LocalRepo, RemoteRepo, RenovateConfig, SchemaLoader};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct PgPlanCommand {}

#[async_trait]
impl CommandExecutor for PgPlanCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let config_file = Path::new("renovate.yml");
        if !config_file.exists() {
            bail!("config file renovate.yml not found in current directory");
        }
        let config = RenovateConfig::load(config_file).await?;
        let local_schema = LocalRepo::new(&config.output.path).load().await?;
        let remote_schema = RemoteRepo::new(&config.url).load().await?;
        let plan = local_schema.plan(&remote_schema)?;

        println!("The following SQLs will be applied:\n");
        for item in plan {
            println!("  {}", item);
        }
        Ok(())
    }
}
