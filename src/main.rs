mod cli;
mod core;
mod engine;
mod storage;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run(std::env::args())
}
