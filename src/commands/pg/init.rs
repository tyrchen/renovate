use crate::{RemoteRepo, RenovateConfig, SchemaLoader, SqlSaver};

use super::{Args, CommandExecutor};
use clap_utils::prelude::*;
use std::{fs, path::PathBuf};
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct PgInitCommand {
    #[clap(value_parser = parse_url)]
    pub url: Url,
}

#[async_trait]
impl CommandExecutor for PgInitCommand {
    async fn execute(&self, _args: &Args) -> Result<(), Error> {
        let path = PathBuf::from(format!(".{}", self.url.path()));
        if path.exists() && path.read_dir()?.next().is_some() {
            bail!("directory {} already exists and not empty", path.display());
        } else {
            fs::create_dir(&path)?;
        }
        let config = RenovateConfig::new(self.url.to_string(), &path);
        let repo = RemoteRepo::new(self.url.clone());
        let schema = repo.load().await?;

        schema.save(&config.output).await?;
        config.save(path.join("renovate.yml")).await?;

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
