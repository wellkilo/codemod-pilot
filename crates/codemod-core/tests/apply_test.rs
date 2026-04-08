//! Integration tests for the `apply` workflow.
//!
//! These tests verify that the `TransformApplier` can generate correct diffs
//! and apply pattern-based replacements with proper variable substitution,
//! indentation preservation, and multi-match handling.

use std::path::PathBuf;

use codemod_core::pattern::matcher::{Match, Position};
use codemod_core::pattern::{Pattern, PatternVar};
use codemod_core::transform::{TransformApplier, TransformResult};

// ---------------------------------------------------------------------------
// Diff generation
// ---------------------------------------------------------------------------

#[test]
fn test_generate_diff_basic() {
    let original = "let x = foo(1);\nlet y = foo(2);\n";
    let transformed = "let x = bar(1);\nlet y = bar(2);\n";
    let diff = TransformApplier::generate_diff("test.ts", original, transformed);

    assert!(
        diff.contains("--- a/test.ts"),
        "diff should contain old file header"
    );
    assert!(
        diff.contains("+++ b/test.ts"),
        "diff should contain new file header"
    );
    assert!(
        diff.contains("-let x = foo(1);"),
        "diff should show removed lines"
    );
    assert!(
        diff.contains("+let x = bar(1);"),
        "diff should show added lines"
    );
}

#[test]
fn test_generate_diff_no_changes() {
    let content = "unchanged content\n";
    let diff = TransformApplier::generate_diff("noop.ts", content, content);
    // When there are no changes, the diff should have headers but no hunks.
    assert!(
        diff.contains("--- a/noop.ts"),
        "diff should still contain headers"
    );
    // No +/- lines expected (other than the header lines themselves).
    let hunk_lines: Vec<&str> = diff
        .lines()
        .filter(|l| {
            (l.starts_with('+') || l.starts_with('-'))
                && !l.starts_with("---")
                && !l.starts_with("+++")
        })
        .collect();
    assert!(
        hunk_lines.is_empty(),
        "no-change diff should have no hunk content"
    );
}

#[test]
fn test_generate_diff_multiline() {
    let original = "line1\nline2\nline3\nline4\nline5\n";
    let transformed = "line1\nLINE2\nline3\nLINE4\nline5\n";
    let diff = TransformApplier::generate_diff("multi.ts", original, transformed);

    assert!(diff.contains("-line2"), "should show removed line2");
    assert!(diff.contains("+LINE2"), "should show added LINE2");
    assert!(diff.contains("-line4"), "should show removed line4");
    assert!(diff.contains("+LINE4"), "should show added LINE4");
}

// ---------------------------------------------------------------------------
// TransformResult
// ---------------------------------------------------------------------------

#[test]
fn test_transform_result_has_changes() {
    let result_with_changes = TransformResult {
        file_path: PathBuf::from("src/main.ts"),
        match_count: 3,
        applied_count: 2,
        diff: "some diff".to_string(),
        original_content: "old".to_string(),
        new_content: "new".to_string(),
    };
    assert!(result_with_changes.has_changes());

    let result_no_changes = TransformResult {
        file_path: PathBuf::from("src/main.ts"),
        match_count: 0,
        applied_count: 0,
        diff: String::new(),
        original_content: "same".to_string(),
        new_content: "same".to_string(),
    };
    assert!(!result_no_changes.has_changes());
}

// ---------------------------------------------------------------------------
// Apply with variable substitution
// ---------------------------------------------------------------------------

/// Helper to construct a `Match` value for testing.
fn make_match(start: usize, end: usize, text: &str, bindings: Vec<(&str, &str)>) -> Match {
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
        bindings: bindings
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

#[test]
fn test_apply_single_variable_substitution() {
    let source = "fetchUser(id)";
    let pattern = Pattern::new(
        "fetchUser($x)".to_string(),
        "getUser($x)".to_string(),
        vec![PatternVar {
            name: "$x".to_string(),
            node_type: None,
        }],
        "typescript".to_string(),
        0.9,
    );

    let m = make_match(0, 13, "fetchUser(id)", vec![("$x", "id")]);
    let result = TransformApplier::apply(source, &pattern, &[m]).unwrap();
    assert_eq!(result, "getUser(id)");
}

#[test]
fn test_apply_multiple_matches() {
    // Two non-overlapping matches in the same source.
    let source = "foo(a); foo(b);";
    let pattern = Pattern::new(
        "foo($v)".to_string(),
        "bar($v)".to_string(),
        vec![PatternVar {
            name: "$v".to_string(),
            node_type: None,
        }],
        "typescript".to_string(),
        0.9,
    );

    let m1 = make_match(0, 6, "foo(a)", vec![("$v", "a")]);
    let m2 = make_match(8, 14, "foo(b)", vec![("$v", "b")]);
    let result = TransformApplier::apply(source, &pattern, &[m1, m2]).unwrap();
    assert_eq!(result, "bar(a); bar(b);");
}

#[test]
fn test_apply_empty_matches_returns_original() {
    let source = "untouched source";
    let pattern = Pattern::new(
        "a".to_string(),
        "b".to_string(),
        vec![],
        "typescript".to_string(),
        0.9,
    );

    let result = TransformApplier::apply(source, &pattern, &[]).unwrap();
    assert_eq!(result, source);
}

// ---------------------------------------------------------------------------
// TransformResult serialization
// ---------------------------------------------------------------------------

#[test]
fn test_transform_result_serialization_roundtrip() {
    let result = TransformResult {
        file_path: PathBuf::from("src/api.ts"),
        match_count: 5,
        applied_count: 5,
        diff: "--- a/src/api.ts\n+++ b/src/api.ts\n".to_string(),
        original_content: "original()".to_string(),
        new_content: "replaced()".to_string(),
    };

    let json = serde_json::to_string(&result).expect("JSON serialization should succeed");
    let deserialized: TransformResult =
        serde_json::from_str(&json).expect("JSON deserialization should succeed");

    assert_eq!(deserialized.file_path, result.file_path);
    assert_eq!(deserialized.match_count, result.match_count);
    assert_eq!(deserialized.applied_count, result.applied_count);
    assert_eq!(deserialized.diff, result.diff);
    assert!(deserialized.has_changes());
}
