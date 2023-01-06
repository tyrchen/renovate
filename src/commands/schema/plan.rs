use super::{Args, CommandExecutor};
use crate::{utils::load_config, LocalRepo, RemoteRepo, SchemaLoader};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct SchemaPlanCommand {}

#[async_trait]
impl CommandExecutor for SchemaPlanCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        generate_plan().await?;
        Ok(())
    }
}

pub(super) async fn generate_plan() -> Result<Vec<String>> {
    let config = load_config().await?;
    let local_schema = LocalRepo::new(&config.output.path).load().await?;
    let remote_schema = RemoteRepo::new(&config.url).load().await?;
    let plan = local_schema.plan(&remote_schema, true)?;

    if plan.is_empty() {
        println!("No changes detected.");
        return Ok(vec![]);
    }

    println!("The following SQLs will be applied:\n");
    for item in plan.iter() {
        println!("  {}", item);
    }
    println!("\n");
    Ok(plan)
}
