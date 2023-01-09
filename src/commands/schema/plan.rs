use super::{Args, CommandExecutor};
use crate::{utils::load_config, DatabaseRepo, LocalRepo, SchemaLoader, SqlLoader};
use clap_utils::{highlight_text, prelude::*};

#[derive(Parser, Debug, Clone)]
pub struct SchemaPlanCommand {}

#[async_trait]
impl CommandExecutor for SchemaPlanCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        generate_plan(false).await?;
        Ok(())
    }
}

pub(super) async fn generate_plan(remote: bool) -> Result<Vec<String>> {
    let config = load_config().await?;
    let db_repo = DatabaseRepo::new(&config);

    let local_schema = if !remote {
        let sql = LocalRepo::new(&config.output.path).load_sql().await?;
        db_repo.normalize(&sql).await?
    } else {
        db_repo.load().await?
    };
    let remote_schema = if !remote {
        db_repo.load().await?
    } else {
        let sql = db_repo.load_sql_string(remote).await?;
        SqlLoader::new(&sql).load().await?
    };
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
        if atty::is(atty::Stream::Stdout) {
            println!("{};", highlight_text(&formatted, "sql", None)?);
        } else {
            println!("{};", formatted);
        }
    }
    Ok(plan)
}
