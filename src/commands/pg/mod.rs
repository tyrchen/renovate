mod_pub_use!(init);

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

subcmd!(Pg, [Init = "init a postgres migration repo"]);
