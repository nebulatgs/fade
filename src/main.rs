use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{new, setup};

mod client;
mod commands;
mod config;
mod gql;

/// Ephemeral virtual machines, leveraging Fly.io
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a temporary VM
    New,

    /// Configure initial Fly settings
    Setup,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New => new::command().await?,
        Commands::Setup => setup::command().await?,
    }

    Ok(())
}
