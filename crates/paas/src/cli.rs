use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init,
    Deploy,
    Redeploy,
    Status,
    Logs {
        #[arg(short, long)]
        follow: bool,
    },
    Stop,
    Env {
        #[command(subcommand)]
        action: EnvAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum EnvAction {
    Set { key_value: String },
    List,
    Remove { key: String },
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}
