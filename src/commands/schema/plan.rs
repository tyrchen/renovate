use super::{Args, CommandExecutor};
use crate::{utils::load_config, LocalRepo, RemoteRepo, SchemaLoader};
use clap_utils::{highlight_text, prelude::*};

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
    let remote_repo = RemoteRepo::new(&config.url);
    let sql = LocalRepo::new(&config.output.path).load_sql().await?;
    let local_schema = remote_repo.normalize(&sql).await?;
    let remote_schema = remote_repo.load().await?;
    let plan = local_schema.plan(&remote_schema, true)?;

    if plan.is_empty() {
        println!("No changes detected.");
        return Ok(vec![]);
    }

    println!("The following SQLs will be applied:\n");
    for item in plan.iter() {
        let formatted = sqlformat::format(
            item,
            &Default::default(),
            config.output.format.unwrap_or_default().into(),
        );
        println!("{};", highlight_text(&formatted, "sql", None)?);
    }
    println!("\n");
    Ok(plan)
}
