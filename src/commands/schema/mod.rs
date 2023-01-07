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
