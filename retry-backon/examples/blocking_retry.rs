use anyhow::Result;
use retry_backon::{blocking_retry::BlockingRetryable, exponential::ExponentialBuilder};

fn hello(name: &str) -> Result<String> {
    Ok(format!("hello, {}!", name))
}

fn main() -> Result<()> {
    let content = (|| hello("world"))
        .retry(&ExponentialBuilder::default())
        .call()?;
    println!("fetch succeeded: {:?}", content);

    Ok(())
}
