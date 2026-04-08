//! Colorized diff output for the terminal.

use colored::Colorize;

use codemod_core::scanner::ScanMatch;

/// Prints diffs and summaries to the terminal with color.
pub struct DiffPrinter;

impl DiffPrinter {
    /// Print a unified diff with colors for a single file.
    pub fn print_diff(file_path: &str, diff: &str) {
        if diff.is_empty() {
            return;
        }

        println!("{}", "\u{2500}".repeat(72).dimmed());
        println!("  {}", file_path.bold().underline());
        println!();

        for line in diff.lines() {
            if line.starts_with("---") {
                println!("  {}", line.red());
            } else if line.starts_with("+++") {
                println!("  {}", line.green());
            } else if line.starts_with("@@") {
                println!("  {}", line.cyan());
            } else if line.starts_with('-') {
                println!("  {}", line.red());
            } else if line.starts_with('+') {
                println!("  {}", line.green());
            } else {
                println!("  {}", line);
            }
        }
        println!();
    }

    /// Print a single scan match with context.
    pub fn print_match(m: &ScanMatch) {
        println!("{}", "\u{2500}".repeat(72).dimmed());
        println!(
            "  {} (line {}, col {})",
            m.file_path.display().to_string().bold().underline(),
            m.line,
            m.column,
        );
        println!();

        // Show context before.
        if !m.context_before.is_empty() {
            for line in m.context_before.lines() {
                println!("    {}", line.dimmed());
            }
        }

        // Show the matched text highlighted.
        for line in m.matched_text.lines() {
            println!("    {}{}", "> ".yellow(), line.yellow().bold());
        }

        // Show context after.
        if !m.context_after.is_empty() {
            for line in m.context_after.lines() {
                println!("    {}", line.dimmed());
            }
        }
        println!();
    }

    /// Print a summary of all changes.
    pub fn print_summary(total_files: usize, total_matches: usize, total_applied: usize) {
        println!("{}", "\u{2500}".repeat(72).dimmed());
        println!();
        println!("  {}", "Summary".bold().underline());
        println!("    Files affected:  {}", total_files.to_string().yellow());
        println!(
            "    Total matches:   {}",
            total_matches.to_string().yellow()
        );
        if total_applied > 0 {
            println!("    Changes applied: {}", total_applied.to_string().green());
        }
        println!();
    }
}
