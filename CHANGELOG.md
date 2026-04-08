# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with Cargo workspace
- `codemod-core` crate: pattern inference engine, AST matching, transformation pipeline
- `codemod-cli` crate: CLI interface with `learn`, `scan`, and `apply` subcommands
- `codemod-languages` crate: language adapter framework with TypeScript/JavaScript support
- Example-based pattern inference from single before/after pair
- Parallel file scanning with `rayon`
- Unified diff preview before applying changes
- Automatic rollback patch generation
- `.codemod.yaml` rule file format (read support)
- Tree-sitter based parsing for JavaScript and TypeScript
- Glob-based file include/exclude filters
- Colored terminal output with progress indicators
- Apache-2.0 license
- CI pipeline with build, test, format, and lint checks
- Project documentation: architecture, rule format, language guide

## [0.1.0] - 2026-04-07

### Added
- First public release
- Core pattern inference engine
- JavaScript and TypeScript language support
- CLI with `learn`, `scan`, and `apply` commands
- Diff preview and safe apply with rollback
- Parallel codebase scanning
- Basic `.codemod.yaml` rule format support

[Unreleased]: https://github.com/codemod-pilot/codemod-pilot/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/codemod-pilot/codemod-pilot/releases/tag/v0.1.0
