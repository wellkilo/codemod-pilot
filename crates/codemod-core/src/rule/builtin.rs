//! Built-in codemod rule templates.
//!
//! This module provides a small library of commonly useful transformation
//! rules that ship with the engine. Users can apply these out of the box or
//! use them as starting points for custom rules.

use super::schema::{CodemodRule, RuleConfig, RulePattern};

/// Collection of built-in codemod rules.
pub struct BuiltinRules;

impl BuiltinRules {
    /// Returns all built-in rules.
    pub fn all() -> Vec<CodemodRule> {
        vec![
            Self::replace_println_with_log(),
            Self::replace_unwrap_with_expect(),
            Self::replace_deprecated_trim(),
        ]
    }

    /// Get a built-in rule by name, or `None` if not found.
    pub fn get(name: &str) -> Option<CodemodRule> {
        Self::all().into_iter().find(|r| r.name == name)
    }

    /// List the names of all available built-in rules.
    pub fn names() -> Vec<&'static str> {
        vec![
            "replace-println-with-log",
            "replace-unwrap-with-expect",
            "replace-deprecated-trim",
        ]
    }

    // -----------------------------------------------------------------
    // Individual rule definitions
    // -----------------------------------------------------------------

    /// Replace `println!` calls with `log::info!`.
    ///
    /// This is one of the most common Rust codemods — switching from ad-hoc
    /// `println!` debugging to structured logging.
    ///
    /// ```yaml
    /// before: "println!($args)"
    /// after:  "log::info!($args)"
    /// ```
    pub fn replace_println_with_log() -> CodemodRule {
        CodemodRule {
            name: "replace-println-with-log".into(),
            description: "Replace println!() calls with log::info!() for structured logging".into(),
            language: "rust".into(),
            version: "1.0".into(),
            pattern: RulePattern {
                before: "println!($args)".into(),
                after: "log::info!($args)".into(),
            },
            config: RuleConfig {
                include: vec!["**/*.rs".into()],
                exclude: vec!["tests/**".into(), "examples/**".into()],
                respect_gitignore: true,
                max_file_size: Some(1_000_000),
            },
        }
    }

    /// Replace `.unwrap()` calls with `.expect("descriptive message")`.
    ///
    /// Bare `.unwrap()` calls produce unhelpful panic messages. This rule
    /// replaces them with `.expect()` so developers can add context.
    ///
    /// ```yaml
    /// before: "$expr.unwrap()"
    /// after:  "$expr.expect(\"TODO: add error context\")"
    /// ```
    pub fn replace_unwrap_with_expect() -> CodemodRule {
        CodemodRule {
            name: "replace-unwrap-with-expect".into(),
            description:
                "Replace .unwrap() with .expect(\"...\") to encourage better error messages".into(),
            language: "rust".into(),
            version: "1.0".into(),
            pattern: RulePattern {
                before: "$expr.unwrap()".into(),
                after: "$expr.expect(\"TODO: add error context\")".into(),
            },
            config: RuleConfig {
                include: vec!["**/*.rs".into()],
                exclude: vec!["tests/**".into()],
                respect_gitignore: true,
                max_file_size: Some(1_000_000),
            },
        }
    }

    /// Replace the deprecated `trim_left()` / `trim_right()` with
    /// `trim_start()` / `trim_end()` (Rust 1.30+).
    ///
    /// ```yaml
    /// before: "$s.trim_left()"
    /// after:  "$s.trim_start()"
    /// ```
    pub fn replace_deprecated_trim() -> CodemodRule {
        CodemodRule {
            name: "replace-deprecated-trim".into(),
            description: "Replace deprecated trim_left()/trim_right() with trim_start()/trim_end()"
                .into(),
            language: "rust".into(),
            version: "1.0".into(),
            pattern: RulePattern {
                before: "$s.trim_left()".into(),
                after: "$s.trim_start()".into(),
            },
            config: RuleConfig {
                include: vec!["**/*.rs".into()],
                exclude: vec![],
                respect_gitignore: true,
                max_file_size: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_valid() {
        for rule in BuiltinRules::all() {
            rule.validate()
                .unwrap_or_else(|e| panic!("Built-in rule '{}' failed validation: {e}", rule.name));
        }
    }

    #[test]
    fn test_get_existing_rule() {
        let rule = BuiltinRules::get("replace-println-with-log");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().language, "rust");
    }

    #[test]
    fn test_get_nonexistent_rule() {
        assert!(BuiltinRules::get("does-not-exist").is_none());
    }

    #[test]
    fn test_names_match_rules() {
        let names = BuiltinRules::names();
        let rules = BuiltinRules::all();
        assert_eq!(names.len(), rules.len());
        for (name, rule) in names.iter().zip(rules.iter()) {
            assert_eq!(*name, rule.name);
        }
    }
}
