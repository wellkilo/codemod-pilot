//! Conflict detection for overlapping or incompatible matches.
//!
//! When multiple pattern matches are found in a single file their byte ranges
//! may overlap. Applying overlapping replacements can corrupt the source.
//! This module detects such conflicts *before* any transformation is applied
//! so that the user or the engine can decide how to resolve them.

use crate::pattern::matcher::Match;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Describes a conflict between two or more matches.
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Index of the match (in the original matches slice) that caused the
    /// conflict.
    pub match_index: usize,
    /// Index of the other match that this one overlaps with.
    pub other_index: usize,
    /// Human-readable description of the conflict.
    pub description: String,
    /// Suggested resolution strategies.
    pub suggestions: Vec<String>,
}

/// How to resolve a detected conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Apply the conflicting match anyway.
    Apply,
    /// Skip the conflicting match.
    Skip,
    /// Apply all remaining matches regardless of conflicts.
    ApplyAll,
    /// Skip all remaining conflicting matches.
    SkipAll,
}

// ---------------------------------------------------------------------------
// ConflictResolver
// ---------------------------------------------------------------------------

/// Detects overlapping or otherwise conflicting matches within a single file.
pub struct ConflictResolver;

impl ConflictResolver {
    /// Detect conflicts among a set of matches.
    ///
    /// Two matches conflict if their byte ranges overlap. The returned
    /// [`Conflict`] entries reference the *indices* into the input `matches`
    /// slice.
    pub fn detect_conflicts(matches: &[Match]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        if matches.len() < 2 {
            return conflicts;
        }

        // Build a list of (index, start, end) sorted by start offset.
        let mut sorted: Vec<(usize, usize, usize)> = matches
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.byte_range.start, m.byte_range.end))
            .collect();
        sorted.sort_by_key(|&(_, start, _)| start);

        // Sweep-line: check each pair of adjacent sorted entries for overlap.
        for window in sorted.windows(2) {
            let (idx_a, _start_a, end_a) = window[0];
            let (idx_b, start_b, _end_b) = window[1];

            if start_b < end_a {
                conflicts.push(Conflict {
                    match_index: idx_b,
                    other_index: idx_a,
                    description: format!(
                        "Match at byte offset {} overlaps with match at byte offset {} (overlap starts at byte {})",
                        matches[idx_b].byte_range.start,
                        matches[idx_a].byte_range.start,
                        start_b,
                    ),
                    suggestions: vec![
                        "Skip the later match".into(),
                        "Skip the earlier match".into(),
                        "Manually resolve the overlap".into(),
                    ],
                });
            }
        }

        conflicts
    }

    /// Filter out conflicting matches, keeping only non-overlapping ones.
    ///
    /// Uses a greedy strategy: iterate matches sorted by start offset and
    /// keep a match only if it does not overlap with the previously kept one.
    pub fn resolve_greedy(matches: &[Match]) -> Vec<usize> {
        if matches.is_empty() {
            return Vec::new();
        }

        let mut indexed: Vec<(usize, usize, usize)> = matches
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.byte_range.start, m.byte_range.end))
            .collect();
        indexed.sort_by_key(|&(_, start, _)| start);

        let mut kept = Vec::new();
        let mut last_end: usize = 0;

        for (idx, start, end) in indexed {
            if start >= last_end {
                kept.push(idx);
                last_end = end;
            }
        }

        kept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::matcher::{Match, Position};
    use std::collections::HashMap;

    fn make_match(start: usize, end: usize) -> Match {
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
            matched_text: String::new(),
            bindings: HashMap::new(),
        }
    }

    #[test]
    fn test_no_conflicts() {
        let matches = vec![make_match(0, 5), make_match(10, 15)];
        let conflicts = ConflictResolver::detect_conflicts(&matches);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_overlapping_conflicts() {
        let matches = vec![make_match(0, 10), make_match(5, 15)];
        let conflicts = ConflictResolver::detect_conflicts(&matches);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].match_index, 1);
        assert_eq!(conflicts[0].other_index, 0);
    }

    #[test]
    fn test_adjacent_no_conflict() {
        let matches = vec![make_match(0, 5), make_match(5, 10)];
        let conflicts = ConflictResolver::detect_conflicts(&matches);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_resolve_greedy() {
        let matches = vec![make_match(0, 10), make_match(5, 15), make_match(15, 20)];
        let kept = ConflictResolver::resolve_greedy(&matches);
        assert_eq!(kept, vec![0, 2]);
    }

    #[test]
    fn test_single_match() {
        let matches = vec![make_match(0, 5)];
        let conflicts = ConflictResolver::detect_conflicts(&matches);
        assert!(conflicts.is_empty());
        let kept = ConflictResolver::resolve_greedy(&matches);
        assert_eq!(kept, vec![0]);
    }
}
