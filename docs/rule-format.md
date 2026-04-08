# Rule Format Specification

This document defines the `.codemod.yaml` rule file format used by codemod-pilot.

## Overview

A codemod rule is a YAML file that describes a code transformation. Rules can be:

- **Inferred** by `codemod-pilot learn` from before/after examples
- **Exported** by `codemod-pilot export` after learning
- **Hand-written** for precise control over the transformation

## File Structure

```yaml
# Required fields
name: <string>              # Unique rule identifier (kebab-case)
description: <string>       # Human-readable description
language: <string>          # Target language
version: <string>           # Rule format version

# The transformation pattern
pattern:
  before: <string>          # Code pattern to match (with $variables)
  after: <string>           # Replacement pattern (with $variables)

# Optional fields
include: <list[string]>     # Glob patterns for files to include
exclude: <list[string]>     # Glob patterns for files to exclude
severity: <string>          # info | warning | error (for CI mode)
tags: <list[string]>        # Categorization tags
examples: <list[example]>   # Test cases for validation
meta: <map>                 # Arbitrary metadata
```

## Field Reference

### `name` (required)

A unique identifier for the rule. Must be kebab-case (lowercase letters, numbers, and hyphens).

```yaml
name: replace-fetch-user-info
```

### `description` (required)

A human-readable description of what the rule does and why.

```yaml
description: |
  Migrate from fetchUserInfo to getUserProfile API.
  The new API uses profileId instead of userId.
```

### `language` (required)

The target programming language. Must be one of the supported language identifiers.

| Value | Language |
|:---|:---|
| `javascript` | JavaScript (.js, .jsx) |
| `typescript` | TypeScript (.ts, .tsx) |
| `python` | Python (.py) — coming in v0.2 |
| `go` | Go (.go) — coming in v0.3 |

```yaml
language: typescript
```

### `version` (required)

The rule format version. Currently must be `"1.0"`.

```yaml
version: "1.0"
```

### `pattern` (required)

The transformation pattern, consisting of `before` and `after` templates.

#### Pattern Variables

Variables are prefixed with `$` and match any AST subtree at that position. The same variable name in `before` and `after` refers to the same captured value.

```yaml
pattern:
  before: |
    fetchUserInfo({ userId: $id })
  after: |
    getUserProfile({ profileId: $id })
```

**Variable rules:**

- Variable names must start with `$` followed by an identifier (`$id`, `$expr`, `$name`, etc.)
- Every variable in `after` must appear in `before` (you can't introduce new variables)
- A variable in `before` that doesn't appear in `after` means that captured value is discarded
- The same variable used multiple times in `before` means all occurrences must bind to the same text

#### Multi-line Patterns

Use YAML block scalars for multi-line patterns:

```yaml
pattern:
  before: |
    if ($condition) {
      $body
    }
  after: |
    when ($condition) {
      $body
    }
```

### `include` (optional)

A list of glob patterns specifying which files to consider. If omitted, all files matching the language's default extensions are included.

```yaml
include:
  - "src/**/*.ts"
  - "src/**/*.tsx"
  - "lib/**/*.ts"
```

### `exclude` (optional)

A list of glob patterns specifying which files to skip. Applied after `include`.

```yaml
exclude:
  - "**/*.test.ts"
  - "**/*.spec.ts"
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/__generated__/**"
```

### `severity` (optional)

The severity level for CI mode reporting. Defaults to `"warning"`.

| Value | Behavior in `--ci` mode |
|:---|:---|
| `info` | Report matches but don't fail |
| `warning` | Report matches as warnings |
| `error` | Fail the CI check if any match is found |

```yaml
severity: error
```

### `tags` (optional)

Categorization tags for organizing rules.

```yaml
tags:
  - migration
  - api-change
  - breaking-change
```

### `examples` (optional)

Test cases to validate the rule works correctly. Each example has an `input` and expected `output`.

```yaml
examples:
  - description: "Simple function call"
    input: |
      const user = fetchUserInfo({ userId: currentId });
    output: |
      const user = getUserProfile({ profileId: currentId });

  - description: "Nested in conditional"
    input: |
      if (isLoggedIn) {
        const data = fetchUserInfo({ userId: session.id });
      }
    output: |
      if (isLoggedIn) {
        const data = getUserProfile({ profileId: session.id });
      }
```

Run `codemod-pilot validate --rule <file>` to check all examples pass.

### `meta` (optional)

Arbitrary metadata for tooling integration.

```yaml
meta:
  author: "team-platform"
  jira: "PLAT-1234"
  deprecated-api-version: "2.x"
  migration-guide: "https://docs.example.com/migration"
```

## Complete Example

```yaml
name: moment-to-dayjs-format
description: |
  Migrate moment().format() calls to dayjs().format().
  Part of the Moment.js to Day.js migration.
language: typescript
version: "1.0"

pattern:
  before: |
    moment($date).format($fmt)
  after: |
    dayjs($date).format($fmt)

include:
  - "src/**/*.ts"
  - "src/**/*.tsx"
exclude:
  - "**/*.test.ts"
  - "**/node_modules/**"

severity: warning

tags:
  - migration
  - moment-to-dayjs

examples:
  - description: "Basic format call"
    input: "const d = moment(today).format('YYYY-MM-DD');"
    output: "const d = dayjs(today).format('YYYY-MM-DD');"

  - description: "Format with variable"
    input: "return moment(event.date).format(dateFormat);"
    output: "return dayjs(event.date).format(dateFormat);"

meta:
  author: frontend-platform
  migration-guide: https://wiki.example.com/dayjs-migration
```

## Validation

Use the `validate` command to check a rule file:

```bash
# Validate syntax and examples
codemod-pilot validate --rule ./my-rule.codemod.yaml

# Validate all rules in a directory
codemod-pilot validate --rule-dir ./codemods/
```

Validation checks:
- YAML syntax is valid
- All required fields are present
- Language is supported
- Pattern variables in `after` exist in `before`
- All examples produce expected output
