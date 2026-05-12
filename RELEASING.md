# Releasing lazygitlab

To ensure consistency between the code, `Cargo.toml`, and Git tags, always use the automated bump script.

## Automated Release Procedure

Run the following command from the project root:

```bash
# Bump the patch version (0.7.3 -> 0.7.4)
python scripts/bump_version.py patch

# Bump the minor version (0.7.3 -> 0.8.0)
python scripts/bump_version.py minor

# Bump the major version (0.7.3 -> 1.0.0)
python scripts/bump_version.py major
```

The script will:
1. Update `version` in `Cargo.toml`.
2. Synchronize `Cargo.lock`.
3. Create a "chore: bump version" commit.
4. Create a Git tag (e.g., `v0.7.4`).

## Manual Checklist

If you cannot run the script, you must manually perform these steps in order:

1. Update `version` in `Cargo.toml`.
2. Run `cargo check` to update `Cargo.lock`.
3. Commit both files: `git commit -m "chore: bump version to X.Y.Z"`.
4. Tag the commit: `git tag vX.Y.Z`.

## Publishing

After bumping and tagging, push to the remote:

```bash
git push origin master --tags
```
