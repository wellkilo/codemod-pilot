//! TypeScript language adapter backed by tree-sitter.

use codemod_core::language::LanguageAdapter;
use tree_sitter::Language;

/// TypeScript language adapter.
pub struct TypeScriptAdapter;

impl LanguageAdapter for TypeScriptAdapter {
    fn name(&self) -> &str {
        "typescript"
    }

    fn language(&self) -> Language {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }

    fn file_extensions(&self) -> &[&str] {
        &["ts", "tsx"]
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
            "type_alias_declaration",
            "interface_declaration",
            "enum_declaration",
            "switch_statement",
            "try_statement",
            "throw_statement",
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
            "as_expression",
            "satisfies_expression",
            "non_null_expression",
            "type_assertion",
        ]
    }

    fn identifier_node_types(&self) -> &[&str] {
        &[
            "identifier",
            "property_identifier",
            "type_identifier",
            "shorthand_property_identifier",
            "shorthand_property_identifier_pattern",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_adapter_name() {
        let adapter = TypeScriptAdapter;
        assert_eq!(adapter.name(), "typescript");
    }

    #[test]
    fn test_typescript_adapter_extensions() {
        let adapter = TypeScriptAdapter;
        assert!(adapter.file_extensions().contains(&"ts"));
        assert!(adapter.file_extensions().contains(&"tsx"));
    }

    #[test]
    fn test_typescript_parse() {
        let adapter = TypeScriptAdapter;
        let tree = adapter.parse("const x: number = 42;").unwrap();
        assert!(!tree.root_node().has_error());
    }
}
