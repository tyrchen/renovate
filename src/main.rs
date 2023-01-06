use clap_utils::prelude::*;
use renovate::commands::{Args, CommandExecutor};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let action = &args.action;
    action.execute(&args).await?;

    Ok(())
}
