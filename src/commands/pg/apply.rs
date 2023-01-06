use super::{generate_plan, Args, CommandExecutor};
use crate::{utils::load_config, GitRepo, RemoteRepo};
use clap_utils::{dialoguer::Confirm, prelude::*};

#[derive(Parser, Debug, Clone)]
pub struct PgApplyCommand {}

#[async_trait]
impl CommandExecutor for PgApplyCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let plan = generate_plan().await?;
        let config = load_config().await?;
        let remote_repo = RemoteRepo::new(&config.url);

        {
            let repo = GitRepo::open(".")?;
            if repo.is_dirty() {
                bail!("There are uncommitted changes in the current git repo. Please commit them first.");
            }
        }
        if confirm("Do you want to perform this update?") {
            remote_repo.apply(plan).await?;
            {
                let repo = GitRepo::open(".")?;
                repo.commit("automatically retrieved most recent schema from remote server")?;
            }
            println!("Database schema has been updated.");
        } else {
            println!("Database schema update has been cancelled.");
        }

        Ok(())
    }
}

pub(crate) fn confirm(prompt: &'static str) -> bool {
    Confirm::new()
        .with_prompt(prompt)
        .interact()
        .expect("confirm UI should work")
}
