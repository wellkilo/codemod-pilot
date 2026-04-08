---
name: Bug Report
about: Report a bug in codemod-pilot
title: "[bug] "
labels: bug
assignees: ""
---

## Describe the Bug

A clear and concise description of what the bug is.

## To Reproduce

Steps to reproduce the behavior:

1. Create a rule or run a command: `codemod-pilot ...`
2. With this input file: (provide minimal code sample)
3. Expected transformation vs actual result

## Expected Behavior

A clear and concise description of what you expected to happen.

## Actual Behavior

What actually happened. Include any error messages or unexpected output.

## Environment

- **OS**: [e.g., macOS 14.3, Ubuntu 22.04, Windows 11]
- **codemod-pilot version**: [e.g., 0.1.0 — run `codemod-pilot --version`]
- **Rust version** (if building from source): [e.g., 1.75.0]
- **Language being transformed**: [e.g., TypeScript, JavaScript]

## Minimal Reproduction

If applicable, provide:

**Before code:**
```
(paste the code you're trying to transform)
```

**Rule / command used:**
```bash
codemod-pilot learn --before '...' --after '...'
```

**Expected after:**
```
(paste what you expected)
```

**Actual after:**
```
(paste what you got)
```

## Additional Context

Add any other context about the problem here (screenshots, log output with `RUST_LOG=debug`, etc.).
