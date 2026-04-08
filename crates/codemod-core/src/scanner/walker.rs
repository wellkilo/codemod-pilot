//! File system walker with glob filtering and `.gitignore` support.
//!
//! Uses [`walkdir`] for recursive directory traversal and [`globset`] for
//! include/exclude pattern matching.

use std::fs;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::WalkDir;

use crate::error::CodemodError;
use crate::language::LanguageAdapter;
use crate::scanner::ScanConfig;

/// Walks a directory tree and collects files eligible for scanning.
pub struct FileWalker {
    target_dir: PathBuf,
    include_set: Option<GlobSet>,
    exclude_set: Option<GlobSet>,
    respect_gitignore: bool,
    max_file_size: usize,
    gitignore_patterns: Vec<GlobSet>,
}

impl FileWalker {
    /// Build a new walker from a [`ScanConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`CodemodError::Scan`] if the target directory does not exist
    /// or a glob pattern is invalid.
    pub fn new(config: &ScanConfig) -> crate::Result<Self> {
        let target_dir = config.target_dir.clone();

        if !target_dir.is_dir() {
            return Err(CodemodError::Scan(format!(
                "Target directory does not exist or is not a directory: {}",
                target_dir.display()
            )));
        }

        let include_set = Self::build_globset(&config.include_patterns)?;
        let exclude_set = Self::build_globset(&config.exclude_patterns)?;

        let gitignore_patterns = if config.respect_gitignore {
            Self::load_gitignore(&target_dir)
        } else {
            Vec::new()
        };

        Ok(Self {
            target_dir,
            include_set,
            exclude_set,
            respect_gitignore: config.respect_gitignore,
            max_file_size: config.max_file_size,
            gitignore_patterns,
        })
    }

    /// Collect all files under the target directory that:
    /// - Have an extension supported by the language adapter
    /// - Match the include patterns (if any)
    /// - Do not match the exclude patterns
    /// - Are not ignored by `.gitignore` (if configured)
    /// - Are smaller than `max_file_size`
    pub fn collect_files(&self, language: &dyn LanguageAdapter) -> crate::Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(&self.target_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.is_hidden(e.path()))
        {
            let entry =
                entry.map_err(|e| CodemodError::Scan(format!("Error walking directory: {e}")))?;

            let path = entry.path();

            // Skip directories.
            if !entry.file_type().is_file() {
                continue;
            }

            // Language filter.
            if !language.supports_file(path) {
                continue;
            }

            // Include filter.
            if let Some(ref inc) = self.include_set {
                if !inc.is_match(path) {
                    continue;
                }
            }

            // Exclude filter.
            if let Some(ref exc) = self.exclude_set {
                if exc.is_match(path) {
                    continue;
                }
            }

            // Gitignore filter.
            if self.respect_gitignore && self.is_gitignored(path) {
                continue;
            }

            // File size filter.
            if let Ok(meta) = fs::metadata(path) {
                if meta.len() as usize > self.max_file_size {
                    log::debug!(
                        "Skipping large file ({} bytes): {}",
                        meta.len(),
                        path.display()
                    );
                    continue;
                }
            }

            files.push(path.to_path_buf());
        }

        Ok(files)
    }

    // -----------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------

    /// Build a [`GlobSet`] from a list of pattern strings.
    fn build_globset(patterns: &[String]) -> crate::Result<Option<GlobSet>> {
        if patterns.is_empty() {
            return Ok(None);
        }

        let mut builder = GlobSetBuilder::new();
        for pat in patterns {
            let glob = Glob::new(pat)
                .map_err(|e| CodemodError::Scan(format!("Invalid glob pattern '{pat}': {e}")))?;
            builder.add(glob);
        }

        let set = builder
            .build()
            .map_err(|e| CodemodError::Scan(format!("Failed to build glob set: {e}")))?;

        Ok(Some(set))
    }

    /// Load `.gitignore` from the target directory (if present).
    fn load_gitignore(target_dir: &Path) -> Vec<GlobSet> {
        let gitignore_path = target_dir.join(".gitignore");
        if !gitignore_path.is_file() {
            return Vec::new();
        }

        let content = match fs::read_to_string(&gitignore_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut sets = Vec::new();
        let mut builder = GlobSetBuilder::new();
        let mut has_patterns = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Attempt to compile the gitignore line as a glob.
            let pattern = if trimmed.ends_with('/') {
                format!("**/{}", trimmed.trim_end_matches('/'))
            } else {
                format!("**/{trimmed}")
            };

            if let Ok(glob) = Glob::new(&pattern) {
                builder.add(glob);
                has_patterns = true;
            }
        }

        if has_patterns {
            if let Ok(set) = builder.build() {
                sets.push(set);
            }
        }

        sets
    }

    /// Check if a path matches any loaded `.gitignore` pattern.
    fn is_gitignored(&self, path: &Path) -> bool {
        for set in &self.gitignore_patterns {
            if set.is_match(path) {
                return true;
            }
        }
        false
    }

    /// Check if a path component is hidden (starts with `.`).
    fn is_hidden(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_globset_empty() {
        let result = FileWalker::build_globset(&[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_globset_valid() {
        let patterns = vec!["**/*.rs".to_string(), "**/*.toml".to_string()];
        let result = FileWalker::build_globset(&patterns).unwrap();
        assert!(result.is_some());
        let set = result.unwrap();
        assert!(set.is_match("src/main.rs"));
        assert!(set.is_match("Cargo.toml"));
        assert!(!set.is_match("README.md"));
    }

    #[test]
    fn test_build_globset_invalid() {
        let patterns = vec!["[invalid".to_string()];
        let result = FileWalker::build_globset(&patterns);
        assert!(result.is_err());
    }
}
