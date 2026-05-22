# Releasing lazygitlab

To ensure consistency between the code, `Cargo.toml`, and Git tags, always use the automated bump script. This document guides developers and AI assistants through the release process.

## Pre-Release Checklist

Before performing a release, ensure:
1. **Working Tree is Clean**: Run `git status` to ensure all feature/fix commits are committed and there are no uncommitted changes.
2. **Tests Pass**: Run `cargo test` to verify that all unit and integration tests are passing.
3. **Linting Check**: Run `cargo clippy --all-targets -- -D warnings` to verify clean compilation.

---

## Automated Release Procedure

Run the version bumping script from the project root using Python 3:

```bash
# Option A: Bump patch version (e.g., 0.13.0 -> 0.13.1)
# Use this for backwards-compatible bug fixes and performance optimizations.
python scripts/bump_version.py patch

# Option B: Bump minor version (e.g., 0.13.0 -> 0.14.0)
# Use this for new backwards-compatible features.
python scripts/bump_version.py minor

# Option C: Bump major version (e.g., 0.13.0 -> 1.0.0)
# Use this for breaking changes or major milestones.
python scripts/bump_version.py major
```

### What the Script Does:
1. Validates and parses the current version from `Cargo.toml`.
2. Bumps the version number and updates `Cargo.toml`.
3. Runs `cargo check` to automatically synchronize `Cargo.lock`.
4. Stages `Cargo.toml` and `Cargo.lock` and commits them under `"chore: bump version to X.Y.Z"`.
5. Creates an annotated git tag: `vX.Y.Z` with the message `"chore: bump version to X.Y.Z"`.

---

## Publishing

After running the script successfully, push the commit and tags to the remote repository:

```bash
git push origin master --tags
```

---

## Manual Procedure (Backup/Fallback)

If you cannot run the script, perform the following steps manually in order:

1. Update the `version` field in `Cargo.toml`.
2. Run `cargo check` to generate/update `Cargo.lock`.
3. Stage and commit both files:
   ```bash
   git add Cargo.toml Cargo.lock
   git commit -m "chore: bump version to X.Y.Z"
   ```
4. Create an **annotated** git tag (essential for repository conventions):
   ```bash
   git tag -a vX.Y.Z -m "chore: bump version to X.Y.Z"
   ```
5. Push master and the tag to the remote:
   ```bash
   git push origin master --tags
   ```

---

## Guidelines for AI Assistants
When asked to bump the version:
1. Check the existing version in `Cargo.toml`.
2. Propose or run `python scripts/bump_version.py [major|minor|patch]` based on the scale of changes.
3. If permissions are needed, ask or request command execution approval.
4. Execute `git push origin master --tags` to publish the changes and tags.
