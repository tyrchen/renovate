mod generate;
mod schema;

use clap_utils::prelude::*;
use generate::*;
use schema::*;

/// Dispatch and execute the command. Make sure to add the new command enum into the enum_dispatch macro below.
#[async_trait]
#[enum_dispatch(Action, Generate, Schema)] // <- [new group] put the new group enum here
pub trait CommandExecutor {
    async fn execute(&self, args: &Args) -> Result<(), Error>;
}

/// Renovate database migration tool
#[derive(Parser, Debug, Clone)]
#[clap(version, author, about, long_about = None)]
pub struct Args {
    /// subcommand to execute
    #[clap(subcommand)]
    pub action: Action,

    /// enable debug mode
    #[clap(long, global = true, value_parser, default_value = "false")]
    pub debug: bool,
}

subcmd!(
    Action,
    // [new group] add the new command enum here
    [Generate = "generate something", Schema = "Schema migration"]
);
