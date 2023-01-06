mod generate;
mod pg;

use clap_utils::prelude::*;
use generate::*;
use pg::*;

/// Dispatch and execute the command. Make sure to add the new command enum into the enum_dispatch macro below.
#[async_trait]
#[enum_dispatch(Action, Generate, Pg)] // <- [new group] put the new group enum here
pub trait CommandExecutor {
    async fn execute(&self, args: &Args) -> Result<(), Error>;
}

/// Cella Team Internal CLI
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
    [
        Generate = "generate something",
        Pg = "Postgres related migration"
    ]
);
