# Supported Languages

This document lists all programming languages supported by codemod-pilot, their status, and any language-specific considerations.

## Language Support Matrix

| Language | Status | File Extensions | Grammar Crate | Since |
|:---|:---:|:---|:---|:---:|
| JavaScript | ✅ Stable | `.js`, `.jsx`, `.mjs`, `.cjs` | `tree-sitter-javascript` 0.23 | v0.1.0 |
| TypeScript | ✅ Stable | `.ts`, `.tsx`, `.mts`, `.cts` | `tree-sitter-typescript` 0.23 | v0.1.0 |
| Python | 🚧 In Progress | `.py`, `.pyi` | `tree-sitter-python` 0.23 | v0.2.0 |
| Go | 🚧 Planned | `.go` | `tree-sitter-go` 0.23 | v0.3.0 |
| Rust | 📋 Planned | `.rs` | `tree-sitter-rust` | TBD |
| Java | 📋 Planned | `.java` | `tree-sitter-java` | TBD |
| C | 📋 Planned | `.c`, `.h` | `tree-sitter-c` | TBD |
| C++ | 📋 Planned | `.cpp`, `.hpp`, `.cc` | `tree-sitter-cpp` | TBD |

### Status Legend

- ✅ **Stable** — Fully supported, tested in production-grade scenarios
- 🚧 **In Progress** — Implementation started; available for testing but may have limitations
- 📋 **Planned** — On the roadmap; contributions welcome

## Language-Specific Notes

### JavaScript

- Supports ES2024 syntax including optional chaining, nullish coalescing, and decorators
- JSX is fully supported
- CommonJS (`require()`) and ESM (`import`) patterns both work
- Dynamic imports (`import()`) are treated as call expressions

### TypeScript

- Full TypeScript syntax support including generics, type annotations, and enums
- TSX (TypeScript + JSX) is supported
- Type-only constructs (interfaces, type aliases) can be matched and transformed
- Decorators (both legacy and Stage 3) are supported
- Note: Type checking is **not** performed — codemod-pilot operates on syntax only

### Python (Coming in v0.2)

- Python 3.x syntax
- f-strings are supported
- Decorators and comprehensions are fully parsed
- Indentation-sensitive patterns are handled by normalizing whitespace in the pattern

### Go (Coming in v0.3)

- Full Go syntax support
- Goroutines and channels can be matched
- Package import patterns are supported

## Requesting a New Language

If your language isn't listed above, you can:

1. Open a [New Language issue](https://github.com/codemod-pilot/codemod-pilot/issues/new?template=new_language.md)
2. Implement it yourself — see [Adding a Language](adding-a-language.md)

Requirements for a new language:
- A maintained tree-sitter grammar must exist
- The grammar crate must be published on crates.io
- At least 10 representative test cases must be provided
