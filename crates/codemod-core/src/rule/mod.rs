//! Codemod rule management — loading, saving, and validation.
//!
//! A *rule* is a self-contained description of a code transformation that can
//! be serialized to YAML and shared across projects. It bundles a before/after
//! pattern with metadata (name, description, language) and scanning
//! configuration (include/exclude globs).

pub mod builtin;
pub mod schema;

pub use builtin::BuiltinRules;
pub use schema::{CodemodRule, RuleConfig};

use std::path::Path;

use crate::error::CodemodError;

/// Load a codemod rule from a YAML file.
///
/// The rule is deserialized and then validated. Invalid rules are rejected
/// with a [`CodemodError::Rule`].
///
/// # Errors
///
/// - [`CodemodError::Io`] if the file cannot be read.
/// - [`CodemodError::Rule`] if the YAML is malformed or the rule fails
///   validation.
pub fn load_rule(path: &Path) -> crate::Result<CodemodRule> {
    let content =
        std::fs::read_to_string(path).map_err(CodemodError::Io)?;
    let rule: CodemodRule = serde_yaml::from_str(&content)
        .map_err(|e| CodemodError::Rule(format!("Failed to parse rule file: {e}")))?;
    rule.validate()?;
    Ok(rule)
}

/// Save a codemod rule to a YAML file.
///
/// # Errors
///
/// - [`CodemodError::Rule`] if serialization fails.
/// - [`CodemodError::Io`] if the file cannot be written.
pub fn save_rule(rule: &CodemodRule, path: &Path) -> crate::Result<()> {
    let content = serde_yaml::to_string(rule)
        .map_err(|e| CodemodError::Rule(format!("Failed to serialize rule: {e}")))?;
    std::fs::write(path, content).map_err(CodemodError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::schema::RulePattern;
    use std::fs;

    #[test]
    fn test_save_and_load_rule() {
        let tmp = std::env::temp_dir().join("codemod_test_rule.yaml");

        let rule = CodemodRule {
            name: "test-rule".into(),
            description: "A test rule".into(),
            language: "rust".into(),
            version: "1.0".into(),
            pattern: RulePattern {
                before: "println!($var1)".into(),
                after: "log::info!($var1)".into(),
            },
            config: RuleConfig::default(),
        };

        save_rule(&rule, &tmp).unwrap();
        let loaded = load_rule(&tmp).unwrap();

        assert_eq!(loaded.name, "test-rule");
        assert_eq!(loaded.pattern.before, "println!($var1)");
        assert_eq!(loaded.pattern.after, "log::info!($var1)");

        let _ = fs::remove_file(&tmp);
    }
}
