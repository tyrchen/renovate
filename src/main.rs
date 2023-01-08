use clap_utils::prelude::*;
use renovate::{
    commands::{Args, CommandExecutor},
    RemoteRepo, RenovateConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let action = &args.action;
    action.execute(&args).await?;

    if args.drop_on_exit {
        let config = RenovateConfig::load("renovate.yml").await?;
        let repo = RemoteRepo::new(&config.url);
        repo.drop_database().await.ok();
    }
    Ok(())
}
