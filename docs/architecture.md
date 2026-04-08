# Architecture

This document describes the high-level architecture of codemod-pilot. It is intended for contributors who want to understand how the system works internally.

## Overview

codemod-pilot is structured as a Cargo workspace with three crates:

```
codemod-pilot/
├── crates/
│   ├── codemod-core/         # Core engine (library)
│   ├── codemod-cli/          # CLI frontend (binary)
│   └── codemod-languages/    # Language adapters (library)
```

The data flows through the system in a pipeline:

```
User Input ──▶ Pattern Inference ──▶ Codebase Scanning ──▶ Transformation ──▶ Output
(examples)     (codemod-core)       (codemod-core)        (codemod-core)     (codemod-cli)
```

## Crate Responsibilities

### `codemod-core`

The core library contains the fundamental algorithms and data structures. It has **no** CLI or language-specific dependencies.

**Key modules:**

| Module | Responsibility |
|:---|:---|
| `pattern` | Infers a structural transformation pattern from before/after AST pairs |
| `matcher` | Finds all occurrences of a pattern in a given AST |
| `transform` | Applies the inferred transformation to matched code |
| `rule` | Parses and serializes `.codemod.yaml` rule files |
| `scanner` | Walks the file system, filters by glob patterns, reads files in parallel |
| `diff` | Generates unified diffs for preview output |

**Design principles:**

- All AST operations go through a `LanguageAdapter` trait — the core never depends on a specific tree-sitter grammar
- The core is fully synchronous; parallelism is achieved via `rayon` in the scanner
- All public functions return `Result<T, CoreError>` using `thiserror`

### `codemod-cli`

The CLI binary provides the user-facing interface. It depends on `codemod-core` and `codemod-languages`.

**Subcommands:**

| Command | Description |
|:---|:---|
| `learn` | Accept before/after examples and infer a pattern |
| `scan` | Scan a directory and report all matches |
| `apply` | Apply transformations (with `--preview`, `--execute`, `--rollback`) |
| `export` | Export the current pattern as a `.codemod.yaml` file |
| `validate` | Validate a `.codemod.yaml` rule file |

**Design principles:**

- Uses `clap` derive API for argument parsing
- All user-facing output goes through a `Printer` abstraction for testability
- Supports `--ci` mode (JSON output, no interactive prompts)

### `codemod-languages`

Provides concrete `LanguageAdapter` implementations backed by tree-sitter grammars.

**Design principles:**

- Each language is a separate module implementing `LanguageAdapter`
- Languages are registered in a `LanguageRegistry` that maps file extensions to adapters
- Adding a new language requires only implementing the trait and registering it

## Key Data Structures

### `Pattern`

Represents an inferred transformation pattern.

```
Pattern {
    before_template: AstTemplate,   // Generalized AST with placeholders
    after_template: AstTemplate,    // Target AST with same placeholders
    variables: Vec<PatternVar>,     // Named placeholders ($id, $expr, etc.)
    language: LanguageId,           // Which language this pattern targets
}
```

### `AstTemplate`

A tree structure that mirrors tree-sitter's concrete syntax tree (CST) but with **placeholder nodes** where pattern variables appear.

```
AstTemplate {
    kind: NodeKind,               // Either Concrete("identifier") or Variable("$name")
    children: Vec<AstTemplate>,
    text: Option<String>,         // Leaf node text (None for inner nodes)
}
```

### `Match`

A found occurrence of a pattern in a source file.

```
Match {
    file_path: PathBuf,
    byte_range: Range<usize>,
    line_range: Range<usize>,
    bindings: HashMap<String, String>,  // $variable -> captured text
    original_text: String,
    transformed_text: String,
}
```

### `Rule`

A serializable codemod rule (stored as `.codemod.yaml`).

```
Rule {
    name: String,
    description: String,
    language: LanguageId,
    version: String,
    pattern: PatternDef,          // before/after strings
    include: Vec<GlobPattern>,
    exclude: Vec<GlobPattern>,
    examples: Vec<Example>,       // For validation
}
```

## Pipeline Deep Dive

### 1. Pattern Inference

```
before_code ──▶ parse(AST₁) ──┐
                               ├──▶ structural_diff(AST₁, AST₂) ──▶ generalize() ──▶ Pattern
after_code  ──▶ parse(AST₂) ──┘
```

The inference algorithm:

1. Parse both snippets into tree-sitter CSTs
2. Walk both trees in parallel, comparing node types and text
3. Where nodes differ, create a pattern variable
4. Where nodes are identical, keep them as concrete template nodes
5. Validate that all variables in `after` also appear in `before` (no invented variables)

### 2. Codebase Scanning

```
target_dir ──▶ walk_files() ──▶ filter(globs) ──▶ par_iter() ──▶ parse + match ──▶ Vec<Match>
```

Scanning uses `walkdir` for traversal, `globset` for filtering, and `rayon` for parallel processing. Each file is independently parsed and matched.

### 3. Transformation

```
Match ──▶ substitute(after_template, bindings) ──▶ transformed_text
```

For each match, the `after_template` is instantiated by replacing pattern variables with their captured bindings from the match.

### 4. Apply

```
Vec<Match> ──▶ sort_by_file_and_offset() ──▶ apply_in_reverse_order() ──▶ write_files()
                                                                          ──▶ generate_rollback_patch()
```

Matches within the same file are applied in reverse byte-offset order to avoid invalidating earlier offsets. A rollback patch (unified diff) is always generated before writing.

## Error Handling Strategy

- **`codemod-core`**: Uses `thiserror` with a `CoreError` enum. All functions return `Result<T, CoreError>`.
- **`codemod-cli`**: Uses `anyhow` for ergonomic error propagation. Errors are formatted for human-readable output.
- **Panics**: The codebase should never panic in release mode. All potential panics are converted to `Result` errors.

## Concurrency Model

- **File scanning**: `rayon::par_iter()` over files — each file is processed independently
- **Pattern matching**: Single-threaded within a file (tree-sitter is not thread-safe per parser instance)
- **File writing**: Sequential to avoid data races on the file system

## Testing Strategy

| Level | Location | Framework |
|:---|:---|:---|
| Unit tests | `src/*.rs` (`#[cfg(test)]`) | Built-in |
| Integration tests | `crates/*/tests/` | Built-in + `insta` |
| Snapshot tests | Transformation output | `insta` (YAML snapshots) |
| End-to-end tests | `tests/` workspace root | `assert_cmd` + `tempfile` |

## Future Architecture Considerations

- **Plugin system**: Language adapters may move to dynamic loading (`.so`/`.dylib`) for v1.0
- **LSP server**: A `codemod-lsp` crate may be added for VS Code extension support
- **WASM target**: Core may compile to WASM for the web playground
