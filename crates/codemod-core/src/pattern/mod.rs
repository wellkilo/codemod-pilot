//! Pattern inference, matching, and validation.
//!
//! This module contains the core algorithms for:
//!
//! - **Inference** ([`inferrer`]): Deriving transformation patterns from
//!   before/after example pairs by performing a structural diff of their ASTs.
//! - **Matching** ([`matcher`]): Finding occurrences of a pattern in arbitrary
//!   source code by comparing AST sub-trees.
//! - **Validation** ([`validator`]): Checking that an inferred pattern is
//!   well-formed and likely to produce correct transformations.

pub mod inferrer;
pub mod matcher;
pub mod validator;

pub use inferrer::PatternInferrer;
pub use matcher::PatternMatcher;
pub use validator::PatternValidator;

use serde::{Deserialize, Serialize};

/// Represents a pattern variable that matches any expression or identifier
/// at a particular position in the AST.
///
/// During inference, differing leaf nodes (identifiers, literals) between the
/// before and after examples are extracted as pattern variables. Each variable
/// has a unique name (e.g. `$id`, `$expr1`) and an optional constraint on the
/// tree-sitter node type it must match.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PatternVar {
    /// Variable name (e.g. `"$id"`, `"$expr"`).
    pub name: String,
    /// Optional constraint on the tree-sitter node type this variable must
    /// match (e.g. `"identifier"`, `"string_literal"`).
    pub node_type: Option<String>,
}

/// Represents an inferred transformation pattern.
///
/// A `Pattern` is the central data structure of the codemod engine. It captures
/// *what* to look for in source code (`before_template`) and *what* to replace
/// it with (`after_template`), using [`PatternVar`]s as placeholders for
/// varying sub-expressions.
///
/// ## Template syntax
///
/// Variables are written as `$name` inside the template strings. For example:
///
/// ```text
/// before: "println!($fmt, $arg)"
/// after:  "log::info!($fmt, $arg)"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// The "before" pattern template with `$variable` placeholders.
    pub before_template: String,
    /// The "after" pattern template with `$variable` placeholders.
    pub after_template: String,
    /// Variables extracted during inference.
    pub variables: Vec<PatternVar>,
    /// The source language identifier (e.g. `"rust"`, `"javascript"`).
    pub language: String,
    /// Confidence score of the inference in the range `[0.0, 1.0]`.
    ///
    /// Higher values indicate that the inferred pattern is more likely to be
    /// correct and generalizable.
    pub confidence: f64,
}

impl Pattern {
    /// Creates a new pattern with the given templates and variables.
    pub fn new(
        before_template: String,
        after_template: String,
        variables: Vec<PatternVar>,
        language: String,
        confidence: f64,
    ) -> Self {
        Self {
            before_template,
            after_template,
            variables,
            language,
            confidence,
        }
    }

    /// Returns `true` if this pattern contains at least one variable.
    pub fn has_variables(&self) -> bool {
        !self.variables.is_empty()
    }

    /// Returns `true` if the confidence score meets the given threshold.
    pub fn meets_confidence(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}
