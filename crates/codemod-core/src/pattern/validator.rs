//! Pattern validation.
//!
//! After a [`Pattern`](super::Pattern) is inferred, it should be validated
//! before being applied to a codebase. This module checks for common issues
//! such as empty templates, unused variables, and low confidence scores.

use super::Pattern;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Result of validating a [`Pattern`].
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// `true` if the pattern passes all hard checks.
    pub is_valid: bool,
    /// Non-fatal issues that the user should review.
    pub warnings: Vec<String>,
    /// Fatal issues that prevent the pattern from being used.
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// Creates a passing result with no warnings or errors.
    fn ok() -> Self {
        Self {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// PatternValidator
// ---------------------------------------------------------------------------

/// Validates that a [`Pattern`] is well-formed and likely to produce correct
/// transformations.
pub struct PatternValidator;

impl PatternValidator {
    /// Validate a pattern and return a [`ValidationResult`].
    ///
    /// ## Checks performed
    ///
    /// | Check | Severity |
    /// |---|---|
    /// | `before_template` is non-empty | error |
    /// | `after_template` is non-empty | error |
    /// | `language` is non-empty | error |
    /// | All variables appear in `before_template` | error |
    /// | All variables appear in `after_template` | warning |
    /// | Confidence >= 0.3 | warning |
    /// | Confidence >= 0.1 | error |
    /// | `before_template` != `after_template` | warning |
    pub fn validate(pattern: &Pattern) -> crate::Result<ValidationResult> {
        let mut result = ValidationResult::ok();

        // --- Hard errors ---

        if pattern.before_template.trim().is_empty() {
            result.errors.push("before_template is empty".into());
        }
        if pattern.after_template.trim().is_empty() {
            result.errors.push("after_template is empty".into());
        }
        if pattern.language.trim().is_empty() {
            result.errors.push("language is not specified".into());
        }

        // Every variable must appear in the before template.
        for var in &pattern.variables {
            if !pattern.before_template.contains(&var.name) {
                result.errors.push(format!(
                    "Variable '{}' does not appear in before_template",
                    var.name
                ));
            }
        }

        // Confidence floor.
        if pattern.confidence < 0.1 {
            result.errors.push(format!(
                "Confidence score ({:.2}) is below the minimum threshold (0.1)",
                pattern.confidence
            ));
        }

        // --- Warnings ---

        // Variables missing from after_template may indicate a deletion-only
        // transform, which is valid but worth flagging.
        for var in &pattern.variables {
            if !pattern.after_template.contains(&var.name) {
                result.warnings.push(format!(
                    "Variable '{}' does not appear in after_template — captured value will be dropped",
                    var.name
                ));
            }
        }

        if pattern.confidence < 0.3 {
            result.warnings.push(format!(
                "Low confidence score ({:.2}); consider providing more examples",
                pattern.confidence
            ));
        }

        if pattern.before_template == pattern.after_template {
            result.warnings.push(
                "before_template and after_template are identical — no transformation will occur"
                    .into(),
            );
        }

        // Check for duplicate variable names.
        let mut seen = std::collections::HashSet::new();
        for var in &pattern.variables {
            if !seen.insert(&var.name) {
                result
                    .warnings
                    .push(format!("Duplicate variable name '{}'", var.name));
            }
        }

        result.is_valid = result.errors.is_empty();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::{Pattern, PatternVar};

    #[test]
    fn test_valid_pattern() {
        let p = Pattern::new(
            "foo($var1)".into(),
            "bar($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "rust".into(),
            0.9,
        );
        let res = PatternValidator::validate(&p).unwrap();
        assert!(res.is_valid);
        assert!(res.errors.is_empty());
    }

    #[test]
    fn test_empty_before_template() {
        let p = Pattern::new(
            "".into(),
            "bar($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "rust".into(),
            0.9,
        );
        let res = PatternValidator::validate(&p).unwrap();
        assert!(!res.is_valid);
        assert!(res.errors.iter().any(|e| e.contains("before_template")));
    }

    #[test]
    fn test_missing_variable_in_before() {
        let p = Pattern::new(
            "foo(x)".into(),
            "bar($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "rust".into(),
            0.9,
        );
        let res = PatternValidator::validate(&p).unwrap();
        assert!(!res.is_valid);
        assert!(res.errors.iter().any(|e| e.contains("$var1")));
    }

    #[test]
    fn test_low_confidence_warning() {
        let p = Pattern::new(
            "foo($var1)".into(),
            "bar($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "rust".into(),
            0.2,
        );
        let res = PatternValidator::validate(&p).unwrap();
        assert!(res.is_valid);
        assert!(res.warnings.iter().any(|w| w.contains("confidence")));
    }

    #[test]
    fn test_identical_templates_warning() {
        let p = Pattern::new(
            "foo($var1)".into(),
            "foo($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "rust".into(),
            0.9,
        );
        let res = PatternValidator::validate(&p).unwrap();
        assert!(res.is_valid);
        assert!(res.warnings.iter().any(|w| w.contains("identical")));
    }
}
