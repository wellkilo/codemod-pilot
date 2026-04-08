---
name: New Language Support
about: Request or propose support for a new programming language
title: "[language] Add support for "
labels: language-support
assignees: ""
---

## Language

**Language name**: (e.g., Python, Go, Rust, Java)

## Tree-sitter Grammar

**Grammar crate**: (e.g., `tree-sitter-python`)
**Grammar repository**: (link to the tree-sitter grammar repo)
**Crate version**: (latest stable version on crates.io)

## Motivation

Why should codemod-pilot support this language? Include information about:

- Estimated user demand
- Common refactoring scenarios in this language
- Any unique AST challenges for this language

## Example Transformations

Provide 2-3 example before/after transformations that are common in this language:

**Example 1:**
```
# Before
(code before transformation)

# After
(code after transformation)
```

**Example 2:**
```
# Before
(code before transformation)

# After
(code after transformation)
```

## Implementation Notes

If you plan to implement this yourself, describe your approach:

- [ ] Added tree-sitter grammar dependency
- [ ] Created language adapter (`crates/codemod-languages/src/<lang>.rs`)
- [ ] Implemented `LanguageAdapter` trait
- [ ] Registered language in the adapter registry
- [ ] Added unit tests
- [ ] Added integration tests with representative code samples
- [ ] Updated documentation (supported languages table)

## Volunteer

- [ ] I'm willing to implement this language adapter
- [ ] I can help with testing but not implementation
- [ ] I'm requesting this for tracking purposes only
