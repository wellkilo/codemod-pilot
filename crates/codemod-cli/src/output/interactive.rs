//! Interactive confirmation prompts using `dialoguer`.

use anyhow::Result;
use colored::Colorize;
use dialoguer::Confirm;

/// Provides interactive prompts for user confirmation.
pub struct InteractivePrompt;

impl InteractivePrompt {
    /// Ask the user a yes/no confirmation question.
    pub fn confirm(message: &str) -> Result<bool> {
        let result = Confirm::new()
            .with_prompt(message)
            .default(false)
            .interact()?;
        Ok(result)
    }

    /// Ask the user to select from a list of options.
    pub fn select(message: &str, options: &[&str]) -> Result<usize> {
        let selection = dialoguer::Select::new()
            .with_prompt(message)
            .items(options)
            .default(0)
            .interact()?;
        Ok(selection)
    }

    /// Print a styled warning message and ask for confirmation.
    pub fn confirm_destructive(action: &str, details: &str) -> Result<bool> {
        println!();
        println!("  {} {}", "WARNING".bold().yellow(), action.bold());
        if !details.is_empty() {
            println!("  {}", details.dimmed());
        }
        println!();

        Self::confirm("Are you sure you want to proceed?")
    }
}
