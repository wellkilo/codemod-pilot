//! codemod-pilot CLI entry point.

use anyhow::Result;
use clap::Parser;
use env_logger::Env;

mod commands;
mod config;
mod output;

#[derive(Parser)]
#[command(
    name = "codemod-pilot",
    about = "Transform your codebase by example. No AST knowledge required.",
    version,
    author,
    long_about = None,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,

    /// Increase logging verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all output except errors
    #[arg(short = 'q', long, global = true)]
    quiet: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging based on verbosity.
    let log_level = if cli.quiet {
        "error"
    } else {
        match cli.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    // Dispatch to command handler.
    commands::execute(cli.command)
}
