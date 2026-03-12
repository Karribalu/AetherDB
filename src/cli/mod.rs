//! CLI entry point: argument parsing and command dispatch.
//!
//! Commands:
//! - `aetherdb serve` — start the HTTP server.
//! - `aetherdb info` — print version and config.
//!
//! The CLI has no storage, index, or query logic. It delegates immediately
//! to the engine or server module.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aetherdb", version, about = "Text and vector search database")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the AetherDB HTTP server.
    Serve {
        /// Path to a JSON config file.
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Print version and effective configuration.
    Info,
}
