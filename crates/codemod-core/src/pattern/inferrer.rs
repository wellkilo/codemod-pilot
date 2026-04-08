//! Pattern inference engine.
//!
//! This module implements the core algorithm that derives a transformation
//! pattern from one or more before/after example pairs. The process works as
//! follows:
//!
//! 1. Parse both the *before* and *after* code into tree-sitter ASTs.
//! 2. Walk the two trees in parallel, performing a **structural diff**.
//! 3. Where leaf nodes differ (identifiers, literals, etc.), extract them as
//!    [`PatternVar`](super::PatternVar)s.
//! 4. Where the structure is identical, preserve it verbatim in the template.
//! 5. Assemble the `before_template` and `after_template` and compute a
//!    confidence score.

use std::collections::HashMap;

use tree_sitter::{Node, Parser, Tree};

use super::{Pattern, PatternVar};
use crate::error::CodemodError;
use crate::language::LanguageAdapter;

// ---------------------------------------------------------------------------
// Internal helper: a flattened representation of AST nodes for diffing
// ---------------------------------------------------------------------------

/// A lightweight, owned snapshot of a tree-sitter node used during diffing.
#[derive(Debug, Clone)]
struct NodeSnapshot {
    /// tree-sitter node kind (e.g. `"identifier"`, `"call_expression"`).
    kind: String,
    /// The source text spanned by this node.
    text: String,
    /// Whether this is a named node.
    #[allow(dead_code)]
    is_named: bool,
    /// Indices of child snapshots in the owning `Vec<NodeSnapshot>`.
    children: Vec<usize>,
    /// Depth in the tree (root = 0).
    #[allow(dead_code)]
    depth: usize,
}

/// Describes how two nodes relate during the structural diff.
#[derive(Debug)]
enum DiffKind {
    /// Nodes are structurally and textually identical.
    Same,
    /// Nodes have the same kind but different text — a variable candidate.
    Changed {
        before_text: String,
        after_text: String,
        node_kind: String,
    },
    /// The tree structures diverge in a way that cannot be captured by a
    /// simple variable substitution.
    Structural,
}

/// Indicates which side of the diff we are generating a template for.
#[derive(Debug, Clone, Copy)]
enum TemplateSource {
    Before,
    After,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// The pattern inference engine.
///
/// Given one or more before/after example pairs, the inferrer derives a
/// [`Pattern`] that captures the transformation.
pub struct PatternInferrer {
    language: Box<dyn LanguageAdapter>,
}

impl PatternInferrer {
    /// Creates a new inferrer backed by the given language adapter.
    pub fn new(language: Box<dyn LanguageAdapter>) -> Self {
        Self { language }
    }

    // -----------------------------------------------------------------
    // Public entry points
    // -----------------------------------------------------------------

    /// Infer a pattern from a single before/after example pair.
    ///
    /// # Errors
    ///
    /// Returns [`CodemodError::PatternInference`] if the ASTs cannot be
    /// compared or no meaningful pattern can be derived.
    pub fn infer_from_example(&self, before: &str, after: &str) -> crate::Result<Pattern> {
        let before_tree = self.parse(before)?;
        let after_tree = self.parse(after)?;

        // Flatten trees into snapshot vectors for easier traversal.
        let before_snaps = Self::flatten_tree(&before_tree, before);
        let after_snaps = Self::flatten_tree(&after_tree, after);

        // Structural diff starting from the root nodes.
        let mut var_counter: usize = 0;
        let mut variables: Vec<PatternVar> = Vec::new();
        // Map from (before_text, after_text) -> variable name, used to
        // ensure the *same* textual change gets the *same* variable.
        let mut var_map: HashMap<(String, String), String> = HashMap::new();

        let before_template = self.build_template(
            &before_snaps,
            &after_snaps,
            0,
            0,
            before,
            TemplateSource::Before,
            &mut var_counter,
            &mut variables,
            &mut var_map,
        );

        let after_template = self.build_template(
            &before_snaps,
            &after_snaps,
            0,
            0,
            after,
            TemplateSource::After,
            &mut var_counter,
            &mut variables,
            &mut var_map,
        );

        let confidence = Self::compute_confidence(&variables, &before_template, &after_template);

        let pattern = Pattern::new(
            before_template,
            after_template,
            variables,
            self.language.name().to_string(),
            confidence,
        );

        Ok(pattern)
    }

    /// Infer a pattern from multiple before/after example pairs.
    ///
    /// The algorithm infers a pattern from the first pair and then validates
    /// it against subsequent pairs, refining the confidence score.
    ///
    /// # Errors
    ///
    /// Returns [`CodemodError::PatternInference`] if no consistent pattern
    /// can be derived across the supplied examples.
    pub fn infer_from_examples(&self, examples: &[(String, String)]) -> crate::Result<Pattern> {
        if examples.is_empty() {
            return Err(CodemodError::PatternInference(
                "At least one example pair is required".into(),
            ));
        }

        // Infer from the first pair.
        let mut pattern = self.infer_from_example(&examples[0].0, &examples[0].1)?;

        if examples.len() == 1 {
            return Ok(pattern);
        }

        // Cross-validate against subsequent pairs and adjust confidence.
        let mut confirmed: usize = 1;
        for (before, after) in &examples[1..] {
            match self.infer_from_example(before, after) {
                Ok(other) => {
                    if Self::patterns_compatible(&pattern, &other) {
                        confirmed += 1;
                    } else {
                        log::warn!("Example pair produced an incompatible pattern — skipping");
                    }
                }
                Err(e) => {
                    log::warn!("Failed to infer from example pair: {e}");
                }
            }
        }

        // Confidence is boosted proportionally to the number of confirmed
        // examples.
        let cross_factor = confirmed as f64 / examples.len() as f64;
        pattern.confidence = (pattern.confidence * 0.6 + cross_factor * 0.4).min(1.0);

        Ok(pattern)
    }

    // -----------------------------------------------------------------
    // Parsing helpers
    // -----------------------------------------------------------------

    /// Parse the given source code into a tree-sitter [`Tree`].
    fn parse(&self, source: &str) -> crate::Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&self.language.language())
            .map_err(|e| CodemodError::Parse(format!("Failed to set language: {e}")))?;
        parser
            .parse(source, None)
            .ok_or_else(|| CodemodError::Parse("tree-sitter returned no tree".into()))
    }

    // -----------------------------------------------------------------
    // Tree flattening
    // -----------------------------------------------------------------

    /// Flatten a tree-sitter tree into a `Vec<NodeSnapshot>`. Index `0` is
    /// always the root node.
    fn flatten_tree(tree: &Tree, source: &str) -> Vec<NodeSnapshot> {
        let mut snaps = Vec::new();
        Self::flatten_node(tree.root_node(), source, &mut snaps, 0);
        snaps
    }

    /// Recursively flatten a node and its children.
    fn flatten_node(
        node: Node,
        source: &str,
        snaps: &mut Vec<NodeSnapshot>,
        depth: usize,
    ) -> usize {
        let idx = snaps.len();
        // Push a placeholder that we will fill in with child indices.
        snaps.push(NodeSnapshot {
            kind: node.kind().to_string(),
            text: source[node.byte_range()].to_string(),
            is_named: node.is_named(),
            children: Vec::new(),
            depth,
        });

        let mut child_indices = Vec::new();
        let child_count = node.named_child_count();
        for i in 0..child_count {
            if let Some(child) = node.named_child(i) {
                let child_idx = Self::flatten_node(child, source, snaps, depth + 1);
                child_indices.push(child_idx);
            }
        }

        snaps[idx].children = child_indices;
        idx
    }

    // -----------------------------------------------------------------
    // Structural diff / template building
    // -----------------------------------------------------------------

    /// Build a template string for one side of the diff.
    ///
    /// Walks both snapshot trees in parallel. Where the trees agree, the
    /// original text is emitted verbatim. Where they disagree at a leaf
    /// level, a `$variable` placeholder is emitted.
    #[allow(clippy::too_many_arguments)]
    fn build_template(
        &self,
        before_snaps: &[NodeSnapshot],
        after_snaps: &[NodeSnapshot],
        before_idx: usize,
        after_idx: usize,
        source: &str,
        side: TemplateSource,
        var_counter: &mut usize,
        variables: &mut Vec<PatternVar>,
        var_map: &mut HashMap<(String, String), String>,
    ) -> String {
        // Guard against out-of-bounds.
        if before_idx >= before_snaps.len() || after_idx >= after_snaps.len() {
            return source.to_string();
        }

        let b_snap = &before_snaps[before_idx];
        let a_snap = &after_snaps[after_idx];

        match self.diff_nodes(b_snap, a_snap) {
            DiffKind::Same => {
                // Trees agree — return original text from the requested side.
                match side {
                    TemplateSource::Before => b_snap.text.clone(),
                    TemplateSource::After => a_snap.text.clone(),
                }
            }
            DiffKind::Changed {
                before_text,
                after_text,
                node_kind,
            } => {
                // Leaf-level change — introduce or reuse a variable.
                let key = (before_text.clone(), after_text.clone());
                let var_name = if let Some(name) = var_map.get(&key) {
                    name.clone()
                } else {
                    *var_counter += 1;
                    let name = format!("$var{}", *var_counter);
                    var_map.insert(key, name.clone());
                    variables.push(PatternVar {
                        name: name.clone(),
                        node_type: Some(node_kind),
                    });
                    name
                };
                var_name
            }
            DiffKind::Structural => {
                // The structures diverge. If there are children we can still
                // try to walk them in parallel; otherwise fall back to the
                // entire text as a variable.
                if b_snap.children.is_empty() && a_snap.children.is_empty() {
                    let key = (b_snap.text.clone(), a_snap.text.clone());
                    let var_name = if let Some(name) = var_map.get(&key) {
                        name.clone()
                    } else {
                        *var_counter += 1;
                        let name = format!("$var{}", *var_counter);
                        var_map.insert(key, name.clone());
                        variables.push(PatternVar {
                            name: name.clone(),
                            node_type: Some(b_snap.kind.clone()),
                        });
                        name
                    };
                    return var_name;
                }

                // Attempt parallel walk of children and reconstruct the
                // template using the source text as a scaffold.
                self.build_template_from_children(
                    before_snaps,
                    after_snaps,
                    b_snap,
                    a_snap,
                    source,
                    side,
                    var_counter,
                    variables,
                    var_map,
                )
            }
        }
    }

    /// Reconstruct a template by walking the children of two differing
    /// structural nodes in parallel and stitching the results into the
    /// original source text.
    #[allow(clippy::too_many_arguments)]
    fn build_template_from_children(
        &self,
        before_snaps: &[NodeSnapshot],
        after_snaps: &[NodeSnapshot],
        b_snap: &NodeSnapshot,
        a_snap: &NodeSnapshot,
        _source: &str,
        side: TemplateSource,
        var_counter: &mut usize,
        variables: &mut Vec<PatternVar>,
        var_map: &mut HashMap<(String, String), String>,
    ) -> String {
        let base_snap = match side {
            TemplateSource::Before => b_snap,
            TemplateSource::After => a_snap,
        };
        let base_text = &base_snap.text;

        // Walk the minimum number of children present on both sides.
        let min_children = b_snap.children.len().min(a_snap.children.len());
        if min_children == 0 {
            return base_text.clone();
        }

        let mut result = base_text.clone();
        // We replace child texts from last to first to preserve byte offsets
        // within `result`.
        let mut replacements: Vec<(String, String)> = Vec::new();

        for i in 0..min_children {
            let b_child_idx = b_snap.children[i];
            let a_child_idx = a_snap.children[i];

            let child_template = self.build_template(
                before_snaps,
                after_snaps,
                b_child_idx,
                a_child_idx,
                match side {
                    TemplateSource::Before => &before_snaps[b_child_idx].text,
                    TemplateSource::After => &after_snaps[a_child_idx].text,
                },
                side,
                var_counter,
                variables,
                var_map,
            );

            let original_child_text = match side {
                TemplateSource::Before => &before_snaps[b_child_idx].text,
                TemplateSource::After => &after_snaps[a_child_idx].text,
            };

            if child_template != *original_child_text {
                replacements.push((original_child_text.clone(), child_template));
            }
        }

        // Apply replacements. We do a simple first-occurrence replacement for
        // each pair. For identical child texts this is a best-effort heuristic.
        for (old, new) in replacements.iter().rev() {
            if let Some(pos) = result.rfind(old.as_str()) {
                result.replace_range(pos..pos + old.len(), new);
            }
        }

        result
    }

    /// Compare two node snapshots and classify the relationship.
    fn diff_nodes(&self, before: &NodeSnapshot, after: &NodeSnapshot) -> DiffKind {
        // Exact text match — trivially the same.
        if before.text == after.text {
            return DiffKind::Same;
        }

        // Both are leaf nodes of the same kind — treat as a variable change.
        if before.children.is_empty() && after.children.is_empty() && before.kind == after.kind {
            return DiffKind::Changed {
                before_text: before.text.clone(),
                after_text: after.text.clone(),
                node_kind: before.kind.clone(),
            };
        }

        // Same node kind with children — structural diff is needed.
        if before.kind == after.kind {
            return DiffKind::Structural;
        }

        // Completely different kinds — treat as a structural change.
        DiffKind::Structural
    }

    // -----------------------------------------------------------------
    // Confidence computation
    // -----------------------------------------------------------------

    /// Compute a heuristic confidence score for the inferred pattern.
    ///
    /// Factors considered:
    /// - Number of variables (fewer -> higher confidence).
    /// - Ratio of template text that is *fixed* vs. *variable*.
    fn compute_confidence(
        variables: &[PatternVar],
        before_template: &str,
        _after_template: &str,
    ) -> f64 {
        if before_template.is_empty() {
            return 0.0;
        }

        let total_len = before_template.len() as f64;
        let var_len: f64 = variables.iter().map(|v| v.name.len() as f64).sum();

        // Fixed ratio: how much of the template is literal code.
        let fixed_ratio = 1.0 - (var_len / total_len).min(1.0);

        // Penalty for too many variables.
        let var_penalty = 1.0 / (1.0 + variables.len() as f64 * 0.15);

        (fixed_ratio * 0.7 + var_penalty * 0.3).clamp(0.0, 1.0)
    }

    // -----------------------------------------------------------------
    // Cross-example compatibility check
    // -----------------------------------------------------------------

    /// Check whether two independently-inferred patterns are "compatible",
    /// meaning they have the same variable count and the same fixed template
    /// skeleton.
    fn patterns_compatible(a: &Pattern, b: &Pattern) -> bool {
        // Same number of variables is a strong signal.
        if a.variables.len() != b.variables.len() {
            return false;
        }

        // Strip variable placeholders and compare the skeletons.
        let skeleton_a = Self::strip_variables(&a.before_template);
        let skeleton_b = Self::strip_variables(&b.before_template);

        skeleton_a == skeleton_b
    }

    /// Replace all `$varN` placeholders with a fixed sentinel so that two
    /// templates can be compared structurally.
    fn strip_variables(template: &str) -> String {
        let mut result = String::with_capacity(template.len());
        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '$' {
                // Skip the variable name.
                result.push_str("$$");
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_variables() {
        let input = "foo($var1, $var2)";
        let stripped = PatternInferrer::strip_variables(input);
        assert_eq!(stripped, "foo($$, $$)");
    }

    #[test]
    fn test_compute_confidence_no_variables() {
        let vars: Vec<PatternVar> = vec![];
        let conf = PatternInferrer::compute_confidence(&vars, "println!(\"hello\")", "");
        // No variables -> high confidence.
        assert!(conf > 0.9, "expected high confidence, got {conf}");
    }

    #[test]
    fn test_compute_confidence_with_variables() {
        let vars = vec![
            PatternVar {
                name: "$var1".into(),
                node_type: Some("identifier".into()),
            },
            PatternVar {
                name: "$var2".into(),
                node_type: Some("identifier".into()),
            },
        ];
        let conf = PatternInferrer::compute_confidence(&vars, "foo($var1, $var2)", "");
        assert!(
            conf > 0.0 && conf < 1.0,
            "expected moderate confidence, got {conf}"
        );
    }

    #[test]
    fn test_patterns_compatible_same() {
        let a = Pattern::new(
            "foo($var1)".into(),
            "bar($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "stub".into(),
            0.9,
        );
        let b = Pattern::new(
            "foo($var1)".into(),
            "bar($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "stub".into(),
            0.8,
        );
        assert!(PatternInferrer::patterns_compatible(&a, &b));
    }

    #[test]
    fn test_patterns_incompatible_different_var_count() {
        let a = Pattern::new(
            "foo($var1)".into(),
            "bar($var1)".into(),
            vec![PatternVar {
                name: "$var1".into(),
                node_type: None,
            }],
            "stub".into(),
            0.9,
        );
        let b = Pattern::new(
            "foo($var1, $var2)".into(),
            "bar($var1, $var2)".into(),
            vec![
                PatternVar {
                    name: "$var1".into(),
                    node_type: None,
                },
                PatternVar {
                    name: "$var2".into(),
                    node_type: None,
                },
            ],
            "stub".into(),
            0.8,
        );
        assert!(!PatternInferrer::patterns_compatible(&a, &b));
    }
}
