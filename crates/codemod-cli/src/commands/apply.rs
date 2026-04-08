//! `apply` subcommand — apply transformation to matching code.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use codemod_core::{
    Pattern, PatternMatcher, ScanConfig, Scanner, TransformApplier, TransformResult,
};
use codemod_core::transform::rollback::RollbackManager;
use codemod_languages::get_language;

use crate::config::SessionState;
use crate::output::{DiffPrinter, InteractivePrompt};

/// Arguments for the `apply` subcommand.
#[derive(Args)]
pub struct ApplyArgs {
    /// Target directory
    #[arg(short, long, default_value = ".")]
    target: String,

    /// Rule file to use
    #[arg(short, long)]
    rule: Option<String>,

    /// Preview changes without applying
    #[arg(long)]
    preview: bool,

    /// Execute the transformation
    #[arg(long)]
    execute: bool,

    /// Rollback the last transformation
    #[arg(long)]
    rollback: bool,

    /// Skip interactive confirmation
    #[arg(long)]
    yes: bool,

    /// File include patterns (glob)
    #[arg(long)]
    include: Vec<String>,

    /// File exclude patterns (glob)
    #[arg(long)]
    exclude: Vec<String>,
}

pub fn execute(args: ApplyArgs) -> Result<()> {
    // Handle rollback first.
    if args.rollback {
        return execute_rollback();
    }

    // Require either --preview or --execute.
    if !args.preview && !args.execute {
        anyhow::bail!(
            "Please specify either {} or {}",
            "--preview".yellow(),
            "--execute".yellow()
        );
    }

    println!("{}", "codemod-pilot apply".bold().cyan());
    println!();

    let target = PathBuf::from(&args.target);
    if !target.exists() {
        anyhow::bail!("Target directory does not exist: {}", args.target);
    }

    // Load pattern.
    let (pattern, language, include, exclude) = load_pattern_and_config(&args)?;

    let adapter = get_language(&language)
        .with_context(|| format!("Unsupported language: {}", language))?;

    // Scan for matches.
    let config = ScanConfig {
        target_dir: target.clone(),
        include_patterns: include,
        exclude_patterns: exclude,
        respect_gitignore: true,
        max_file_size: 1_000_000,
    };

    let scanner = Scanner::new(config, get_language(&language).unwrap());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&[
                "\u{28cb}", "\u{28d9}", "\u{28f9}", "\u{28f8}", "\u{28fc}",
                "\u{28f4}", "\u{28e6}", "\u{28e7}", "\u{28c7}", "\u{28cf}",
            ]),
    );
    pb.set_message("Scanning for matches...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let scan_result = scanner.scan(&pattern).with_context(|| "Scan failed")?;
    pb.finish_and_clear();

    if scan_result.matches.is_empty() {
        println!("{} No matches found.", "INFO".bold().blue());
        return Ok(());
    }

    // Group matches by file.
    let file_groups = group_matches_by_file(&scan_result.matches);

    println!(
        "{} Found {} match(es) across {} file(s)",
        ">>".bold().green(),
        scan_result.total_matches.to_string().yellow(),
        file_groups.len().to_string().yellow()
    );
    println!();

    // Build transform results for each file.
    let matcher = PatternMatcher::new(adapter);
    let transform_results = build_transform_results(&matcher, &pattern, &file_groups)?;

    // Preview mode: show diffs and exit.
    if args.preview {
        for result in &transform_results {
            if result.has_changes() {
                DiffPrinter::print_diff(
                    result.file_path.to_string_lossy().as_ref(),
                    &result.diff,
                );
            }
        }
        let total_applied: usize = transform_results.iter().map(|r| r.applied_count).sum();
        DiffPrinter::print_summary(transform_results.len(), total_applied, 0);
        println!();
        println!(
            "Run with {} to apply these changes.",
            "--execute".cyan()
        );
        return Ok(());
    }

    // Execute mode: confirm and apply.
    if args.execute {
        // Show a summary of changes.
        for result in &transform_results {
            if result.has_changes() {
                DiffPrinter::print_diff(
                    result.file_path.to_string_lossy().as_ref(),
                    &result.diff,
                );
            }
        }

        let total_applied: usize = transform_results.iter().map(|r| r.applied_count).sum();

        // Confirm unless --yes.
        if !args.yes {
            let confirmed = InteractivePrompt::confirm(&format!(
                "Apply {} change(s) to {} file(s)?",
                total_applied,
                transform_results.len()
            ))?;
            if !confirmed {
                println!("{} Aborted.", "CANCEL".bold().yellow());
                return Ok(());
            }
        }

        // Save rollback data before writing.
        let project_root = std::env::current_dir()?;
        let rollback_mgr = RollbackManager::new(&project_root)
            .with_context(|| "Failed to initialize rollback manager")?;
        let rollback_path = rollback_mgr
            .save_rollback(&transform_results)
            .with_context(|| "Failed to save rollback data")?;

        // Apply changes: write new content to files.
        let apply_pb = ProgressBar::new(transform_results.len() as u64);
        apply_pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files",
            )
            .unwrap()
            .progress_chars("=> "),
        );

        let mut applied_count = 0usize;
        for result in &transform_results {
            if result.has_changes() {
                std::fs::write(&result.file_path, &result.new_content)
                    .with_context(|| {
                        format!("Failed to write {}", result.file_path.display())
                    })?;
                applied_count += 1;
            }
            apply_pb.inc(1);
        }
        apply_pb.finish_and_clear();

        DiffPrinter::print_summary(
            transform_results.len(),
            total_applied,
            total_applied,
        );

        println!();
        println!(
            "{} {} file(s) modified. Rollback saved to {}",
            "OK".bold().green(),
            applied_count.to_string().yellow(),
            rollback_path.display().to_string().dimmed()
        );
        println!(
            "  Run {} to undo.",
            "codemod-pilot apply --rollback".cyan()
        );
    }

    Ok(())
}

fn execute_rollback() -> Result<()> {
    println!("{}", "codemod-pilot rollback".bold().cyan());
    println!();

    let project_root = std::env::current_dir()?;
    let rollback_mgr = RollbackManager::new(&project_root)
        .with_context(|| "Failed to initialize rollback manager")?;

    let entries = rollback_mgr
        .list_rollbacks()
        .with_context(|| "Failed to list rollbacks")?;

    if entries.is_empty() {
        anyhow::bail!("No rollback data found. Nothing to undo.");
    }

    // Show available rollbacks.
    println!("{} Available rollbacks:", ">>".bold().green());
    for (i, entry) in entries.iter().enumerate() {
        println!(
            "  [{}] {} — {} file(s) — {}",
            i.to_string().yellow(),
            entry.timestamp.dimmed(),
            entry.file_count.to_string().yellow(),
            entry.description.dimmed()
        );
    }
    println!();

    // Default to the most recent rollback.
    let selection = if entries.len() == 1 {
        0
    } else {
        let options: Vec<&str> = entries
            .iter()
            .map(|e| e.description.as_str())
            .collect();
        InteractivePrompt::select("Select rollback to apply:", &options)?
    };

    let entry = &entries[selection];
    let confirmed = InteractivePrompt::confirm(&format!(
        "Restore {} file(s) from {}?",
        entry.file_count, entry.timestamp
    ))?;

    if !confirmed {
        println!("{} Aborted.", "CANCEL".bold().yellow());
        return Ok(());
    }

    let restored = rollback_mgr
        .apply_rollback(&entry.path)
        .with_context(|| "Failed to apply rollback")?;

    println!(
        "{} Restored {} file(s) to their original state.",
        "OK".bold().green(),
        restored.to_string().yellow()
    );

    Ok(())
}

fn load_pattern_and_config(
    args: &ApplyArgs,
) -> Result<(Pattern, String, Vec<String>, Vec<String>)> {
    if let Some(ref rule_path) = args.rule {
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
        Ok((pattern, rule.language.clone(), inc, exc))
    } else {
        let project_root = std::env::current_dir()?;
        let session = SessionState::load(&project_root)?
            .with_context(|| {
                "No active session. Run `codemod-pilot learn` first or provide --rule"
            })?;
        let pattern = session
            .pattern
            .with_context(|| "Session has no inferred pattern")?;
        Ok((
            pattern,
            session.language,
            args.include.clone(),
            args.exclude.clone(),
        ))
    }
}

/// Group scan matches by file path.
fn group_matches_by_file(
    matches: &[codemod_core::ScanMatch],
) -> Vec<(PathBuf, Vec<&codemod_core::ScanMatch>)> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<PathBuf, Vec<&codemod_core::ScanMatch>> = BTreeMap::new();
    for m in matches {
        groups
            .entry(m.file_path.clone())
            .or_default()
            .push(m);
    }
    groups.into_iter().collect()
}

/// For each file, re-run the pattern matcher to get `Match` objects with
/// bindings, then apply the transformation to produce a `TransformResult`.
fn build_transform_results(
    matcher: &PatternMatcher,
    pattern: &Pattern,
    file_groups: &[(PathBuf, Vec<&codemod_core::ScanMatch>)],
) -> Result<Vec<TransformResult>> {
    let mut results = Vec::new();

    for (file_path, _scan_matches) in file_groups {
        let original_content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let matches = matcher
            .find_matches(&original_content, pattern)
            .with_context(|| format!("Failed to match in {}", file_path.display()))?;

        if matches.is_empty() {
            continue;
        }

        let new_content = TransformApplier::apply(&original_content, pattern, &matches)
            .with_context(|| format!("Failed to transform {}", file_path.display()))?;

        let diff = TransformApplier::generate_diff(
            &file_path.to_string_lossy(),
            &original_content,
            &new_content,
        );

        results.push(TransformResult {
            file_path: file_path.clone(),
            match_count: matches.len(),
            applied_count: matches.len(),
            diff,
            original_content,
            new_content,
        });
    }

    Ok(results)
}
