use super::{generate_plan, Args, CommandExecutor};
use crate::{utils::load_config, GitRepo, RemoteRepo};
use clap_utils::{
    dialoguer::{theme::ColorfulTheme, Confirm},
    prelude::*,
};

#[derive(Parser, Debug, Clone)]
pub struct SchemaApplyCommand {}

#[async_trait]
impl CommandExecutor for SchemaApplyCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let plan = generate_plan().await?;
        if plan.is_empty() {
            return Ok(());
        }
        let config = load_config().await?;
        let remote_repo = RemoteRepo::new(&config.url);

        {
            let repo = GitRepo::open(".")?;
            if repo.is_dirty() && !confirm("Your repo is dirty. Do you want to continue?") {
                bail!("Your repo is dirty. Please commit the changes before applying.");
            }
        }
        if confirm("Do you want to perform this update?") {
            remote_repo.apply(plan).await?;
            {
                let repo = GitRepo::open(".")?;
                if repo.is_dirty() {
                    repo.commit("automatically retrieved most recent schema from remote server")?;
                }
            }
            println!(
                "Successfully applied migration to {}.\nYour repo is updated with the latest schema. See `git diff HEAD~1` for details.",
                config.url
            );
        } else {
            println!("Database schema update has been cancelled.");
        }

        Ok(())
    }
}

pub(crate) fn confirm(prompt: &'static str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .expect("confirm UI should work")
}
