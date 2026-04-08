//! Integration tests for the `learn` workflow.
//!
//! These tests exercise pattern inference and serialization without requiring
//! a full tree-sitter language grammar. They validate that the core data
//! structures (`Pattern`, `PatternVar`) behave correctly in realistic
//! scenarios and that YAML round-tripping works as expected.

use codemod_core::pattern::{Pattern, PatternVar};
use codemod_core::pattern::validator::PatternValidator;

// ---------------------------------------------------------------------------
// Pattern construction and property tests
// ---------------------------------------------------------------------------

#[test]
fn test_simple_rename_pattern_construction() {
    // Verify that a simple function-rename pattern can be constructed
    // with the expected variable placeholders.
    let pattern = Pattern::new(
        "fetchUserInfo($id)".to_string(),
        "getUserProfile($id)".to_string(),
        vec![PatternVar {
            name: "$id".to_string(),
            node_type: Some("identifier".to_string()),
        }],
        "typescript".to_string(),
        0.95,
    );

    assert_eq!(pattern.before_template, "fetchUserInfo($id)");
    assert_eq!(pattern.after_template, "getUserProfile($id)");
    assert_eq!(pattern.variables.len(), 1);
    assert_eq!(pattern.variables[0].name, "$id");
    assert_eq!(
        pattern.variables[0].node_type.as_deref(),
        Some("identifier")
    );
    assert_eq!(pattern.language, "typescript");
    assert!(pattern.has_variables());
    assert!(pattern.meets_confidence(0.9));
    assert!(!pattern.meets_confidence(0.99));
}

#[test]
fn test_multi_variable_pattern() {
    // A pattern that captures multiple independent variables.
    let pattern = Pattern::new(
        "console.$method($args)".to_string(),
        "logger.$method($args)".to_string(),
        vec![
            PatternVar {
                name: "$method".to_string(),
                node_type: Some("identifier".to_string()),
            },
            PatternVar {
                name: "$args".to_string(),
                node_type: None,
            },
        ],
        "typescript".to_string(),
        0.85,
    );

    assert_eq!(pattern.variables.len(), 2);
    assert!(pattern.has_variables());
    assert!(pattern.meets_confidence(0.8));
}

#[test]
fn test_pattern_without_variables() {
    // A literal-replacement pattern with no variables.
    let pattern = Pattern::new(
        "componentWillMount()".to_string(),
        "componentDidMount()".to_string(),
        vec![],
        "typescript".to_string(),
        1.0,
    );

    assert!(!pattern.has_variables());
    assert!(pattern.meets_confidence(1.0));
}

// ---------------------------------------------------------------------------
// Pattern YAML serialization round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_pattern_yaml_serialization_roundtrip() {
    let pattern = Pattern::new(
        "fetchUser($id)".to_string(),
        "getUser($id)".to_string(),
        vec![PatternVar {
            name: "$id".to_string(),
            node_type: Some("identifier".to_string()),
        }],
        "typescript".to_string(),
        0.95,
    );

    let yaml = serde_yaml::to_string(&pattern).expect("serialization should succeed");
    assert!(yaml.contains("before_template"));
    assert!(yaml.contains("after_template"));
    assert!(yaml.contains("fetchUser($id)"));

    let deserialized: Pattern =
        serde_yaml::from_str(&yaml).expect("deserialization should succeed");
    assert_eq!(deserialized.before_template, pattern.before_template);
    assert_eq!(deserialized.after_template, pattern.after_template);
    assert_eq!(deserialized.variables.len(), pattern.variables.len());
    assert_eq!(deserialized.language, pattern.language);
    assert!((deserialized.confidence - pattern.confidence).abs() < f64::EPSILON);
}

#[test]
fn test_pattern_json_serialization_roundtrip() {
    let pattern = Pattern::new(
        "require($mod)".to_string(),
        "import $mod".to_string(),
        vec![PatternVar {
            name: "$mod".to_string(),
            node_type: None,
        }],
        "javascript".to_string(),
        0.88,
    );

    let json = serde_json::to_string_pretty(&pattern).expect("JSON serialization should succeed");
    let deserialized: Pattern =
        serde_json::from_str(&json).expect("JSON deserialization should succeed");

    assert_eq!(deserialized.before_template, pattern.before_template);
    assert_eq!(deserialized.after_template, pattern.after_template);
    assert_eq!(deserialized.variables.len(), 1);
    assert_eq!(deserialized.variables[0].name, "$mod");
    assert!(deserialized.variables[0].node_type.is_none());
}

// ---------------------------------------------------------------------------
// Pattern validation
// ---------------------------------------------------------------------------

#[test]
fn test_validation_valid_pattern() {
    let pattern = Pattern::new(
        "old($x)".to_string(),
        "new($x)".to_string(),
        vec![PatternVar {
            name: "$x".to_string(),
            node_type: None,
        }],
        "typescript".to_string(),
        0.9,
    );

    let result = PatternValidator::validate(&pattern).unwrap();
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn test_validation_empty_before_template() {
    let pattern = Pattern::new(
        "".to_string(),
        "bar()".to_string(),
        vec![],
        "typescript".to_string(),
        0.9,
    );

    let result = PatternValidator::validate(&pattern).unwrap();
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("before_template")));
}

#[test]
fn test_validation_low_confidence_warning() {
    let pattern = Pattern::new(
        "old($x)".to_string(),
        "new($x)".to_string(),
        vec![PatternVar {
            name: "$x".to_string(),
            node_type: None,
        }],
        "typescript".to_string(),
        0.2,
    );

    let result = PatternValidator::validate(&pattern).unwrap();
    assert!(result.is_valid); // Low confidence is a warning, not an error
    assert!(result.warnings.iter().any(|w| w.contains("confidence")));
}

#[test]
fn test_validation_variable_missing_from_before() {
    let pattern = Pattern::new(
        "foo(x)".to_string(),
        "bar($phantom)".to_string(),
        vec![PatternVar {
            name: "$phantom".to_string(),
            node_type: None,
        }],
        "typescript".to_string(),
        0.9,
    );

    let result = PatternValidator::validate(&pattern).unwrap();
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("$phantom")));
}

#[test]
fn test_validation_identical_templates_warning() {
    let pattern = Pattern::new(
        "same($x)".to_string(),
        "same($x)".to_string(),
        vec![PatternVar {
            name: "$x".to_string(),
            node_type: None,
        }],
        "typescript".to_string(),
        0.9,
    );

    let result = PatternValidator::validate(&pattern).unwrap();
    assert!(result.is_valid);
    assert!(result.warnings.iter().any(|w| w.contains("identical")));
}

// ---------------------------------------------------------------------------
// Fixture file loading
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
fn test_fixture_files_exist_and_differ() {
    // Verify that before/after fixture pairs exist and are non-empty.
    let fixture_dirs = &[
        "rename-function",
        "change-signature",
        "migrate-library",
        "replace-console",
    ];

    let base = workspace_root().join("tests").join("fixtures");

    for dir_name in fixture_dirs {
        let dir = base.join(dir_name);
        assert!(
            dir.exists(),
            "Fixture directory '{}' should exist at {}",
            dir_name,
            dir.display()
        );

        // Look for before/after pairs with any extension
        let before_files: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with("before."))
                    .unwrap_or(false)
            })
            .collect();
        assert!(
            !before_files.is_empty(),
            "Fixture {dir_name} should have a before.* file"
        );

        for before_entry in &before_files {
            let before_content =
                std::fs::read_to_string(before_entry.path()).unwrap();
            assert!(
                !before_content.trim().is_empty(),
                "{dir_name}/before file should not be empty"
            );

            // Derive the corresponding after file
            let before_name = before_entry.file_name();
            let after_name = before_name
                .to_str()
                .unwrap()
                .replace("before.", "after.");
            let after_path = dir.join(&after_name);
            assert!(
                after_path.exists(),
                "Fixture {dir_name} should have a corresponding {after_name}"
            );

            let after_content =
                std::fs::read_to_string(&after_path).unwrap();
            assert!(
                !after_content.trim().is_empty(),
                "{dir_name}/{after_name} should not be empty"
            );
            assert_ne!(
                before_content, after_content,
                "{dir_name}: before and after fixtures should differ"
            );
        }
    }
}
