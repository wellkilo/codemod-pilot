//! `export` subcommand — export inferred pattern as a `.codemod.yaml` rule file.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;

use codemod_core::rule::schema::{CodemodRule, RuleConfig, RulePattern};

use crate::config::SessionState;

/// Arguments for the `export` subcommand.
#[derive(Args)]
pub struct ExportArgs {
    /// Output file path (defaults to <name>.codemod.yaml)
    #[arg(short, long)]
    output: Option<String>,

    /// Rule name (kebab-case)
    #[arg(short, long)]
    name: Option<String>,

    /// Rule description
    #[arg(short, long)]
    description: Option<String>,

    /// File include patterns (glob)
    #[arg(long)]
    include: Vec<String>,

    /// File exclude patterns (glob)
    #[arg(long)]
    exclude: Vec<String>,
}

pub fn execute(args: ExportArgs) -> Result<()> {
    println!("{}", "codemod-pilot export".bold().cyan());
    println!();

    // Load session.
    let project_root = std::env::current_dir()?;
    let session = SessionState::load(&project_root)?
        .with_context(|| "No active session. Run `codemod-pilot learn` first.")?;
    let pattern = session
        .pattern
        .with_context(|| "Session has no inferred pattern to export")?;

    // Build rule name.
    let rule_name = args.name.unwrap_or_else(|| {
        format!(
            "codemod-{}",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        )
    });

    let description = args
        .description
        .unwrap_or_else(|| "Auto-generated rule from codemod-pilot learn".to_string());

    let include = if args.include.is_empty() {
        default_includes(&pattern.language)
    } else {
        args.include.clone()
    };
    let exclude = if args.exclude.is_empty() {
        default_excludes()
    } else {
        args.exclude.clone()
    };

    // Build the rule.
    let rule = CodemodRule {
        name: rule_name.clone(),
        description,
        language: pattern.language.clone(),
        version: "1.0".into(),
        pattern: RulePattern {
            before: pattern.before_template.clone(),
            after: pattern.after_template.clone(),
        },
        config: RuleConfig {
            include,
            exclude,
            respect_gitignore: true,
            max_file_size: Some(1_000_000),
        },
    };

    // Validate the rule.
    rule.validate()
        .with_context(|| "Generated rule failed validation")?;

    // Determine output path.
    let output_path = args
        .output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("{}.codemod.yaml", rule_name)));

    codemod_core::rule::save_rule(&rule, &output_path)
        .with_context(|| format!("Failed to write rule to {}", output_path.display()))?;

    println!(
        "{} Rule exported to {}",
        "OK".bold().green(),
        output_path.display().to_string().cyan()
    );
    println!();
    println!("Usage:");
    println!(
        "  {} {}",
        "codemod-pilot scan --rule".dimmed(),
        output_path.display()
    );
    println!(
        "  {} {}",
        "codemod-pilot apply --rule".dimmed(),
        output_path.display()
    );

    Ok(())
}

fn default_includes(language: &str) -> Vec<String> {
    match language {
        "typescript" => vec!["src/**/*.ts".into(), "src/**/*.tsx".into()],
        "javascript" => vec!["src/**/*.js".into(), "src/**/*.jsx".into()],
        _ => vec![],
    }
}

fn default_excludes() -> Vec<String> {
    vec![
        "**/node_modules/**".into(),
        "**/dist/**".into(),
        "**/build/**".into(),
        "**/*.test.*".into(),
        "**/*.spec.*".into(),
    ]
}
