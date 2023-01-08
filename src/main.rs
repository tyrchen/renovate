use clap_utils::prelude::*;
use renovate::commands::{Args, CommandExecutor};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let action = &args.action;
    action.execute(&args).await?;

    #[cfg(feature = "cli-test")]
    {
        use renovate::{RemoteRepo, RenovateConfig};
        let config = RenovateConfig::load("renovate.yml").await?;
        let repo = RemoteRepo::new(&config.url);
        repo.drop_database().await.ok();
    }
    Ok(())
}
