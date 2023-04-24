mod opts;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = opts::Opts::parse();

    println!("Hello, world!\n\n{cli:?}");
    Ok(())
}
