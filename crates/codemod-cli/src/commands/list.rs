//! `list` subcommand — list available built-in rules.

use anyhow::Result;
use clap::Args;
use colored::Colorize;

use codemod_core::rule::builtin::BuiltinRules;

/// Arguments for the `list` subcommand.
#[derive(Args)]
pub struct ListArgs {
    /// Filter rules by language
    #[arg(short, long)]
    language: Option<String>,

    /// Show detailed information for each rule
    #[arg(long = "details")]
    details: bool,

    /// Output format: "text" or "json"
    #[arg(long, default_value = "text")]
    format: String,
}

pub fn execute(args: ListArgs) -> Result<()> {
    let rules = BuiltinRules::all();

    // Apply filters.
    let filtered: Vec<_> = rules
        .into_iter()
        .filter(|r| {
            if let Some(ref lang) = args.language {
                if r.language != *lang {
                    return false;
                }
            }
            true
        })
        .collect();

    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&filtered)?;
            println!("{}", json);
        }
        _ => {
            println!("{}", "codemod-pilot rules".bold().cyan());
            println!();

            if filtered.is_empty() {
                println!(
                    "  {} No rules match the given filters.",
                    "INFO".bold().blue()
                );
                return Ok(());
            }

            // Print header.
            println!(
                "  {:<35} {:<15} {}",
                "NAME".bold().underline(),
                "LANGUAGE".bold().underline(),
                "DESCRIPTION".bold().underline()
            );

            for rule in &filtered {
                println!(
                    "  {:<35} {:<15} {}",
                    rule.name.green(),
                    rule.language,
                    rule.description
                        .lines()
                        .next()
                        .unwrap_or("")
                        .dimmed()
                );

                if args.details {
                    println!(
                        "    {} {}",
                        "Before:".bold().red(),
                        rule.pattern.before
                    );
                    println!(
                        "    {} {}",
                        "After: ".bold().green(),
                        rule.pattern.after
                    );
                    if !rule.config.include.is_empty() {
                        println!(
                            "    {} {}",
                            "Include:".dimmed(),
                            rule.config.include.join(", ")
                        );
                    }
                    if !rule.config.exclude.is_empty() {
                        println!(
                            "    {} {}",
                            "Exclude:".dimmed(),
                            rule.config.exclude.join(", ")
                        );
                    }
                    println!();
                }
            }

            println!();
            println!(
                "  {} rule(s) available.",
                filtered.len().to_string().yellow()
            );

            // Also show available languages.
            println!();
            println!("  {}:", "Supported languages".bold());
            for lang in codemod_languages::available_languages() {
                println!("    - {}", lang.green());
            }
        }
    }

    Ok(())
}
