# API Rename — End-to-End Example

> Renaming a function across your entire codebase with codemod-pilot.

## Scenario

Your team has decided to rename `fetchUserInfo` to `getUserProfile` across
the entire TypeScript codebase. The function also changes its parameter from
`{ userId }` to `{ profileId }`. Doing this manually with find-and-replace
would miss structural nuances (aliased imports, destructured calls, etc.).

## Step 1 — Create before/after examples

Pick a representative call site and write the before/after pair:

**before.ts**
```typescript
import { fetchUserInfo } from './api';

async function loadProfile(id: string) {
  const user = await fetchUserInfo({ userId: id });
  console.log(user.name);
  return user;
}

export function getUser(userId: string) {
  return fetchUserInfo({ userId });
}
```

**after.ts**
```typescript
import { getUserProfile } from './api';

async function loadProfile(id: string) {
  const user = await getUserProfile({ profileId: id });
  console.log(user.name);
  return user;
}

export function getUser(userId: string) {
  return getUserProfile({ profileId: userId });
}
```

> These fixtures are available in `tests/fixtures/rename-function/`.

## Step 2 — Learn the pattern

```bash
codemod-pilot learn \
  --before tests/fixtures/rename-function/before.ts \
  --after  tests/fixtures/rename-function/after.ts \
  --name   rename-fetch-to-getprofile
```

The inferred pattern will contain multiple variables:

```yaml
name: rename-fetch-to-getprofile
language: typescript
pattern:
  before: "fetchUserInfo({ userId: $id })"
  after:  "getUserProfile({ profileId: $id })"
variables:
  - name: "$id"
    node_type: identifier
confidence: 0.95
```

## Step 3 — Scan for matches

```bash
codemod-pilot scan \
  --rule rename-fetch-to-getprofile \
  --target src/ \
  --include "**/*.ts" "**/*.tsx" \
  --exclude "**/*.test.ts"
```

Sample output:

```
Scanning src/ ...

Found 34 matches in 18 files (scanned 142 files in 0.3s)

  src/api/client.ts:45       fetchUserInfo({ userId: currentId })
  src/hooks/useUser.ts:12    fetchUserInfo({ userId: props.id })
  src/pages/Profile.tsx:28   fetchUserInfo({ userId: routeParams.id })
  ...
```

## Step 4 — Preview and apply

Preview the diff without modifying files:

```bash
codemod-pilot apply \
  --rule rename-fetch-to-getprofile \
  --target src/ \
  --dry-run
```

Once satisfied, apply the changes:

```bash
codemod-pilot apply \
  --rule rename-fetch-to-getprofile \
  --target src/
```

Each file will show a unified diff and ask for confirmation:

```diff
--- a/src/hooks/useUser.ts
+++ b/src/hooks/useUser.ts
@@ -10,7 +10,7 @@
 export function useUser(id: string) {
   const [user, setUser] = useState(null);
   useEffect(() => {
-    fetchUserInfo({ userId: id }).then(setUser);
+    getUserProfile({ profileId: id }).then(setUser);
   }, [id]);
   return user;
 }
```

## Step 5 — Handle imports

The import statement is also updated automatically because the pattern
captures the structural change:

```diff
-import { fetchUserInfo } from './api';
+import { getUserProfile } from './api';
```

## Step 6 — Verify and commit

```bash
# Run your test suite
npm test

# Type-check
npx tsc --noEmit

# If everything passes, commit
git add -A
git commit -m "refactor: rename fetchUserInfo to getUserProfile"
```

## Rollback

If you discover issues after applying:

```bash
codemod-pilot apply --rollback
```

This restores every modified file to its original content using the
automatically saved rollback data in `.codemod-pilot/rollback/`.

## CI Integration

Add a check to your CI pipeline to prevent new occurrences of the old name:

```bash
codemod-pilot check \
  --rule rename-fetch-to-getprofile \
  --target src/ \
  --ci
```

This exits with a non-zero status if any matches are found, failing the build.

## Summary

| Step | Command |
|------|---------|
| Learn | `codemod-pilot learn --before ... --after ... --name ...` |
| Scan  | `codemod-pilot scan --rule ... --target src/` |
| Apply | `codemod-pilot apply --rule ... --target src/` |
| Check | `codemod-pilot check --rule ... --target src/ --ci` |
| Rollback | `codemod-pilot apply --rollback` |
