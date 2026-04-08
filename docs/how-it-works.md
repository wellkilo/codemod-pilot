# How It Works

This document explains the pattern inference and transformation algorithms that power codemod-pilot.

## Overview

codemod-pilot works in four phases:

1. **Parse** — Convert before/after code snippets into Abstract Syntax Trees (ASTs)
2. **Infer** — Compute a structural diff between the two ASTs to create a transformation pattern
3. **Match** — Scan target files for code that matches the "before" pattern
4. **Transform** — Rewrite matched code using the "after" pattern

## Phase 1: Parsing

Both the "before" and "after" code snippets are parsed using [tree-sitter](https://tree-sitter.github.io/tree-sitter/), a fast, incremental parsing framework.

Tree-sitter produces a **concrete syntax tree** (CST) that preserves all syntactic details, including whitespace and comments. This is important because codemod-pilot aims to produce minimal, clean diffs.

```
Input: fetchUserInfo({ userId: id })

CST:
  call_expression
    identifier: "fetchUserInfo"
    arguments
      object
        pair
          property_identifier: "userId"
          identifier: "id"
```

## Phase 2: Pattern Inference

### Single-Example Inference

Given a before/after pair, the engine performs a parallel tree walk:

```
Before AST          After AST
─────────           ─────────
call_expression     call_expression
  ├─ "fetchUserInfo"  ├─ "getUserProfile"     ← CHANGED
  └─ arguments        └─ arguments
       └─ object           └─ object
            └─ pair              └─ pair
                 ├─ "userId"          ├─ "profileId"  ← CHANGED
                 └─ "id"              └─ "id"          ← SAME
```

**Algorithm:**

1. Walk both trees simultaneously, comparing nodes at the same structural position
2. **Same node type + same text** → Keep as a concrete pattern node
3. **Same node type + different text** → Create a pattern variable (e.g., `$1`)
4. **Different node type** → Mark as a structural change
5. **Identical subtrees** → Collapse into a single concrete node (optimization)

The result is a **pattern pair** (before-template, after-template) with shared variables:

```
Before Template:
  call_expression
    identifier: "fetchUserInfo"
    arguments
      object
        pair
          property_identifier: "userId"
          identifier: $1

After Template:
  call_expression
    identifier: "getUserProfile"
    arguments
      object
        pair
          property_identifier: "profileId"
          identifier: $1
```

### Multi-Example Inference (v0.2)

When multiple examples are provided, the engine:

1. Infers a pattern from each example independently
2. Finds the **intersection** of all patterns — parts that are consistent across all examples become concrete; parts that vary become variables
3. Validates that the combined pattern correctly transforms all provided examples

This allows learning more general patterns from specific examples.

## Phase 3: Pattern Matching

The before-template is used to search target files. For each file:

1. Parse the file into a CST
2. Walk the CST, attempting to match the before-template at each node
3. A match succeeds when the structural shape and concrete text match, with variables binding to any subtree

**Matching algorithm (recursive):**

```
match(pattern_node, cst_node) -> Option<Bindings>:
  if pattern_node is Variable($name):
    return Some({ $name: cst_node.text() })

  if pattern_node.kind != cst_node.kind:
    return None

  if pattern_node is leaf:
    if pattern_node.text == cst_node.text:
      return Some({})
    else:
      return None

  // Inner node: match all children
  bindings = {}
  for (p_child, c_child) in zip(pattern_node.children, cst_node.children):
    child_bindings = match(p_child, c_child)?
    bindings.merge(child_bindings)?  // Fail if same var binds to different text
  return Some(bindings)
```

### Handling Structural Flexibility

Some patterns need to match regardless of surrounding context (e.g., a function call inside an `if` statement vs. at the top level). The matcher handles this by:

- Attempting to match at every node in the CST (not just the root)
- Supporting "contextual anchoring" — matching within a specific parent node type if specified in the rule

## Phase 4: Transformation

For each match, the transformation substitutes captured bindings into the after-template:

```
substitute(after_template, bindings) -> String:
  if after_template is Variable($name):
    return bindings[$name]

  if after_template is leaf:
    return after_template.text

  // Reconstruct from children
  return join(children.map(|c| substitute(c, bindings)))
```

### Whitespace Preservation

codemod-pilot aims to preserve the original code's formatting:

- **Indentation**: The transformation inherits the indentation level of the matched code
- **Trailing whitespace**: Preserved from the original
- **Newlines**: Line endings (LF vs. CRLF) are preserved per file

### Conflict Detection

Sometimes a transformation is ambiguous:

- A pattern variable could bind to multiple valid subtrees
- Overlapping matches in the same file
- A transformation would produce syntactically invalid code

In these cases, the match is flagged as a **conflict** and presented to the user in interactive mode for manual resolution.

## Performance

### Parallel Scanning

File scanning uses `rayon` for parallel processing:

```
files.par_iter()
    .filter(|f| glob_matches(f))
    .map(|f| {
        let content = fs::read_to_string(f)?;
        let tree = parser.parse(&content)?;
        find_matches(&tree, &pattern)
    })
    .collect()
```

### Early Termination

Before parsing a file with tree-sitter, a quick text-based pre-filter checks if the file contains any of the concrete text fragments from the pattern. Files that definitely don't match are skipped without parsing.

### Benchmarks

On a MacBook Pro M3 (16GB RAM):

| Codebase Size | Files | Scan Time |
|:---|:---:|:---:|
| Small (1k files) | 1,000 | ~0.2s |
| Medium (10k files) | 10,000 | ~1.5s |
| Large (100k files) | 100,000 | ~12s |

*Note: Benchmarks depend on file sizes, pattern complexity, and language grammar.*

## Limitations

- **Cross-file patterns**: codemod-pilot currently works on a per-file basis. Patterns that span multiple files (e.g., renaming an export and all its imports) require multiple rules.
- **Semantic analysis**: The engine operates on syntax only. It cannot understand types, scoping, or control flow. A variable named `x` in the pattern will match any identifier, regardless of its semantic meaning.
- **Macro expansion**: In languages with macros (Rust, C), the engine operates on the pre-expansion source code.
- **Comments**: Comments within a matched region are preserved but not pattern-matched. A pattern won't match or skip code based on comment content.
