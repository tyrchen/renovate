use super::{Args, CommandExecutor};
use crate::{GitRepo, RemoteRepo, RenovateConfig};
use clap_utils::prelude::*;
use std::{env::set_current_dir, fs, path::PathBuf};
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct SchemaInitCommand {
    #[clap(value_parser = parse_url)]
    pub url: Url,
}

#[async_trait]
impl CommandExecutor for SchemaInitCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let path = PathBuf::from(format!(".{}", self.url.path()));
        if path.exists() && path.read_dir()?.next().is_some() {
            bail!("directory {} already exists and not empty", path.display());
        } else {
            fs::create_dir(&path)?;
        }

        set_current_dir(&path)?;
        let config = RenovateConfig::new(self.url.to_string());
        config.save("renovate.yml").await?;

        let remote_repo = RemoteRepo::new(&config.url);
        remote_repo.fetch().await?;

        #[cfg(feature = "cli-test")]
        let is_test = true;
        #[cfg(not(feature = "cli-test"))]
        let is_test = false;
        if !is_test {
            let repo = GitRepo::init(".")?;
            repo.commit(format!("init schema migration repo for {}", self.url))?;
        }

        println!(
            "Database schema for {} has successfully dumped into {}.",
            self.url,
            path.display()
        );
        Ok(())
    }
}

fn parse_url(s: &str) -> Result<Url, Error> {
    let url = Url::parse(s)?;
    if url.scheme() != "postgres" {
        bail!("only postgres url is supported");
    }
    if url.path().is_empty() {
        bail!("database name is required in the url");
    }
    Ok(url)
}
