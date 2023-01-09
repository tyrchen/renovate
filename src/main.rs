use clap_utils::prelude::*;
use renovate::commands::{Args, CommandExecutor};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let action = &args.action;
    action.execute(&args).await?;

    #[cfg(feature = "cli-test")]
    if args.drop_on_exit {
        use renovate::{DatabaseRepo, RenovateConfig};
        let config = RenovateConfig::load("renovate.yml").await?;
        let repo = DatabaseRepo::new(&config);
        repo.drop_database().await.ok();
    }
    Ok(())
}
