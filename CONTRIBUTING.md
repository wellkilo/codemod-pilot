# Contributing to codemod-pilot

Thank you for your interest in contributing to **codemod-pilot**! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Making Changes](#making-changes)
- [Commit Convention](#commit-convention)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Code Style](#code-style)
- [Adding a New Language](#adding-a-new-language)
- [Submitting a Built-in Rule](#submitting-a-built-in-rule)
- [Getting Help](#getting-help)

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

1. **Fork** the repository on GitHub
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/<your-username>/codemod-pilot.git
   cd codemod-pilot
   ```
3. **Set up** the development environment (see below)
4. **Create** a feature branch:
   ```bash
   git checkout -b feat/my-feature
   ```
5. **Make** your changes and commit them
6. **Push** to your fork and open a Pull Request

## Development Environment

### Prerequisites

- **Rust** 1.75.0 or later (install via [rustup](https://rustup.rs/))
- **Git** 2.x or later
- **C compiler** (for tree-sitter grammar compilation)
  - Linux: `build-essential` or equivalent
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Windows: Visual Studio Build Tools

### Setup

Run the provided setup script to configure your development environment:

```bash
# Clone the repository
git clone https://github.com/codemod-pilot/codemod-pilot.git
cd codemod-pilot

# Run the dev setup script
./scripts/setup-dev.sh

# Or manually:
rustup toolchain install stable
rustup component add rustfmt clippy
cargo build --workspace
cargo test --workspace
```

### Useful Commands

```bash
# Build the entire workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p codemod-core

# Run a specific test
cargo test -p codemod-core -- test_name

# Check formatting
cargo fmt --all -- --check

# Run clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Run the CLI during development
cargo run -p codemod-cli -- learn --before 'foo()' --after 'bar()'

# Update snapshot tests (using insta)
cargo insta test --workspace
cargo insta review
```

## Project Structure

```
codemod-pilot/
├── crates/
│   ├── codemod-core/          # Core engine
│   │   ├── src/
│   │   │   ├── lib.rs         # Public API
│   │   │   ├── pattern/       # Pattern inference from examples
│   │   │   ├── matcher/       # AST pattern matching
│   │   │   ├── transform/     # Code transformation engine
│   │   │   ├── rule/          # Rule parsing and serialization
│   │   │   └── scanner/       # File system scanning
│   │   └── tests/             # Integration tests
│   ├── codemod-cli/           # CLI application
│   │   └── src/
│   │       ├── main.rs        # Entry point
│   │       └── commands/      # CLI subcommands
│   └── codemod-languages/     # Language adapters
│       └── src/
│           ├── lib.rs         # Language registry
│           ├── javascript.rs  # JS/TS adapter
│           └── ...
├── rules/                     # Built-in codemod rules
├── tests/                     # End-to-end integration tests
│   └── fixtures/              # Test fixture files
├── docs/                      # Documentation
└── scripts/                   # Development and CI scripts
```

## Making Changes

### For Bug Fixes

1. Create an issue describing the bug (if one doesn't exist)
2. Write a failing test that reproduces the bug
3. Fix the bug
4. Ensure all tests pass
5. Submit a PR referencing the issue

### For New Features

1. Open a feature request issue to discuss the design
2. Wait for maintainer approval before starting significant work
3. Implement the feature with tests
4. Update documentation as needed
5. Submit a PR referencing the issue

### For Documentation

Documentation improvements are always welcome and don't require an issue. Just submit a PR directly.

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/) for commit messages. This enables automatic changelog generation and semantic versioning.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description |
|:---|:---|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation only changes |
| `style` | Formatting, missing semicolons, etc. (no code change) |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | Performance improvement |
| `test` | Adding or correcting tests |
| `build` | Changes to build system or dependencies |
| `ci` | Changes to CI configuration |
| `chore` | Other changes that don't modify src or test files |

### Scopes

| Scope | Description |
|:---|:---|
| `core` | Changes to `codemod-core` crate |
| `cli` | Changes to `codemod-cli` crate |
| `langs` | Changes to `codemod-languages` crate |
| `docs` | Documentation changes |
| `ci` | CI/CD changes |

### Examples

```
feat(core): add multi-example pattern inference

Supports learning transformation patterns from multiple before/after
example pairs. The engine finds the common structural diff across all
examples and generalizes pattern variables accordingly.

Closes #42
```

```
fix(cli): handle empty scan results gracefully

Previously, scanning a directory with no matching files would panic.
Now it prints a helpful message and exits with code 0.

Fixes #87
```

## Pull Request Process

1. **Ensure** your branch is up to date with `main`:
   ```bash
   git fetch origin
   git rebase origin/main
   ```

2. **Run** the full test suite locally:
   ```bash
   cargo test --workspace
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   ```

3. **Fill out** the PR template completely:
   - Describe what changed and why
   - Link to related issues
   - Note any breaking changes
   - Include screenshots/examples if relevant

4. **Wait** for CI checks to pass

5. **Address** review feedback promptly

6. **Squash** commits if requested by maintainers

### PR Size Guidelines

- **Small PRs** (< 200 lines) are reviewed faster and more thoroughly
- If a change is large, consider splitting it into multiple PRs
- Each PR should be a single, coherent change

## Testing

### Test Categories

- **Unit tests**: Located alongside source code (`#[cfg(test)]` modules)
- **Integration tests**: Located in `crates/*/tests/`
- **Snapshot tests**: Using [insta](https://insta.rs/) for output comparison
- **End-to-end tests**: Located in `tests/` at the workspace root

### Writing Tests

Every new feature or bug fix should include tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matches_simple_rename() {
        let pattern = Pattern::from_example(
            "fetchUser(id)",
            "getUser(id)",
            Language::JavaScript,
        ).unwrap();

        let matches = pattern.find_matches("fetchUser(42)").unwrap();
        assert_eq!(matches.len(), 1);
    }
}
```

### Snapshot Tests

We use [insta](https://insta.rs/) for snapshot testing transformation outputs:

```rust
#[test]
fn test_transform_output() {
    let result = transform(input, rule);
    insta::assert_yaml_snapshot!(result);
}
```

To update snapshots after intentional changes:
```bash
cargo insta test --workspace
cargo insta review
```

### Test Coverage

While we don't enforce a strict coverage target, we aim for:
- All public API functions have at least one test
- All error paths have tests
- Edge cases are covered (empty input, large input, unicode, etc.)

## Code Style

### Rust Style

- Follow the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` with the project's `rustfmt.toml` configuration
- Use `clippy` with no warnings
- Prefer `thiserror` for library errors and `anyhow` for application errors
- Document all public items with doc comments
- Use `log` for logging, not `println!`

### Naming Conventions

- Use descriptive variable names; avoid single-letter names except in iterators
- Module names should be singular (`pattern`, not `patterns`)
- Test function names should describe the scenario: `test_<what>_<condition>_<expected>`

### Error Handling

```rust
// Library code (codemod-core): use thiserror
#[derive(Debug, thiserror::Error)]
pub enum PatternError {
    #[error("failed to parse before example: {0}")]
    ParseBefore(String),
    #[error("no structural diff found between before and after")]
    NoDiff,
}

// Application code (codemod-cli): use anyhow
fn main() -> anyhow::Result<()> {
    let pattern = Pattern::from_example(before, after)?;
    Ok(())
}
```

## Adding a New Language

See [docs/adding-a-language.md](docs/adding-a-language.md) for the full guide.

Quick overview:

1. Add the tree-sitter grammar dependency to `crates/codemod-languages/Cargo.toml`
2. Create a new adapter file (e.g., `src/python.rs`)
3. Implement the `LanguageAdapter` trait
4. Register the language in `src/lib.rs`
5. Add tests with representative code samples
6. Update the supported languages documentation

## Submitting a Built-in Rule

Built-in rules live in the `rules/` directory:

1. Create a `.codemod.yaml` file following the [rule format specification](docs/rule-format.md)
2. Add at least 3 test cases in a `rules/tests/` fixture file
3. Document the rule in the file's `description` field
4. Submit a PR with the `rules` label

## Getting Help

- **Questions**: Open a [Discussion](https://github.com/codemod-pilot/codemod-pilot/discussions) on GitHub
- **Bugs**: File an [Issue](https://github.com/codemod-pilot/codemod-pilot/issues/new?template=bug_report.md) with reproduction steps
- **Feature Ideas**: Open a [Feature Request](https://github.com/codemod-pilot/codemod-pilot/issues/new?template=feature_request.md)

Thank you for contributing to codemod-pilot! Every contribution, no matter how small, makes a difference.
