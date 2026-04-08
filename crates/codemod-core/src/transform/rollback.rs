//! Rollback management for undoing applied transformations.
//!
//! Before any file is modified, the original content is saved so that the
//! user can undo the changes later. Rollback data is stored as JSON files
//! inside a `.codemod-pilot/rollback/` directory under the project root.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CodemodError;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// An entry in the rollback history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackEntry {
    /// Path to the rollback JSON file.
    pub path: PathBuf,
    /// ISO-8601 timestamp of when the rollback was created.
    pub timestamp: String,
    /// Number of files affected.
    pub file_count: usize,
    /// Human-readable description.
    pub description: String,
}

/// Internal representation stored on disk.
#[derive(Debug, Serialize, Deserialize)]
struct RollbackData {
    timestamp: String,
    description: String,
    files: Vec<RollbackFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RollbackFile {
    path: PathBuf,
    original_content: String,
}

// ---------------------------------------------------------------------------
// RollbackManager
// ---------------------------------------------------------------------------

/// Manages rollback data for transformation undo support.
///
/// Rollback files are stored as JSON under `<project_root>/.codemod-pilot/rollback/`.
pub struct RollbackManager {
    rollback_dir: PathBuf,
}

impl RollbackManager {
    /// Create a new `RollbackManager` for the given project root.
    ///
    /// The rollback directory is created lazily on the first [`Self::save_rollback`]
    /// call.
    pub fn new(project_root: &Path) -> crate::Result<Self> {
        let rollback_dir = project_root.join(".codemod-pilot").join("rollback");
        Ok(Self { rollback_dir })
    }

    /// Save a rollback entry for the given transformation results.
    ///
    /// Returns the path to the saved rollback JSON file.
    pub fn save_rollback(
        &self,
        results: &[super::TransformResult],
    ) -> crate::Result<PathBuf> {
        // Ensure directory exists.
        fs::create_dir_all(&self.rollback_dir)?;

        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
        let iso_timestamp = now.to_rfc3339();

        let data = RollbackData {
            timestamp: iso_timestamp.clone(),
            description: format!(
                "Rollback for {} file(s) transformed at {}",
                results.len(),
                iso_timestamp
            ),
            files: results
                .iter()
                .map(|r| RollbackFile {
                    path: r.file_path.clone(),
                    original_content: r.original_content.clone(),
                })
                .collect(),
        };

        let filename = format!("rollback_{timestamp}.json");
        let file_path = self.rollback_dir.join(&filename);

        let json = serde_json::to_string_pretty(&data).map_err(|e| {
            CodemodError::Transform(format!("Failed to serialize rollback data: {e}"))
        })?;

        fs::write(&file_path, json)?;

        log::info!("Saved rollback to {}", file_path.display());
        Ok(file_path)
    }

    /// Apply a rollback from a saved JSON file, restoring original file
    /// contents.
    ///
    /// Returns the number of files restored.
    pub fn apply_rollback(&self, patch_path: &Path) -> crate::Result<usize> {
        let content = fs::read_to_string(patch_path)?;
        let data: RollbackData = serde_json::from_str(&content).map_err(|e| {
            CodemodError::Transform(format!("Failed to parse rollback file: {e}"))
        })?;

        let mut restored = 0usize;
        for file in &data.files {
            if file.path.exists() {
                fs::write(&file.path, &file.original_content)?;
                restored += 1;
                log::info!("Restored {}", file.path.display());
            } else {
                log::warn!(
                    "Skipping {} — file does not exist",
                    file.path.display()
                );
            }
        }

        Ok(restored)
    }

    /// List all available rollback entries, most recent first.
    pub fn list_rollbacks(&self) -> crate::Result<Vec<RollbackEntry>> {
        if !self.rollback_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&self.rollback_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            match self.read_rollback_entry(&path) {
                Ok(re) => entries.push(re),
                Err(e) => {
                    log::warn!(
                        "Skipping malformed rollback file {}: {e}",
                        path.display()
                    );
                }
            }
        }

        // Sort by timestamp descending (newest first).
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(entries)
    }

    // -----------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------

    /// Read a single rollback file and produce a [`RollbackEntry`].
    fn read_rollback_entry(&self, path: &Path) -> crate::Result<RollbackEntry> {
        let content = fs::read_to_string(path)?;
        let data: RollbackData = serde_json::from_str(&content).map_err(|e| {
            CodemodError::Transform(format!("Failed to parse rollback file: {e}"))
        })?;

        Ok(RollbackEntry {
            path: path.to_path_buf(),
            timestamp: data.timestamp,
            file_count: data.files.len(),
            description: data.description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::TransformResult;

    #[test]
    fn test_rollback_roundtrip() {
        let tmp = std::env::temp_dir().join("codemod_test_rollback");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // Create a dummy file to transform.
        let test_file = tmp.join("test.rs");
        fs::write(&test_file, "original content").unwrap();

        let manager = RollbackManager::new(&tmp).unwrap();

        let results = vec![TransformResult {
            file_path: test_file.clone(),
            match_count: 1,
            applied_count: 1,
            diff: String::new(),
            original_content: "original content".into(),
            new_content: "new content".into(),
        }];

        // Simulate applying the transform.
        fs::write(&test_file, "new content").unwrap();

        // Save rollback.
        let patch_path = manager.save_rollback(&results).unwrap();
        assert!(patch_path.exists());

        // List rollbacks.
        let entries = manager.list_rollbacks().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_count, 1);

        // Apply rollback.
        let restored = manager.apply_rollback(&patch_path).unwrap();
        assert_eq!(restored, 1);
        assert_eq!(fs::read_to_string(&test_file).unwrap(), "original content");

        // Cleanup.
        let _ = fs::remove_dir_all(&tmp);
    }
}
