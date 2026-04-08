//! JavaScript language adapter backed by tree-sitter.

use codemod_core::language::LanguageAdapter;
use tree_sitter::Language;

/// JavaScript language adapter.
pub struct JavaScriptAdapter;

impl LanguageAdapter for JavaScriptAdapter {
    fn name(&self) -> &str {
        "javascript"
    }

    fn language(&self) -> Language {
        tree_sitter_javascript::LANGUAGE.into()
    }

    fn file_extensions(&self) -> &[&str] {
        &["js", "jsx", "mjs", "cjs"]
    }

    fn statement_node_types(&self) -> &[&str] {
        &[
            "expression_statement",
            "variable_declaration",
            "lexical_declaration",
            "function_declaration",
            "class_declaration",
            "if_statement",
            "for_statement",
            "for_in_statement",
            "while_statement",
            "do_statement",
            "return_statement",
            "import_statement",
            "export_statement",
            "switch_statement",
            "try_statement",
            "throw_statement",
            "with_statement",
            "labeled_statement",
        ]
    }

    fn expression_node_types(&self) -> &[&str] {
        &[
            "call_expression",
            "member_expression",
            "assignment_expression",
            "binary_expression",
            "unary_expression",
            "ternary_expression",
            "arrow_function",
            "template_string",
            "object",
            "array",
            "new_expression",
            "await_expression",
            "parenthesized_expression",
            "sequence_expression",
            "yield_expression",
        ]
    }

    fn identifier_node_types(&self) -> &[&str] {
        &[
            "identifier",
            "property_identifier",
            "shorthand_property_identifier",
            "shorthand_property_identifier_pattern",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_adapter_name() {
        let adapter = JavaScriptAdapter;
        assert_eq!(adapter.name(), "javascript");
    }

    #[test]
    fn test_javascript_adapter_extensions() {
        let adapter = JavaScriptAdapter;
        assert!(adapter.file_extensions().contains(&"js"));
        assert!(adapter.file_extensions().contains(&"jsx"));
        assert!(adapter.file_extensions().contains(&"mjs"));
        assert!(adapter.file_extensions().contains(&"cjs"));
    }

    #[test]
    fn test_javascript_parse() {
        let adapter = JavaScriptAdapter;
        let tree = adapter.parse("const x = 42;").unwrap();
        assert!(!tree.root_node().has_error());
    }
}
