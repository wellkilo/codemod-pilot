//! Scan result report output (text and JSON).

use anyhow::Result;
use colored::Colorize;

use codemod_core::scanner::ScanResult;

/// Prints scan result reports.
pub struct ReportPrinter;

/// JSON-serializable version of a scan match.
#[derive(serde::Serialize)]
struct JsonMatch {
    file: String,
    line: usize,
    column: usize,
    matched_text: String,
}

/// JSON-serializable scan report.
#[derive(serde::Serialize)]
struct JsonReport {
    total_files_scanned: usize,
    total_matches: usize,
    duration_ms: u64,
    matches: Vec<JsonMatch>,
}

impl ReportPrinter {
    /// Print scan results in human-readable text format.
    pub fn print_text(result: &ScanResult) {
        println!(
            "  {} Scanned {} file(s), found {} match(es) in {:.1}s",
            ">>".bold().green(),
            result.total_files_scanned.to_string().yellow(),
            result.total_matches.to_string().yellow(),
            result.duration_ms as f64 / 1000.0,
        );

        if result.matches.is_empty() {
            println!();
            println!("  {} No matches found.", "INFO".bold().blue());
        }
    }

    /// Print scan results as JSON.
    pub fn print_json(result: &ScanResult) -> Result<()> {
        let json_matches: Vec<JsonMatch> = result
            .matches
            .iter()
            .map(|m| JsonMatch {
                file: m.file_path.to_string_lossy().into_owned(),
                line: m.line,
                column: m.column,
                matched_text: m.matched_text.clone(),
            })
            .collect();

        let report = JsonReport {
            total_files_scanned: result.total_files_scanned,
            total_matches: result.total_matches,
            duration_ms: result.duration_ms,
            matches: json_matches,
        };

        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
        Ok(())
    }
}
