use anyhow::Result;
use renovate::LocalRepo;
use renovate::SchemaLoader;

#[tokio::main]
async fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures/db".to_string());
    let repo = LocalRepo::new(path);
    let schema = repo.load().await?;
    println!("{:#?}", schema);
    Ok(())
}
