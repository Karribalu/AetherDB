use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::core::config::{AppConfig, LogFormat};
use crate::core::telemetry;
use crate::engine::Engine;

#[derive(Debug, Parser)]
#[command(name = "aetherdb", version, about = "AetherDB Day 1 foundation")]
struct Cli {
    #[arg(long, default_value = ".aetherdb")]
    data_dir: PathBuf,

    #[arg(long, value_enum, default_value_t = LogFormat::Text)]
    log_format: LogFormat,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Start,
    Info,
}

pub fn run<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    let config = AppConfig::new(cli.data_dir, cli.log_format);

    telemetry::init(config.log_format)?;

    let engine = Engine::open(config)?;

    match cli.command {
        Command::Start => {
            let paths = engine.layout();
            println!("AetherDB v{}", env!("CARGO_PKG_VERSION"));
            println!("status: initialized");
            println!("data dir: {}", paths.root.display());
            println!("catalog dir: {}", paths.catalog.display());
            println!("wal dir: {}", paths.wal.display());
            println!("snapshots dir: {}", paths.snapshots.display());
        }
        Command::Info => {
            println!("{}", engine.describe());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clap_parses_start_command() {
        let cli = Cli::parse_from(["aetherdb", "start"]);
        assert!(matches!(cli.command, Command::Start));
    }

    #[test]
    fn clap_parses_custom_data_dir() {
        let cli = Cli::parse_from(["aetherdb", "--data-dir", "./tmp-db", "info"]);
        assert_eq!(cli.data_dir, PathBuf::from("./tmp-db"));
        assert!(matches!(cli.command, Command::Info));
    }
}
