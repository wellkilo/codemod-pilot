//! `scan` subcommand — scan codebase for pattern matches.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use codemod_core::{ScanConfig, Scanner};
use codemod_languages::get_language;

use crate::config::SessionState;
use crate::output::{DiffPrinter, ReportPrinter};

/// Arguments for the `scan` subcommand.
#[derive(Args)]
pub struct ScanArgs {
    /// Target directory to scan
    #[arg(short, long, default_value = ".")]
    target: String,

    /// Rule file to use (instead of session pattern)
    #[arg(short, long)]
    rule: Option<String>,

    /// Output format: "text" or "json"
    #[arg(long, default_value = "text")]
    format: String,

    /// File include patterns (glob)
    #[arg(long)]
    include: Vec<String>,

    /// File exclude patterns (glob)
    #[arg(long)]
    exclude: Vec<String>,
}

pub fn execute(args: ScanArgs) -> Result<()> {
    println!("{}", "codemod-pilot scan".bold().cyan());
    println!();

    let target = PathBuf::from(&args.target);
    if !target.exists() {
        anyhow::bail!("Target directory does not exist: {}", args.target);
    }

    // Load the pattern from rule file or session.
    let (pattern, language, include, exclude) = if let Some(ref rule_path) = args.rule {
        let rule = codemod_core::rule::load_rule(std::path::Path::new(rule_path))
            .with_context(|| format!("Failed to load rule: {}", rule_path))?;
        let pattern = rule.to_pattern();
        let lang = rule.language.clone();
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
        (pattern, lang, inc, exc)
    } else {
        let project_root = std::env::current_dir()?;
        let session = SessionState::load(&project_root)?
            .with_context(|| {
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

    let adapter = get_language(&language)
        .with_context(|| format!("Unsupported language: {}", language))?;

    // Setup scanner config.
    let config = ScanConfig {
        target_dir: target,
        include_patterns: include,
        exclude_patterns: exclude,
        respect_gitignore: true,
        max_file_size: 1_000_000,
    };

    let scanner = Scanner::new(config, adapter);

    // Show progress.
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&[
                "\u{28cb}", "\u{28d9}", "\u{28f9}", "\u{28f8}", "\u{28fc}",
                "\u{28f4}", "\u{28e6}", "\u{28e7}", "\u{28c7}", "\u{28cf}",
            ]),
    );
    pb.set_message("Scanning files...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let result = scanner
        .scan(&pattern)
        .with_context(|| "Scan failed")?;
    pb.finish_and_clear();

    // Output results.
    match args.format.as_str() {
        "json" => {
            ReportPrinter::print_json(&result)?;
        }
        _ => {
            ReportPrinter::print_text(&result);
            if !result.matches.is_empty() {
                println!();
                for m in &result.matches {
                    DiffPrinter::print_match(m);
                }
            }
        }
    }

    // Update session with last scan target.
    let project_root = std::env::current_dir()?;
    if let Ok(Some(mut session)) = SessionState::load(&project_root) {
        session.last_scan_target = Some(args.target.clone());
        let _ = session.save(&project_root);
    }

    Ok(())
}
