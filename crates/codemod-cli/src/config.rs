//! Session configuration management.
//!
//! Manages the `.codemod-pilot/session.json` file which stores
//! the current learning session state.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The session directory name (created in the project root).
pub const SESSION_DIR: &str = ".codemod-pilot";
/// The session state file name.
const SESSION_FILE: &str = "session.json";

/// Persistent state for the current codemod-pilot session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// The inferred pattern (if any).
    pub pattern: Option<codemod_core::pattern::Pattern>,
    /// The last directory that was scanned.
    pub last_scan_target: Option<String>,
    /// The language being used.
    pub language: String,
    /// When this session was created.
    pub created_at: String,
}

impl SessionState {
    /// Load the session state from the project root directory.
    /// Returns `Ok(None)` if no session file exists.
    pub fn load(project_root: &Path) -> Result<Option<Self>> {
        let path = session_path(project_root);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read session file: {}", path.display()))?;
        let state: SessionState =
            serde_json::from_str(&content).with_context(|| "Failed to parse session file")?;
        Ok(Some(state))
    }

    /// Save the session state to the project root directory.
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let dir = project_root.join(SESSION_DIR);
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create session directory: {}", dir.display()))?;

        let path = session_path(project_root);
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize session state")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write session file: {}", path.display()))?;
        Ok(())
    }

    /// Clear (remove) the session state file.
    #[allow(dead_code)]
    pub fn clear(project_root: &Path) -> Result<()> {
        let path = session_path(project_root);
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to remove session file: {}", path.display()))?;
        }
        Ok(())
    }
}

/// Get the full path to the session file.
fn session_path(project_root: &Path) -> PathBuf {
    project_root.join(SESSION_DIR).join(SESSION_FILE)
}
