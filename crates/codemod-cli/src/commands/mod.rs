//! CLI subcommands.

pub mod apply;
pub mod check;
pub mod export;
pub mod learn;
pub mod list;
pub mod scan;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Learn a transformation pattern from before/after examples
    Learn(learn::LearnArgs),

    /// Scan codebase for pattern matches
    Scan(scan::ScanArgs),

    /// Apply transformation to matching code
    Apply(apply::ApplyArgs),

    /// Check for matches in CI mode (no modifications)
    Check(check::CheckArgs),

    /// Export inferred pattern as a reusable rule file
    Export(export::ExportArgs),

    /// List available built-in rules
    List(list::ListArgs),
}

pub fn execute(command: Commands) -> Result<()> {
    match command {
        Commands::Learn(args) => learn::execute(args),
        Commands::Scan(args) => scan::execute(args),
        Commands::Apply(args) => apply::execute(args),
        Commands::Check(args) => check::execute(args),
        Commands::Export(args) => export::execute(args),
        Commands::List(args) => list::execute(args),
    }
}
