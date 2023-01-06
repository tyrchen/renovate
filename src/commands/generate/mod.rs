mod_pub_use!(completion);

use super::{Args, CommandExecutor};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct ActionGenerateCommand {
    #[clap(subcommand)]
    pub cmd: Generate,
}

#[async_trait]
impl CommandExecutor for ActionGenerateCommand {
    async fn execute(&self, args: &Args) -> Result<(), Error> {
        self.cmd.execute(args).await
    }
}

subcmd!(Generate, [Completion = "generate shell completion"]);
