use super::{Args, CommandExecutor};
use clap_utils::prelude::*;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct PgInitCommand {
    /// input file which contains the data
    #[clap(short, long, value_parser)]
    pub input: PathBuf,
}

#[async_trait]
impl CommandExecutor for PgInitCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        todo!()
    }
}
