//! Integration tests for CI mode — rule loading, validation, and serialization.
//!
//! These tests exercise the rule subsystem (`CodemodRule`, `RuleConfig`,
//! `RulePattern`, `BuiltinRules`) without requiring network access or a
//! running CI environment.

use std::path::Path;

use codemod_core::rule::builtin::BuiltinRules;
use codemod_core::rule::schema::{CodemodRule, RuleConfig, RulePattern};
use codemod_core::rule::{load_rule, save_rule};

// ---------------------------------------------------------------------------
// Rule validation
// ---------------------------------------------------------------------------

#[test]
fn test_rule_validation_valid() {
    let rule = CodemodRule {
        name: "test-rule".to_string(),
        description: "A test rule for CI mode".to_string(),
        language: "typescript".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "console.log($msg)".to_string(),
            after: "logger.info($msg)".to_string(),
        },
        config: RuleConfig::default(),
    };

    assert!(rule.validate().is_ok());
}

#[test]
fn test_rule_validation_empty_name() {
    let rule = CodemodRule {
        name: "".to_string(),
        description: "desc".to_string(),
        language: "typescript".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "old()".to_string(),
            after: "new()".to_string(),
        },
        config: RuleConfig::default(),
    };

    assert!(rule.validate().is_err());
}

#[test]
fn test_rule_validation_empty_language() {
    let rule = CodemodRule {
        name: "test".to_string(),
        description: "desc".to_string(),
        language: "".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "old()".to_string(),
            after: "new()".to_string(),
        },
        config: RuleConfig::default(),
    };

    assert!(rule.validate().is_err());
}

#[test]
fn test_rule_validation_empty_before_pattern() {
    let rule = CodemodRule {
        name: "test".to_string(),
        description: "desc".to_string(),
        language: "typescript".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "".to_string(),
            after: "new()".to_string(),
        },
        config: RuleConfig::default(),
    };

    assert!(rule.validate().is_err());
}

#[test]
fn test_rule_validation_identical_patterns() {
    let rule = CodemodRule {
        name: "test".to_string(),
        description: "desc".to_string(),
        language: "typescript".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "same()".to_string(),
            after: "same()".to_string(),
        },
        config: RuleConfig::default(),
    };

    assert!(rule.validate().is_err());
}

// ---------------------------------------------------------------------------
// Rule-to-Pattern conversion
// ---------------------------------------------------------------------------

#[test]
fn test_rule_to_pattern_extracts_variables() {
    let rule = CodemodRule {
        name: "replace-console".to_string(),
        description: "Replace console.log".to_string(),
        language: "typescript".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "console.log($msg)".to_string(),
            after: "logger.info($msg)".to_string(),
        },
        config: RuleConfig::default(),
    };

    let pattern = rule.to_pattern();
    assert_eq!(pattern.language, "typescript");
    assert_eq!(pattern.confidence, 1.0); // user-defined rules get max confidence
    assert_eq!(pattern.variables.len(), 1);
    assert_eq!(pattern.variables[0].name, "$msg");
    assert_eq!(pattern.before_template, "console.log($msg)");
    assert_eq!(pattern.after_template, "logger.info($msg)");
}

#[test]
fn test_rule_to_pattern_multiple_variables() {
    let rule = CodemodRule {
        name: "test".to_string(),
        description: "desc".to_string(),
        language: "typescript".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "$obj.addEventListener($event, $handler)".to_string(),
            after: "$obj.on($event, $handler)".to_string(),
        },
        config: RuleConfig::default(),
    };

    let pattern = rule.to_pattern();
    assert_eq!(pattern.variables.len(), 3);

    let var_names: Vec<&str> = pattern.variables.iter().map(|v| v.name.as_str()).collect();
    assert!(var_names.contains(&"$obj"));
    assert!(var_names.contains(&"$event"));
    assert!(var_names.contains(&"$handler"));
}

#[test]
fn test_rule_to_pattern_deduplicates_variables() {
    let rule = CodemodRule {
        name: "test".to_string(),
        description: "desc".to_string(),
        language: "typescript".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "$x + $x".to_string(),
            after: "2 * $x".to_string(),
        },
        config: RuleConfig::default(),
    };

    let pattern = rule.to_pattern();
    // $x should appear only once despite being used multiple times
    assert_eq!(pattern.variables.len(), 1);
    assert_eq!(pattern.variables[0].name, "$x");
}

// ---------------------------------------------------------------------------
// Rule YAML serialization
// ---------------------------------------------------------------------------

#[test]
fn test_rule_yaml_roundtrip() {
    let rule = CodemodRule {
        name: "yaml-roundtrip".to_string(),
        description: "Test YAML round-trip serialization".to_string(),
        language: "typescript".to_string(),
        version: "2.0".to_string(),
        pattern: RulePattern {
            before: "require($mod)".to_string(),
            after: "import $mod".to_string(),
        },
        config: RuleConfig {
            include: vec!["src/**/*.ts".to_string()],
            exclude: vec!["dist/**".to_string(), "node_modules/**".to_string()],
            respect_gitignore: true,
            max_file_size: Some(500_000),
        },
    };

    let yaml = serde_yaml::to_string(&rule).expect("YAML serialization should succeed");
    let parsed: CodemodRule =
        serde_yaml::from_str(&yaml).expect("YAML deserialization should succeed");

    assert_eq!(parsed.name, "yaml-roundtrip");
    assert_eq!(parsed.version, "2.0");
    assert_eq!(parsed.pattern.before, "require($mod)");
    assert_eq!(parsed.pattern.after, "import $mod");
    assert_eq!(parsed.config.include, vec!["src/**/*.ts"]);
    assert_eq!(parsed.config.exclude.len(), 2);
    assert_eq!(parsed.config.max_file_size, Some(500_000));
    assert!(parsed.config.respect_gitignore);
}

// ---------------------------------------------------------------------------
// Rule file I/O (save and load)
// ---------------------------------------------------------------------------

#[test]
fn test_rule_save_and_load_from_disk() {
    let tmp = std::env::temp_dir().join("codemod_ci_test_rule.yaml");

    let rule = CodemodRule {
        name: "disk-roundtrip".to_string(),
        description: "Test saving and loading from disk".to_string(),
        language: "typescript".to_string(),
        version: "1.0".to_string(),
        pattern: RulePattern {
            before: "old($x)".to_string(),
            after: "new($x)".to_string(),
        },
        config: RuleConfig::default(),
    };

    save_rule(&rule, &tmp).expect("save_rule should succeed");
    assert!(tmp.exists(), "rule file should be written to disk");

    let loaded = load_rule(&tmp).expect("load_rule should succeed");
    assert_eq!(loaded.name, "disk-roundtrip");
    assert_eq!(loaded.pattern.before, "old($x)");
    assert_eq!(loaded.pattern.after, "new($x)");

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_load_rule_nonexistent_file() {
    let result = load_rule(Path::new("/nonexistent/rule.yaml"));
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Rule loading from the rules/ directory
// ---------------------------------------------------------------------------

/// Helper to resolve the workspace root from within the codemod-core crate.
fn workspace_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR for codemod-core is crates/codemod-core/
    // The workspace root is two levels up.
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn test_load_rules_from_rules_directory() {
    let rules_dir = workspace_root().join("rules");
    if !rules_dir.exists() {
        // Rules directory is optional; skip if not present.
        return;
    }

    let mut yaml_count = 0;
    fn visit_dir(dir: &Path, count: &mut usize) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, count);
                } else if path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "yaml" || e == "yml")
                    .unwrap_or(false)
                {
                    let rule = load_rule(&path).unwrap_or_else(|e| {
                        panic!("Failed to load rule from {}: {e}", path.display())
                    });
                    assert!(
                        !rule.name.is_empty(),
                        "Rule from {} should have a non-empty name",
                        path.display()
                    );
                    *count += 1;
                }
            }
        }
    }

    visit_dir(&rules_dir, &mut yaml_count);
    assert!(
        yaml_count > 0,
        "rules/ directory should contain at least one YAML rule file"
    );
}

// ---------------------------------------------------------------------------
// Built-in rules
// ---------------------------------------------------------------------------

#[test]
fn test_builtin_rules_all_valid() {
    for rule in BuiltinRules::all() {
        rule.validate()
            .unwrap_or_else(|e| panic!("Built-in rule '{}' failed validation: {e}", rule.name));
    }
}

#[test]
fn test_builtin_rules_names_match() {
    let names = BuiltinRules::names();
    let rules = BuiltinRules::all();
    assert_eq!(names.len(), rules.len());
    for (name, rule) in names.iter().zip(rules.iter()) {
        assert_eq!(*name, rule.name);
    }
}

#[test]
fn test_builtin_rule_get_existing() {
    let rule = BuiltinRules::get("replace-println-with-log");
    assert!(rule.is_some());
    let rule = rule.unwrap();
    assert_eq!(rule.language, "rust");
    assert!(rule.validate().is_ok());
}

#[test]
fn test_builtin_rule_get_nonexistent() {
    assert!(BuiltinRules::get("nonexistent-rule").is_none());
}

// ---------------------------------------------------------------------------
// RuleConfig defaults
// ---------------------------------------------------------------------------

#[test]
fn test_rule_config_defaults() {
    let config = RuleConfig::default();
    assert!(config.include.is_empty());
    assert!(config.exclude.is_empty());
    assert!(config.respect_gitignore);
    assert!(config.max_file_size.is_none());
}
