## Description

<!-- Describe what this PR does and why. Link to any relevant issues. -->

Closes #<!-- issue number -->

## Type of Change

<!-- Check all that apply -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] CI/build change
- [ ] New language support
- [ ] New built-in rule

## Changes Made

<!-- List the key changes in this PR -->

-
-
-

## How Has This Been Tested?

<!-- Describe the tests you ran to verify your changes -->

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Snapshot tests added/updated (run `cargo insta review`)
- [ ] Manual testing performed

**Test commands run:**
```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Screenshots / Output

<!-- If applicable, add screenshots or terminal output showing the change -->

## Checklist

<!-- Check all that apply -->

- [ ] My code follows the project's code style (`cargo fmt` and `cargo clippy` pass)
- [ ] I have added tests that prove my fix is effective or my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] I have updated the documentation accordingly
- [ ] I have added an entry to `CHANGELOG.md` under `[Unreleased]`
- [ ] My commit messages follow the [Conventional Commits](https://www.conventionalcommits.org/) convention

## Additional Notes

<!-- Any additional information for reviewers -->
