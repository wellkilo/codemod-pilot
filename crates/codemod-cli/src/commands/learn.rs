//! `learn` subcommand — infer a pattern from before/after examples.

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;

use codemod_core::pattern::validator::PatternValidator;
use codemod_core::PatternInferrer;
use codemod_languages::get_language;

use crate::config::SessionState;

/// Arguments for the `learn` subcommand.
#[derive(Args)]
pub struct LearnArgs {
    /// The "before" code snippet
    #[arg(long)]
    before: Option<String>,

    /// The "after" code snippet
    #[arg(long)]
    after: Option<String>,

    /// Path to a YAML file containing multiple examples
    #[arg(long, conflicts_with_all = ["before", "after"])]
    examples: Option<String>,

    /// Target language (auto-detected if not specified)
    #[arg(short, long, default_value = "typescript")]
    language: String,
}

/// A YAML file with multiple before/after examples.
#[derive(serde::Deserialize)]
struct ExamplesFile {
    #[allow(dead_code)]
    language: Option<String>,
    examples: Vec<ExampleEntry>,
}

#[derive(serde::Deserialize)]
struct ExampleEntry {
    before: String,
    after: String,
}

pub fn execute(args: LearnArgs) -> Result<()> {
    println!("{}", "codemod-pilot learn".bold().cyan());
    println!();

    // 1. Resolve language adapter.
    let lang_name = args.language.clone();
    let adapter =
        get_language(&lang_name).with_context(|| format!("Unsupported language: {}", lang_name))?;

    // 2. Collect examples.
    let examples: Vec<(String, String)> = if let Some(ref examples_path) = args.examples {
        let content = std::fs::read_to_string(examples_path)
            .with_context(|| format!("Failed to read examples file: {}", examples_path))?;
        let file: ExamplesFile =
            serde_yaml::from_str(&content).with_context(|| "Failed to parse examples YAML")?;
        file.examples
            .into_iter()
            .map(|e| (e.before, e.after))
            .collect()
    } else {
        let before = args
            .before
            .as_ref()
            .with_context(|| "Either --before/--after or --examples is required")?;
        let after = args
            .after
            .as_ref()
            .with_context(|| "--after is required when --before is provided")?;
        vec![(before.clone(), after.clone())]
    };

    if examples.is_empty() {
        anyhow::bail!("No examples provided");
    }

    println!(
        "{} Learning pattern from {} example(s) [{}]",
        ">>".bold().green(),
        examples.len(),
        lang_name.yellow()
    );
    println!();

    // 3. Infer pattern.
    let inferrer = PatternInferrer::new(adapter);

    let pattern = if examples.len() == 1 {
        inferrer
            .infer_from_example(&examples[0].0, &examples[0].1)
            .with_context(|| "Pattern inference failed")?
    } else {
        inferrer
            .infer_from_examples(&examples)
            .with_context(|| "Pattern inference failed")?
    };

    // 4. Validate the pattern.
    let validation =
        PatternValidator::validate(&pattern).with_context(|| "Pattern validation failed")?;

    if !validation.is_valid {
        println!("{} Pattern validation errors:", "ERROR".bold().red());
        for err in &validation.errors {
            println!("  {} {}", "-".red(), err);
        }
        anyhow::bail!("Inferred pattern is invalid");
    }

    if !validation.warnings.is_empty() {
        println!("{} Warnings:", "WARN".bold().yellow());
        for warn in &validation.warnings {
            println!("  {} {}", "-".yellow(), warn);
        }
        println!();
    }

    // 5. Display inferred pattern.
    println!("{}", "Inferred pattern:".bold().underline());
    println!();
    println!(
        "  {} {}",
        "Before:".bold().red(),
        pattern.before_template.trim()
    );
    println!(
        "  {} {}",
        "After: ".bold().green(),
        pattern.after_template.trim()
    );
    println!();

    if !pattern.variables.is_empty() {
        println!("  {}:", "Variables".bold());
        for var in &pattern.variables {
            let constraint = var.node_type.as_deref().unwrap_or("any");
            println!("    {} ({})", var.name.yellow(), constraint.dimmed());
        }
        println!();
    }

    println!(
        "  {} {:.0}%",
        "Confidence:".bold(),
        pattern.confidence * 100.0
    );
    println!();

    // 6. Save session state.
    let project_root = std::env::current_dir()?;
    let session = SessionState {
        pattern: Some(pattern),
        last_scan_target: None,
        language: lang_name.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    session.save(&project_root)?;

    println!(
        "{} Pattern saved to session. Run {} or {} next.",
        "OK".bold().green(),
        "codemod-pilot scan".cyan(),
        "codemod-pilot apply --preview".cyan()
    );

    Ok(())
}
