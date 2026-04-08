//! Rule schema definitions for serialization / deserialization.
//!
//! The schema follows a simple YAML structure:
//!
//! ```yaml
//! name: replace-println
//! description: Replace println! with log::info!
//! language: rust
//! version: "1.0"
//! pattern:
//!   before: "println!($fmt, $args)"
//!   after: "log::info!($fmt, $args)"
//! config:
//!   include:
//!     - "src/**/*.rs"
//!   exclude:
//!     - "tests/**"
//!   respect_gitignore: true
//!   max_file_size: 500000
//! ```

use serde::{Deserialize, Serialize};

use crate::error::CodemodError;

/// A complete codemod rule, suitable for serialization to/from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodemodRule {
    /// Unique rule name (kebab-case recommended).
    pub name: String,
    /// Human-readable description of what the rule does.
    pub description: String,
    /// Target programming language (e.g. `"rust"`, `"javascript"`).
    pub language: String,
    /// Semantic version of the rule (default `"1.0"`).
    #[serde(default = "default_version")]
    pub version: String,
    /// The before/after transformation pattern.
    pub pattern: RulePattern,
    /// Optional scanning configuration.
    #[serde(default)]
    pub config: RuleConfig,
}

/// Default version string.
fn default_version() -> String {
    "1.0".to_string()
}

/// The before/after pattern inside a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePattern {
    /// Source pattern (what to look for).
    pub before: String,
    /// Replacement pattern (what to replace it with).
    pub after: String,
}

/// Scanning / filtering configuration embedded in a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Glob patterns for files to include.
    #[serde(default)]
    pub include: Vec<String>,
    /// Glob patterns for files to exclude.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Whether to respect `.gitignore` during scanning.
    #[serde(default = "default_true")]
    pub respect_gitignore: bool,
    /// Optional maximum file size (in bytes) for scanning.
    #[serde(default)]
    pub max_file_size: Option<usize>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            respect_gitignore: true,
            max_file_size: None,
        }
    }
}

/// Default value for boolean fields that should be `true`.
fn default_true() -> bool {
    true
}

impl CodemodRule {
    /// Validate the rule, checking for missing or inconsistent fields.
    ///
    /// # Errors
    ///
    /// Returns [`CodemodError::Rule`] if validation fails.
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.trim().is_empty() {
            return Err(CodemodError::Rule("Rule name must not be empty".into()));
        }
        if self.language.trim().is_empty() {
            return Err(CodemodError::Rule("Rule language must not be empty".into()));
        }
        if self.pattern.before.trim().is_empty() {
            return Err(CodemodError::Rule(
                "Rule pattern.before must not be empty".into(),
            ));
        }
        if self.pattern.after.trim().is_empty() {
            return Err(CodemodError::Rule(
                "Rule pattern.after must not be empty".into(),
            ));
        }
        if self.pattern.before == self.pattern.after {
            return Err(CodemodError::Rule(
                "Rule pattern.before and pattern.after must not be identical".into(),
            ));
        }
        Ok(())
    }

    /// Convert this rule into a [`Pattern`](crate::pattern::Pattern) by
    /// parsing the before/after templates and extracting variables.
    ///
    /// Variables are detected by the `$name` syntax in the templates.
    pub fn to_pattern(&self) -> crate::pattern::Pattern {
        let variables = Self::extract_variables(&self.pattern.before, &self.pattern.after);
        crate::pattern::Pattern::new(
            self.pattern.before.clone(),
            self.pattern.after.clone(),
            variables,
            self.language.clone(),
            1.0, // user-defined rules get maximum confidence
        )
    }

    /// Extract pattern variables from the before and after templates.
    ///
    /// A variable is any token matching `$[a-zA-Z_][a-zA-Z0-9_]*`.
    fn extract_variables(before: &str, after: &str) -> Vec<crate::pattern::PatternVar> {
        let mut seen = std::collections::HashSet::new();
        let mut vars = Vec::new();

        for template in &[before, after] {
            let mut chars = template.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '$' {
                    let mut name = String::from('$');
                    while let Some(&next) = chars.peek() {
                        if next.is_alphanumeric() || next == '_' {
                            name.push(next);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if name.len() > 1 && seen.insert(name.clone()) {
                        vars.push(crate::pattern::PatternVar {
                            name,
                            node_type: None,
                        });
                    }
                }
            }
        }

        vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_rule() {
        let rule = CodemodRule {
            name: "test".into(),
            description: "desc".into(),
            language: "rust".into(),
            version: "1.0".into(),
            pattern: RulePattern {
                before: "old()".into(),
                after: "new()".into(),
            },
            config: RuleConfig::default(),
        };
        assert!(rule.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let rule = CodemodRule {
            name: "".into(),
            description: "desc".into(),
            language: "rust".into(),
            version: "1.0".into(),
            pattern: RulePattern {
                before: "old()".into(),
                after: "new()".into(),
            },
            config: RuleConfig::default(),
        };
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_validate_identical_patterns() {
        let rule = CodemodRule {
            name: "test".into(),
            description: "desc".into(),
            language: "rust".into(),
            version: "1.0".into(),
            pattern: RulePattern {
                before: "same()".into(),
                after: "same()".into(),
            },
            config: RuleConfig::default(),
        };
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_extract_variables() {
        let vars = CodemodRule::extract_variables("foo($arg1, $arg2)", "bar($arg1, $arg2)");
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].name, "$arg1");
        assert_eq!(vars[1].name, "$arg2");
    }

    #[test]
    fn test_extract_variables_dedup() {
        let vars = CodemodRule::extract_variables("f($x, $x)", "g($x)");
        assert_eq!(vars.len(), 1);
    }

    #[test]
    fn test_to_pattern() {
        let rule = CodemodRule {
            name: "test".into(),
            description: "desc".into(),
            language: "rust".into(),
            version: "1.0".into(),
            pattern: RulePattern {
                before: "old($x)".into(),
                after: "new($x)".into(),
            },
            config: RuleConfig::default(),
        };
        let p = rule.to_pattern();
        assert_eq!(p.language, "rust");
        assert_eq!(p.variables.len(), 1);
        assert_eq!(p.confidence, 1.0);
    }

    #[test]
    fn test_default_config() {
        let cfg = RuleConfig::default();
        assert!(cfg.include.is_empty());
        assert!(cfg.exclude.is_empty());
        assert!(cfg.respect_gitignore);
        assert!(cfg.max_file_size.is_none());
    }

    #[test]
    fn test_yaml_roundtrip() {
        let rule = CodemodRule {
            name: "yaml-test".into(),
            description: "Round-trip test".into(),
            language: "javascript".into(),
            version: "2.0".into(),
            pattern: RulePattern {
                before: "require($mod)".into(),
                after: "import $mod".into(),
            },
            config: RuleConfig {
                include: vec!["src/**/*.js".into()],
                exclude: vec!["dist/**".into()],
                respect_gitignore: true,
                max_file_size: Some(500_000),
            },
        };

        let yaml = serde_yaml::to_string(&rule).unwrap();
        let parsed: CodemodRule = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.name, "yaml-test");
        assert_eq!(parsed.config.include, vec!["src/**/*.js"]);
        assert_eq!(parsed.config.max_file_size, Some(500_000));
    }
}
