use anyhow::Result;
use renovate::RenovateConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures/config/test.yml".to_string());
    let config = RenovateConfig::load(&path).await?;
    println!("{:#?}", config);
    Ok(())
}
