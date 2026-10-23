mod args;
mod commands;
mod output;

use std::process::ExitCode;

use clap::Parser;
use dubbit_core::error::{cancelled, DubbitError};

use args::{Cli, Command};

fn dispatch(cli: &Cli) -> Result<(), DubbitError> {
    match &cli.command {
        Command::Dub(args) => commands::dub::run(args, cli.json, cli.verbose),
        Command::Models { command } => commands::models::run(command, cli.json),
        Command::Doctor(args) => commands::doctor::run(args, cli.json),
        Command::Languages => {
            commands::languages::run(cli.json);
            Ok(())
        }
        Command::Config { command } => commands::config::run(command, cli.json),
        Command::Project { directory } => commands::project::run(directory, cli.json),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            output::print_error(&error, cli.json);
            if error.code == cancelled().code {
                ExitCode::from(130)
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
