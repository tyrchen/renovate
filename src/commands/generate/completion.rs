use super::{Args, CommandExecutor};
use clap::{CommandFactory, Parser};
use clap_utils::prelude::*;

#[derive(Parser, Debug, Clone)]
pub struct GenerateCompletionCommand {
    /// the type of the shell
    #[clap(default_value = "fish")]
    pub shell_type: ShellType,
}

#[async_trait]
impl CommandExecutor for GenerateCompletionCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        self.shell_type
            .generate_completion("renovate", &mut Args::command());

        Ok(())
    }
}
