mod_pub_use!(apply, fetch, init, plan);

use super::{Args, CommandExecutor};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct ActionPgCommand {
    #[clap(subcommand)]
    pub cmd: Pg,
}

#[async_trait]
impl CommandExecutor for ActionPgCommand {
    async fn execute(&self, args: &Args) -> Result<(), Error> {
        self.cmd.execute(args).await
    }
}

subcmd!(
    Pg,
    [
        Apply = "apply the migration plan to the remote postgres server",
        Fetch = "fetch the most recent schema from the remote postgres server",
        Init = "init a postgres migration repo",
        Plan = "diff the local change and remote state, then make a migration plan"
    ]
);
