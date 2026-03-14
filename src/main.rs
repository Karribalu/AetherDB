use aetherdb::cli::{Cli, Command};
use clap::Parser;

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Serve { config: _ } => {
            // TODO (Week 29): start tokio runtime and HTTP server.
            eprintln!("server not yet implemented — coming in a later milestone");
        }
        Command::Info => {
            println!("AetherDB v{}", env!("CARGO_PKG_VERSION"));
            println!("Text and vector search database backed by object storage.");
        }
    }
}
