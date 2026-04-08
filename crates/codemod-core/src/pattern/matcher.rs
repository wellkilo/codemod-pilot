//! Pattern matching engine.
//!
//! Given a [`Pattern`](super::Pattern) and a source file, this module finds
//! all locations where the `before_template` matches. Each match records the
//! byte range, position, matched text, and variable bindings so that the
//! [`TransformApplier`](crate::transform::applier::TransformApplier) can
//! later produce the replacement text.

use std::collections::HashMap;

use tree_sitter::{Node, Parser, Tree};

use super::{Pattern, PatternVar};
use crate::error::CodemodError;
use crate::language::LanguageAdapter;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single match found in source code.
#[derive(Debug, Clone)]
pub struct Match {
    /// Byte offset range in the source.
    pub byte_range: std::ops::Range<usize>,
    /// Line/column start position (0-indexed).
    pub start_position: Position,
    /// Line/column end position (0-indexed).
    pub end_position: Position,
    /// The matched source text.
    pub matched_text: String,
    /// Captured variable bindings: variable name -> matched text.
    pub bindings: HashMap<String, String>,
}

/// A line/column position in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// 0-indexed line number.
    pub line: usize,
    /// 0-indexed column (byte offset within the line).
    pub column: usize,
}

// ---------------------------------------------------------------------------
// PatternMatcher
// ---------------------------------------------------------------------------

/// Finds occurrences of a [`Pattern`] in source code.
///
/// The matcher parses the source into an AST and then walks every subtree,
/// attempting to unify it with the `before_template` AST. When a subtree
/// matches — i.e. has the same structure and all non-variable leaves agree —
/// it records a [`Match`] with the captured variable bindings.
pub struct PatternMatcher {
    language: Box<dyn LanguageAdapter>,
}

impl PatternMatcher {
    /// Create a new matcher backed by the given language adapter.
    pub fn new(language: Box<dyn LanguageAdapter>) -> Self {
        Self { language }
    }

    /// Find all matches of `pattern` in the given `source` code.
    ///
    /// # Errors
    ///
    /// Returns [`CodemodError::Parse`] if the source or the pattern template
    /// cannot be parsed.
    pub fn find_matches(&self, source: &str, pattern: &Pattern) -> crate::Result<Vec<Match>> {
        // 1. Parse the before_template to get its AST shape.
        let template_tree = self.parse(&pattern.before_template)?;
        // 2. Parse the target source.
        let source_tree = self.parse(source)?;

        let template_root = template_tree.root_node();
        let source_root = source_tree.root_node();

        let mut matches = Vec::new();

        // 3. Walk every node in the source tree and try to match.
        self.walk_and_match(
            source_root,
            template_root,
            source,
            &pattern.before_template,
            &pattern.variables,
            &mut matches,
        );

        Ok(matches)
    }

    // -----------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------

    /// Parse source text into a tree-sitter [`Tree`].
    fn parse(&self, source: &str) -> crate::Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&self.language.language())
            .map_err(|e| CodemodError::Parse(format!("Failed to set language: {e}")))?;
        parser
            .parse(source, None)
            .ok_or_else(|| CodemodError::Parse("tree-sitter returned no tree".into()))
    }

    /// Recursively walk the source tree. At every named node, attempt to
    /// unify with the template root. If the node does not match, recurse
    /// into its children.
    fn walk_and_match(
        &self,
        source_node: Node,
        template_root: Node,
        source: &str,
        template_source: &str,
        variables: &[PatternVar],
        matches: &mut Vec<Match>,
    ) {
        // Try matching the current source node against the template root.
        let mut bindings = HashMap::new();
        if self.try_match(
            source_node,
            template_root,
            source,
            template_source,
            variables,
            &mut bindings,
        ) {
            let start = source_node.start_position();
            let end = source_node.end_position();
            matches.push(Match {
                byte_range: source_node.byte_range(),
                start_position: Position {
                    line: start.row,
                    column: start.column,
                },
                end_position: Position {
                    line: end.row,
                    column: end.column,
                },
                matched_text: source[source_node.byte_range()].to_string(),
                bindings,
            });
            // Do not recurse into children of a matched node to avoid
            // overlapping matches.
            return;
        }

        // No match — recurse into named children.
        let child_count = source_node.named_child_count();
        for i in 0..child_count {
            if let Some(child) = source_node.named_child(i) {
                self.walk_and_match(
                    child,
                    template_root,
                    source,
                    template_source,
                    variables,
                    matches,
                );
            }
        }
    }

    /// Attempt to unify `source_node` with `template_node`.
    ///
    /// Returns `true` if the two trees have the same shape, all literal
    /// leaves agree, and variable positions are captured in `bindings`.
    fn try_match(
        &self,
        source_node: Node,
        template_node: Node,
        source: &str,
        template_source: &str,
        variables: &[PatternVar],
        bindings: &mut HashMap<String, String>,
    ) -> bool {
        let template_text = &template_source[template_node.byte_range()];

        // Check if the template text at this position is a variable placeholder.
        if let Some(var) = self.is_variable_placeholder(template_text, variables) {
            let source_text = source[source_node.byte_range()].to_string();
            // If this variable was already bound, the binding must be consistent.
            if let Some(existing) = bindings.get(&var.name) {
                return *existing == source_text;
            }
            bindings.insert(var.name.clone(), source_text);
            return true;
        }

        // Kinds must agree.
        if source_node.kind() != template_node.kind() {
            return false;
        }

        // Leaf nodes: text must match exactly.
        if template_node.named_child_count() == 0 && source_node.named_child_count() == 0 {
            let s_text = &source[source_node.byte_range()];
            return s_text == template_text;
        }

        // Structural nodes: children count must agree and each child must match.
        let t_count = template_node.named_child_count();
        let s_count = source_node.named_child_count();
        if t_count != s_count {
            return false;
        }

        for i in 0..t_count {
            let t_child = match template_node.named_child(i) {
                Some(c) => c,
                None => return false,
            };
            let s_child = match source_node.named_child(i) {
                Some(c) => c,
                None => return false,
            };
            if !self.try_match(
                s_child,
                t_child,
                source,
                template_source,
                variables,
                bindings,
            ) {
                return false;
            }
        }

        true
    }

    /// Check whether `text` matches a known pattern variable placeholder
    /// (e.g. `"$var1"`).
    fn is_variable_placeholder<'a>(
        &self,
        text: &str,
        variables: &'a [PatternVar],
    ) -> Option<&'a PatternVar> {
        variables.iter().find(|v| v.name == text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_equality() {
        let a = Position { line: 1, column: 4 };
        let b = Position { line: 1, column: 4 };
        assert_eq!(a, b);
    }
}
