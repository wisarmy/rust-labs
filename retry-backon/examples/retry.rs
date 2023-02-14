use anyhow::Result;
use retry_backon::{exponential::ExponentialBuilder, retry::Retryable};

async fn fetch() -> Result<String> {
    Ok(reqwest::get("https://www.rust-lang.org")
        .await?
        .text()
        .await?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let content = fetch.retry(&ExponentialBuilder::default()).await?;
    println!("fetch succeeded: {}", content);

    Ok(())
}
