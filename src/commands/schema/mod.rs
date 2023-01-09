mod_pub_use!(apply, fetch, init, normalize, plan);

use super::{Args, CommandExecutor};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct ActionSchemaCommand {
    #[clap(subcommand)]
    pub cmd: Schema,
}

#[async_trait]
impl CommandExecutor for ActionSchemaCommand {
    async fn execute(&self, args: &Args) -> Result<(), Error> {
        self.cmd.execute(args).await
    }
}

subcmd!(
    Schema,
    [
        Apply = "apply the migration plan to the remote database server",
        Fetch = "fetch the most recent schema from the remote database server",
        Init = "init a database migration repo",
        Normalize = "normalize local schema via a temp local database",
        Plan = "diff the local change and remote state, then make a migration plan"
    ]
);

#[cfg(feature = "cli-test")]
fn git_commit(_msg: impl AsRef<str>) -> Result<()> {
    Ok(())
}

#[cfg(not(feature = "cli-test"))]
fn git_commit(msg: impl AsRef<str>) -> Result<()> {
    use crate::GitRepo;
    let repo = if std::path::Path::new(".git").exists() {
        GitRepo::open(".")?
    } else {
        GitRepo::init(".")?
    };
    if repo.is_dirty() {
        repo.commit(msg)?;
    }

    Ok(())
}
#[cfg(feature = "cli-test")]
fn git_dirty() -> Result<bool> {
    Ok(false)
}

#[cfg(not(feature = "cli-test"))]
fn git_dirty() -> Result<bool> {
    let repo = crate::GitRepo::open(".")?;
    Ok(repo.is_dirty())
}
