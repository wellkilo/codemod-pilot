# Adding a New Language

This guide walks you through adding support for a new programming language to codemod-pilot.

## Prerequisites

- A [tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar for the language
- The grammar crate published on [crates.io](https://crates.io/)
- Familiarity with the language's syntax and common refactoring patterns

## Step-by-Step Guide

### 1. Add the Grammar Dependency

Edit `codemod-pilot/Cargo.toml` to add the grammar crate to workspace dependencies:

```toml
[workspace.dependencies]
# ... existing dependencies ...
tree-sitter-python = "0.23"   # Add your language here
```

Then add it to `crates/codemod-languages/Cargo.toml`:

```toml
[dependencies]
tree-sitter-python.workspace = true
```

### 2. Create the Language Adapter

Create a new file at `crates/codemod-languages/src/<language>.rs`:

```rust
//! Language adapter for <Language>.

use anyhow::Result;
use codemod_core::language::{LanguageAdapter, LanguageId};
use tree_sitter::Language;

/// Adapter for the <Language> programming language.
pub struct PythonAdapter;

impl LanguageAdapter for PythonAdapter {
    fn id(&self) -> LanguageId {
        LanguageId::new("python")
    }

    fn display_name(&self) -> &str {
        "Python"
    }

    fn file_extensions(&self) -> &[&str] {
        &["py", "pyi"]
    }

    fn tree_sitter_language(&self) -> Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn comment_prefixes(&self) -> &[&str] {
        &["#"]
    }

    fn string_delimiters(&self) -> &[(&str, &str)] {
        &[
            ("\"", "\""),
            ("'", "'"),
            ("\"\"\"", "\"\"\""),
            ("'''", "'''"),
        ]
    }

    /// Returns node kinds that represent statement boundaries.
    ///
    /// This helps the pattern matcher understand where one logical
    /// unit of code ends and another begins.
    fn statement_node_kinds(&self) -> &[&str] {
        &[
            "expression_statement",
            "return_statement",
            "if_statement",
            "for_statement",
            "while_statement",
            "function_definition",
            "class_definition",
            "import_statement",
            "import_from_statement",
            "assignment",
        ]
    }

    /// Returns node kinds that represent expression boundaries.
    fn expression_node_kinds(&self) -> &[&str] {
        &[
            "call",
            "attribute",
            "subscript",
            "binary_operator",
            "unary_operator",
            "comparison_operator",
            "boolean_operator",
            "conditional_expression",
            "lambda",
            "list_comprehension",
            "dictionary_comprehension",
            "set_comprehension",
            "generator_expression",
        ]
    }
}
```

### 3. Implement the `LanguageAdapter` Trait

The `LanguageAdapter` trait is defined in `codemod-core`. Here's the full trait:

```rust
pub trait LanguageAdapter: Send + Sync {
    /// Unique identifier for this language (e.g., "python", "go").
    fn id(&self) -> LanguageId;

    /// Human-readable display name (e.g., "Python", "Go").
    fn display_name(&self) -> &str;

    /// File extensions this language handles (without dots).
    fn file_extensions(&self) -> &[&str];

    /// The tree-sitter Language object for parsing.
    fn tree_sitter_language(&self) -> Language;

    /// Comment prefix strings (e.g., ["//", "/*"] for C-style languages).
    fn comment_prefixes(&self) -> &[&str];

    /// String delimiter pairs (open, close).
    fn string_delimiters(&self) -> &[(&str, &str)];

    /// Node kinds that represent statement-level constructs.
    fn statement_node_kinds(&self) -> &[&str];

    /// Node kinds that represent expression-level constructs.
    fn expression_node_kinds(&self) -> &[&str];

    /// Optional: customize how indentation is detected for this language.
    /// Default implementation handles common cases.
    fn detect_indentation(&self, source: &str) -> IndentStyle {
        IndentStyle::detect(source)
    }

    /// Optional: customize how whitespace normalization works for patterns.
    /// Override for whitespace-sensitive languages like Python.
    fn normalize_pattern_whitespace(&self, pattern: &str) -> String {
        pattern.to_string()
    }
}
```

### 4. Register the Language

Edit `crates/codemod-languages/src/lib.rs` to register your adapter:

```rust
mod python;

use python::PythonAdapter;

pub fn register_all(registry: &mut LanguageRegistry) {
    registry.register(Box::new(JavaScriptAdapter));
    registry.register(Box::new(TypeScriptAdapter));
    registry.register(Box::new(PythonAdapter));  // Add this line
}
```

### 5. Add Tests

Create test files in `crates/codemod-languages/tests/`:

```rust
// tests/python_test.rs

use codemod_core::pattern::Pattern;
use codemod_languages::python::PythonAdapter;

#[test]
fn test_python_simple_rename() {
    let adapter = PythonAdapter;
    let pattern = Pattern::from_example(
        &adapter,
        "old_function(x, y)",
        "new_function(x, y)",
    ).unwrap();

    let input = "result = old_function(a, b)";
    let matches = pattern.find_in_source(&adapter, input).unwrap();
    assert_eq!(matches.len(), 1);

    let output = matches[0].apply_transform().unwrap();
    assert!(output.contains("new_function(a, b)"));
}

#[test]
fn test_python_decorator_transform() {
    let adapter = PythonAdapter;
    let pattern = Pattern::from_example(
        &adapter,
        "@app.route($path)\ndef $func($args):",
        "@app.get($path)\ndef $func($args):",
    ).unwrap();

    let input = r#"
@app.route("/users")
def list_users(request):
    return get_all_users()
"#;

    let matches = pattern.find_in_source(&adapter, input).unwrap();
    assert_eq!(matches.len(), 1);
}

// Add at least 10 representative tests covering:
// - Simple function rename
// - Method call transformation
// - Import statement changes
// - Decorator changes
// - Class method patterns
// - Multi-line patterns
// - Nested expressions
// - String literal patterns
// - Comprehension patterns
// - Error cases (no match, ambiguous match)
```

### 6. Test with Real-World Code

Create fixture files in `tests/fixtures/<language>/` with realistic code samples:

```
tests/fixtures/python/
├── simple_rename/
│   ├── input.py
│   ├── expected.py
│   └── rule.codemod.yaml
├── import_migration/
│   ├── input.py
│   ├── expected.py
│   └── rule.codemod.yaml
└── decorator_change/
    ├── input.py
    ├── expected.py
    └── rule.codemod.yaml
```

### 7. Update Documentation

1. Update the language table in `docs/supported-languages.md`
2. Update the README.md language table
3. Add any language-specific notes

### 8. Submit Your PR

Follow the [Contributing Guide](../CONTRIBUTING.md) to submit your pull request:

1. Use the `[language]` issue template to track the work
2. Title your PR: `feat(langs): add <Language> support`
3. Include your test results and any known limitations

## Tips for Good Language Adapters

- **Study the grammar**: Run `tree-sitter parse` on sample code to understand the AST structure
- **Cover edge cases**: Every language has quirks — heredocs, string interpolation, macros, etc.
- **Test whitespace**: Ensure patterns work regardless of indentation style
- **Test unicode**: Identifiers with non-ASCII characters should work
- **Document limitations**: If certain language features aren't supported, document them clearly

## Troubleshooting

### Grammar compilation errors

Ensure you have a C compiler installed. Tree-sitter grammars are compiled from C source:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install
```

### Parser returns unexpected AST

Use `tree-sitter parse` to inspect the AST:

```bash
echo 'your_code_here' | tree-sitter parse --language python
```

Or use the [tree-sitter playground](https://tree-sitter.github.io/tree-sitter/playground) in your browser.

### Pattern matching doesn't work

Common issues:
- The node kinds in `statement_node_kinds()` or `expression_node_kinds()` don't match what tree-sitter produces
- Whitespace normalization is needed for indentation-sensitive languages
- The grammar uses different node names than expected (check with `tree-sitter parse`)
