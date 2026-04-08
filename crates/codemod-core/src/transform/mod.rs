//! Transformation application, conflict detection, and rollback.
//!
//! This module orchestrates the process of applying pattern-based
//! transformations to source files:
//!
//! - [`applier`]: Applies variable substitutions to produce new source text.
//! - [`conflict`]: Detects overlapping or otherwise conflicting matches.
//! - [`rollback`]: Saves and restores original file content for undo support.

pub mod applier;
pub mod conflict;
pub mod rollback;

pub use applier::TransformApplier;
pub use conflict::ConflictResolver;
pub use rollback::RollbackManager;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result of a transformation applied to a single file.
///
/// Contains both the diff and the full original/new content so that the
/// caller can preview changes, persist them, or roll them back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    /// Path of the transformed file.
    pub file_path: PathBuf,
    /// Number of pattern matches found in the file.
    pub match_count: usize,
    /// Number of transformations successfully applied.
    pub applied_count: usize,
    /// Unified diff of the changes (suitable for display or patch files).
    pub diff: String,
    /// Original file content (for rollback).
    pub original_content: String,
    /// New file content after transformation.
    pub new_content: String,
}

impl TransformResult {
    /// Returns `true` if at least one transformation was applied.
    pub fn has_changes(&self) -> bool {
        self.applied_count > 0
    }
}
