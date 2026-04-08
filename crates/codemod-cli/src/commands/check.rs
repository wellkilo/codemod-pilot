//! `check` subcommand — CI mode check for pattern matches.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;

use codemod_core::{ScanConfig, Scanner};
use codemod_languages::get_language;

use crate::config::SessionState;

/// Arguments for the `check` subcommand.
#[derive(Args)]
pub struct CheckArgs {
    /// Target directory to check
    #[arg(short, long, default_value = ".")]
    target: String,

    /// Rule file to use
    #[arg(short, long)]
    rule: Option<String>,

    /// Fail with exit code 1 if any matches are found
    #[arg(long)]
    fail_on_match: bool,

    /// File include patterns (glob)
    #[arg(long)]
    include: Vec<String>,

    /// File exclude patterns (glob)
    #[arg(long)]
    exclude: Vec<String>,
}

/// JSON output structures for CI mode.
#[derive(serde::Serialize)]
struct CheckOutput {
    success: bool,
    total_files_scanned: usize,
    total_matches: usize,
    matches: Vec<CheckMatch>,
}

#[derive(serde::Serialize)]
struct CheckMatch {
    file: String,
    line: usize,
    column: usize,
    matched_text: String,
}

pub fn execute(args: CheckArgs) -> Result<()> {
    let target = PathBuf::from(&args.target);
    if !target.exists() {
        anyhow::bail!("Target directory does not exist: {}", args.target);
    }

    // Load pattern.
    let (pattern, language, include, exclude) = if let Some(ref rule_path) = args.rule {
        let rule = codemod_core::rule::load_rule(std::path::Path::new(rule_path))
            .with_context(|| format!("Failed to load rule: {}", rule_path))?;
        let pattern = rule.to_pattern();
        let inc = if args.include.is_empty() {
            rule.config.include.clone()
        } else {
            args.include.clone()
        };
        let exc = if args.exclude.is_empty() {
            rule.config.exclude.clone()
        } else {
            args.exclude.clone()
        };
        (pattern, rule.language.clone(), inc, exc)
    } else {
        let project_root = std::env::current_dir()?;
        let session = SessionState::load(&project_root)?.with_context(|| {
            "No active session. Run `codemod-pilot learn` first or provide --rule"
        })?;
        let pattern = session
            .pattern
            .with_context(|| "Session has no inferred pattern")?;
        (
            pattern,
            session.language,
            args.include.clone(),
            args.exclude.clone(),
        )
    };

    let adapter =
        get_language(&language).with_context(|| format!("Unsupported language: {}", language))?;

    let config = ScanConfig {
        target_dir: target,
        include_patterns: include,
        exclude_patterns: exclude,
        respect_gitignore: true,
        max_file_size: 1_000_000,
    };

    let scanner = Scanner::new(config, adapter);
    let result = scanner.scan(&pattern).with_context(|| "Scan failed")?;

    // Build JSON output.
    let check_matches: Vec<CheckMatch> = result
        .matches
        .iter()
        .map(|m| CheckMatch {
            file: m.file_path.to_string_lossy().into_owned(),
            line: m.line,
            column: m.column,
            matched_text: m.matched_text.clone(),
        })
        .collect();

    let has_matches = !check_matches.is_empty();

    let output = CheckOutput {
        success: !has_matches || !args.fail_on_match,
        total_files_scanned: result.total_files_scanned,
        total_matches: result.total_matches,
        matches: check_matches,
    };

    // Always output JSON in check mode.
    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);

    // Print human-readable summary to stderr.
    if has_matches {
        eprintln!(
            "{} Found {} match(es) in {} file(s) scanned",
            "CHECK".bold().yellow(),
            result.total_matches.to_string().yellow(),
            result.total_files_scanned.to_string().yellow()
        );
    } else {
        eprintln!(
            "{} No matches found ({} files scanned)",
            "CHECK".bold().green(),
            result.total_files_scanned
        );
    }

    // Exit with non-zero code if --fail-on-match and matches were found.
    if args.fail_on_match && has_matches {
        std::process::exit(1);
    }

    Ok(())
}
