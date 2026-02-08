use anyhow::Ok;

use crate::{
    cli::{Commands, parse_cli},
    commands::init::init_project,
};

mod cli;
mod commands;

fn main() -> anyhow::Result<()> {
    let args = parse_cli();

    match args.command {
        Commands::Init => init_project(),
        Commands::Deploy => Ok(()),
        Commands::Status => Ok(()),
        Commands::Logs => Ok(()),
        Commands::Stop => Ok(()),
    }
}
