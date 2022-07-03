use anyhow::Result;
use clap::{clap_derive::ArgEnum, Parser, Subcommand};
use commands::{new, setup};

mod client;
mod commands;
mod config;
mod gql;
mod interface;

/// Ephemeral virtual machines, leveraging Fly.io
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Kind {
    Min,
    Docker,
    Full,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a temporary VM
    New {
        /// Kind of VM to create
        #[clap(short, long, arg_enum, value_parser, default_value = "min")]
        kind: Kind,

        /// VM Memory (in MB)
        #[clap(short, long, value_parser = clap::value_parser!(u16).range(2048..16384), default_value = "2048")]
        memory: u16,

        /// VM Memory (in MB)
        #[clap(short, long)]
        region: Option<String>,
    },

    /// Configure initial Fly settings
    Setup,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            kind,
            memory,
            region,
        } => new::command(kind, memory, region).await?,
        Commands::Setup => setup::command().await?,
    }

    Ok(())
}
