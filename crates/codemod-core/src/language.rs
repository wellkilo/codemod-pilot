//! Language adapter trait for tree-sitter integration.
//!
//! Each programming language supported by codemod-pilot must implement the
//! [`LanguageAdapter`] trait. This provides tree-sitter grammar access and
//! language-specific metadata such as file extensions, statement types, and
//! identifier node types.
//!
//! ## Implementing a new language
//!
//! ```rust,ignore
//! use codemod_core::LanguageAdapter;
//! use tree_sitter::Language;
//!
//! pub struct RustAdapter;
//!
//! impl LanguageAdapter for RustAdapter {
//!     fn name(&self) -> &str { "rust" }
//!     fn language(&self) -> Language { tree_sitter_rust::LANGUAGE.into() }
//!     fn file_extensions(&self) -> &[&str] { &["rs"] }
//!     fn statement_node_types(&self) -> &[&str] { &["let_declaration", "expression_statement"] }
//!     fn expression_node_types(&self) -> &[&str] { &["call_expression", "binary_expression"] }
//!     fn identifier_node_types(&self) -> &[&str] { &["identifier", "type_identifier"] }
//! }
//! ```

use tree_sitter::Language;

/// Trait for language-specific adapters.
///
/// Each supported language implements this trait to provide
/// tree-sitter grammar and language-specific utilities that the
/// pattern engine uses for parsing, matching, and transformation.
pub trait LanguageAdapter: Send + Sync {
    /// Returns the human-readable language name (e.g., `"rust"`, `"javascript"`).
    fn name(&self) -> &str;

    /// Returns the [`tree_sitter::Language`] grammar used for parsing.
    fn language(&self) -> Language;

    /// Returns common file extensions for this language (without the leading dot).
    ///
    /// # Examples
    ///
    /// Rust: `&["rs"]`
    /// JavaScript: `&["js", "jsx", "mjs"]`
    fn file_extensions(&self) -> &[&str];

    /// Returns tree-sitter node types that represent "statements".
    ///
    /// These are used during pattern inference to decide structural
    /// boundaries for extraction.
    fn statement_node_types(&self) -> &[&str];

    /// Returns tree-sitter node types that represent "expressions".
    fn expression_node_types(&self) -> &[&str];

    /// Returns tree-sitter node types that represent identifiers
    /// (variable names, type names, etc.).
    fn identifier_node_types(&self) -> &[&str];

    /// Checks if a file path is supported by this language adapter.
    ///
    /// The default implementation matches the file extension against
    /// [`Self::file_extensions`].
    fn supports_file(&self, path: &std::path::Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.file_extensions().contains(&ext))
            .unwrap_or(false)
    }

    /// Returns true if the given tree-sitter node kind represents a leaf
    /// token (identifier or literal) that could become a pattern variable.
    fn is_leaf_variable_candidate(&self, node_kind: &str) -> bool {
        self.identifier_node_types().contains(&node_kind)
    }

    /// Returns true if the given tree-sitter node kind is a structural
    /// container (statement or expression) that should be compared recursively.
    fn is_structural_node(&self, node_kind: &str) -> bool {
        self.statement_node_types().contains(&node_kind)
            || self.expression_node_types().contains(&node_kind)
    }

    /// Parse source code into a tree-sitter [`Tree`](tree_sitter::Tree).
    ///
    /// This is a convenience method that creates a parser, sets the language,
    /// and parses the given source.
    fn parse(&self, source: &str) -> std::result::Result<tree_sitter::Tree, String> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&self.language())
            .map_err(|e| format!("Failed to set language: {e}"))?;
        parser
            .parse(source, None)
            .ok_or_else(|| "tree-sitter returned no tree".to_string())
    }
}
