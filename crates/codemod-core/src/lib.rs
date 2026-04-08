//! # codemod-core
//!
//! Core engine for codemod-pilot: pattern inference, matching, and transformation.
//!
//! This crate provides the fundamental building blocks for:
//! - Inferring code transformation patterns from before/after examples
//! - Scanning codebases for pattern matches
//! - Applying transformations with preview and rollback support
//! - Managing reusable codemod rules
//!
//! ## Architecture
//!
//! - [`pattern`]: Pattern inference, matching, and validation
//! - [`transform`]: Transformation application, conflict detection, and rollback
//! - [`scanner`]: Codebase scanning with parallel file processing
//! - [`rule`]: Codemod rule management and serialization
//! - [`language`]: Language adapter trait for tree-sitter integration
//! - [`error`]: Error types and result aliases

pub mod error;
pub mod language;
pub mod pattern;
pub mod rule;
pub mod scanner;
pub mod transform;

// Re-export core types at crate root for ergonomic access.
pub use error::{CodemodError, Result};
pub use language::LanguageAdapter;
pub use pattern::{Pattern, PatternInferrer, PatternMatcher, PatternVar};
pub use pattern::validator::{PatternValidator, ValidationResult};
pub use rule::{CodemodRule, RuleConfig};
pub use rule::builtin::BuiltinRules;
pub use rule::schema::RulePattern;
pub use scanner::{ScanConfig, ScanMatch, ScanResult, Scanner};
pub use transform::{TransformApplier, TransformResult};
pub use transform::conflict::ConflictResolver;
pub use transform::rollback::{RollbackEntry, RollbackManager};
