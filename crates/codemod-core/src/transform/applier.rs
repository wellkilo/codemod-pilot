//! Transform applier — replaces pattern matches with the `after_template`.
//!
//! The applier takes a source string, a [`Pattern`], and a list of
//! [`Match`]es and produces a new string where every match has been replaced
//! according to the pattern's `after_template` with variable bindings
//! substituted.
//!
//! Replacements are applied **back-to-front** (highest byte offset first) so
//! that earlier byte ranges remain valid as we mutate the string.

use similar::TextDiff;

use crate::error::CodemodError;
use crate::pattern::matcher::Match;
use crate::pattern::Pattern;

/// Applies pattern-based transformations to source code.
pub struct TransformApplier;

impl TransformApplier {
    /// Apply a pattern transformation to `source`, replacing every occurrence
    /// described by `matches` with the `after_template` (with bindings
    /// substituted).
    ///
    /// # Errors
    ///
    /// Returns [`CodemodError::Transform`] if a replacement cannot be
    /// constructed (e.g. a variable binding is missing).
    pub fn apply(
        source: &str,
        pattern: &Pattern,
        matches: &[Match],
    ) -> crate::Result<String> {
        if matches.is_empty() {
            return Ok(source.to_string());
        }

        // Sort matches by byte range start, descending, so we can replace
        // from the end of the file backwards without invalidating offsets.
        let mut sorted: Vec<&Match> = matches.iter().collect();
        sorted.sort_by(|a, b| b.byte_range.start.cmp(&a.byte_range.start));

        let mut result = source.to_string();

        for m in &sorted {
            let replacement = Self::render_replacement(pattern, m)?;
            // Preserve leading indentation from the original match.
            let indented = Self::preserve_indentation(source, m, &replacement);
            result.replace_range(m.byte_range.clone(), &indented);
        }

        Ok(result)
    }

    /// Generate a unified diff between the original and transformed source.
    ///
    /// The output follows the standard unified diff format and is suitable
    /// for display or writing to a `.patch` file.
    pub fn generate_diff(file_path: &str, original: &str, transformed: &str) -> String {
        let diff = TextDiff::from_lines(original, transformed);
        let mut output = String::new();

        output.push_str(&format!("--- a/{file_path}\n"));
        output.push_str(&format!("+++ b/{file_path}\n"));

        for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
            output.push_str(&format!("{hunk}"));
        }

        output
    }

    // -----------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------

    /// Render the `after_template` with the variable bindings from a single
    /// match substituted in.
    fn render_replacement(pattern: &Pattern, m: &Match) -> crate::Result<String> {
        let mut result = pattern.after_template.clone();

        for var in &pattern.variables {
            if let Some(value) = m.bindings.get(&var.name) {
                result = result.replace(&var.name, value);
            } else {
                // A variable in the pattern has no binding from this match.
                // This is acceptable if the variable does not appear in the
                // after_template (it was only in the before side).
                if result.contains(&var.name) {
                    return Err(CodemodError::Transform(format!(
                        "Variable '{}' has no binding for match at byte offset {}",
                        var.name, m.byte_range.start
                    )));
                }
            }
        }

        Ok(result)
    }

    /// Preserve the leading whitespace / indentation of the original matched
    /// text when the replacement spans multiple lines.
    fn preserve_indentation(source: &str, m: &Match, replacement: &str) -> String {
        // Find the indentation of the line containing the match start.
        let line_start = source[..m.byte_range.start]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0);
        let indent: String = source[line_start..m.byte_range.start]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();

        if indent.is_empty() || !replacement.contains('\n') {
            return replacement.to_string();
        }

        // Re-indent every line of the replacement after the first.
        let mut lines = replacement.lines();
        let mut result = String::new();
        if let Some(first) = lines.next() {
            result.push_str(first);
        }
        for line in lines {
            result.push('\n');
            if !line.is_empty() {
                result.push_str(&indent);
            }
            result.push_str(line);
        }
        // Preserve trailing newline if the replacement had one.
        if replacement.ends_with('\n') {
            result.push('\n');
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::matcher::Position;
    use crate::pattern::PatternVar;
    use std::collections::HashMap;

    fn make_match(
        start: usize,
        end: usize,
        text: &str,
        bindings: HashMap<String, String>,
    ) -> Match {
        Match {
            byte_range: start..end,
            start_position: Position {
                line: 0,
                column: start,
            },
            end_position: Position {
                line: 0,
                column: end,
            },
            matched_text: text.to_string(),
            bindings,
        }
    }

    #[test]
    fn test_apply_single_replacement() {
        let source = "println!(x);";
        let pattern = Pattern::new(
            "println!($var1)".into(),
            "log::info!($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "rust".into(),
            0.9,
        );
        let mut bindings = HashMap::new();
        bindings.insert("$var1".into(), "x".into());
        let m = make_match(0, 12, "println!(x);", bindings);

        let result = TransformApplier::apply(source, &pattern, &[m]).unwrap();
        assert_eq!(result, "log::info!(x)");
    }

    #[test]
    fn test_generate_diff() {
        let original = "line1\nline2\nline3\n";
        let transformed = "line1\nchanged\nline3\n";
        let diff = TransformApplier::generate_diff("test.rs", original, transformed);
        assert!(diff.contains("--- a/test.rs"));
        assert!(diff.contains("+++ b/test.rs"));
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+changed"));
    }

    #[test]
    fn test_preserve_indentation() {
        let source = "fn main() {\n    old_call();\n}";
        let m = make_match(16, 27, "old_call()", HashMap::new());
        let replacement = "new_call(\n    arg\n)";
        let result = TransformApplier::preserve_indentation(source, &m, replacement);
        assert!(result.contains("    arg"));
    }

    #[test]
    fn test_empty_matches() {
        let source = "hello world";
        let pattern = Pattern::new("a".into(), "b".into(), vec![], "rust".into(), 0.9);
        let result = TransformApplier::apply(source, &pattern, &[]).unwrap();
        assert_eq!(result, source);
    }
}
