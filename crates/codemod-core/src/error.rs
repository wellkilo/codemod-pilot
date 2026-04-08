//! Error types for the codemod-core crate.
//!
//! This module defines all error types used throughout the crate using
//! the `thiserror` derive macro for ergonomic error handling. A unified
//! [`CodemodError`] enum captures all possible failure modes, and a
//! convenience [`Result`] type alias is provided.

use thiserror::Error;

/// Unified error type for all codemod-core operations.
#[derive(Error, Debug)]
pub enum CodemodError {
    /// Pattern inference failed — the engine could not derive a
    /// transformation pattern from the provided before/after examples.
    #[error("Pattern inference error: {0}")]
    PatternInference(String),

    /// AST parsing failed — tree-sitter could not produce a valid
    /// syntax tree for the given source code.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Scanning error — something went wrong while walking the
    /// filesystem or matching patterns across files.
    #[error("Scan error: {0}")]
    Scan(String),

    /// Transformation error — the engine failed to apply a pattern
    /// transformation to a source file.
    #[error("Transform error: {0}")]
    Transform(String),

    /// Rule error — a codemod rule file could not be loaded, parsed,
    /// or failed validation.
    #[error("Rule error: {0}")]
    Rule(String),

    /// Pattern matching error.
    #[error("Pattern matching error: {0}")]
    Matching(String),

    /// File I/O error — a filesystem operation (read, write, create
    /// directory, etc.) failed.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML serialization/deserialization error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Language not supported.
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    /// Validation failed.
    #[error("Validation failed: {0}")]
    Validation(String),

    /// Catch-all for other errors.
    #[error("{0}")]
    Other(String),
}

/// Convenience result type for codemod-core operations.
pub type Result<T> = std::result::Result<T, CodemodError>;
