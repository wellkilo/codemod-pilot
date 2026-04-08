# React Migration Example

> Migrating deprecated React lifecycle methods to modern alternatives with codemod-pilot.

## Scenario

Your project contains hundreds of class components that still use deprecated
lifecycle methods (`componentWillMount`, `componentWillReceiveProps`, etc.).
Manually updating each occurrence is tedious and error-prone. With
codemod-pilot you can teach the tool the transformation by providing a single
before/after example and then apply it across the entire codebase.

## Step 1 — Prepare examples

Create a minimal before/after pair that captures the change you want:

**before.tsx**
```tsx
class UserCard extends React.Component {
  componentWillMount() {
    this.loadUser();
  }

  render() {
    return <div>{this.state.user.name}</div>;
  }
}
```

**after.tsx**
```tsx
class UserCard extends React.Component {
  componentDidMount() {
    this.loadUser();
  }

  render() {
    return <div>{this.state.user.name}</div>;
  }
}
```

## Step 2 — Learn the pattern

```bash
codemod-pilot learn \
  --before examples/before.tsx \
  --after  examples/after.tsx \
  --name   replace-will-mount
```

codemod-pilot will infer the pattern:

```yaml
name: replace-will-mount
pattern:
  before: "componentWillMount()"
  after:  "componentDidMount()"
```

## Step 3 — Scan the codebase

Preview which files will be affected:

```bash
codemod-pilot scan --rule replace-will-mount --target src/
```

Expected output:

```
Found 47 matches in 23 files

  src/components/UserCard.tsx:12      componentWillMount() {
  src/components/Dashboard.tsx:8      componentWillMount() {
  src/pages/Settings.tsx:25           componentWillMount() {
  ...
```

## Step 4 — Apply the transformation

```bash
codemod-pilot apply --rule replace-will-mount --target src/
```

The tool generates a unified diff for each file and prompts for confirmation
in interactive mode, or applies all changes in CI mode (`--ci`).

## Step 5 — Verify

```bash
npm test          # Run your test suite
npm run build     # Ensure the build still passes
```

If anything looks wrong, roll back:

```bash
codemod-pilot apply --rollback
```

## Tips

- **Multiple lifecycles**: Create separate rules for `componentWillReceiveProps`
  → `static getDerivedStateFromProps` and `componentWillUpdate` →
  `getSnapshotBeforeUpdate`.
- **Confidence threshold**: Use `--min-confidence 0.8` to skip low-confidence
  matches.
- **Dry run**: Add `--dry-run` to preview diffs without writing files.
- **CI integration**: Use `codemod-pilot check --rule replace-will-mount` in
  your CI pipeline to prevent regressions.
