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