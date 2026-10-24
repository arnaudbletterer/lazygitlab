# Packaging

Manifests for distributing lazygitlab. They consume the assets published by `.github/workflows/release.yml` on every `v*` tag: `lazygitlab-<tag>-<target>.tar.gz|zip` plus `SHA256SUMS`.

| Path | Channel |
| --- | --- |
| `homebrew/lazygitlab.rb` | Personal Homebrew tap |
| `aur/lazygitlab-bin/PKGBUILD` | AUR, prebuilt binary (x86_64) |
| `aur/lazygitlab/PKGBUILD` | AUR, built from source |
| `scoop/lazygitlab.json` | Personal Scoop bucket |
| `install.sh` | curl-able installer |

All hashes in the repo are placeholders (64 zeros, or `SKIP`) until you fill them for a real release.

## After each release

1. Push the tag `vX.Y.Z` and wait for the release workflow to publish the assets.
2. Fill versions and hashes:

   ```bash
   python3 scripts/update_packaging.py vX.Y.Z --dry-run   # preview
   python3 scripts/update_packaging.py vX.Y.Z             # write
   git diff packaging/
   ```

   The script downloads `SHA256SUMS` from the release. Use `--sums-file PATH` to read a local copy. Only stable tags (`vX.Y.Z`) are accepted, and it fails without writing anything if a placeholder would remain.
3. Commit with `chore(packaging): update manifests for vX.Y.Z`, then submit to each channel below.

Tests: `python3 -I scripts/test_update_packaging.py` (or `python3 -I -m unittest discover -s scripts -p 'test_update_packaging.py'`). The `-m unittest scripts/test_update_packaging.py` form needs the repository root on `sys.path`, so run it without `-I`.

## Homebrew (personal tap)

One-time setup: create the GitHub repository `arnaudbletterer/homebrew-tap` (the `homebrew-` prefix is required).

```bash
git clone git@github.com:arnaudbletterer/homebrew-tap.git
mkdir -p homebrew-tap/Formula
cp packaging/homebrew/lazygitlab.rb homebrew-tap/Formula/
cd homebrew-tap
git add Formula/lazygitlab.rb
git commit -m "lazygitlab X.Y.Z"
git push
```

Users install with `brew install arnaudbletterer/tap/lazygitlab`. Validate before pushing: `brew audit --strict --new arnaudbletterer/tap/lazygitlab` and `brew install arnaudbletterer/tap/lazygitlab` on a test machine.

## AUR

One-time setup: create an AUR account and add your SSH public key to it. Each package is its own git repository on `aur.archlinux.org`.

```bash
# prebuilt binary
git clone ssh://aur@aur.archlinux.org/lazygitlab-bin.git aur-lazygitlab-bin
cp packaging/aur/lazygitlab-bin/PKGBUILD aur-lazygitlab-bin/
cd aur-lazygitlab-bin
makepkg --printsrcinfo > .SRCINFO
makepkg -f            # local build check; namcap PKGBUILD if installed
git add PKGBUILD .SRCINFO
git commit -m "Update to X.Y.Z"
git push
```

Repeat for `lazygitlab` (source package) with `packaging/aur/lazygitlab/PKGBUILD`. The source PKGBUILD hash is the GitHub tag tarball hash, computed by the update script (downloaded, or pass `--source-tarball PATH`). Bump `pkgrel` instead of `pkgver` for packaging-only fixes.

## Scoop (personal bucket)

One-time setup: create the GitHub repository `arnaudbletterer/scoop-bucket` with a `bucket/` directory.

```bash
git clone git@github.com:arnaudbletterer/scoop-bucket.git
mkdir -p scoop-bucket/bucket
cp packaging/scoop/lazygitlab.json scoop-bucket/bucket/
cd scoop-bucket
git add bucket/lazygitlab.json
git commit -m "lazygitlab: update to X.Y.Z"
git push
```

Users install with `scoop bucket add arnaudbletterer https://github.com/arnaudbletterer/scoop-bucket` then `scoop install arnaudbletterer/lazygitlab`. The manifest has `checkver` and `autoupdate` blocks, so Scoop tooling (`checkver.ps1 -u`) can also bump it.

## install.sh

Nothing to submit: it is served from the repository.

```bash
curl -fsSL https://raw.githubusercontent.com/arnaudbletterer/lazygitlab/master/packaging/install.sh | bash
```

It supports Linux x86_64 and macOS (arm64, x86_64), verifies the archive against `SHA256SUMS`, installs to `~/.local/bin`, and refuses to run as root unless `LAZYGITLAB_ALLOW_ROOT=1`. Pin a version with `LAZYGITLAB_VERSION=vX.Y.Z`. Lint with `bash -n packaging/install.sh` and `shellcheck packaging/install.sh`.

## crates.io

Do not publish before a dry run. `Cargo.toml` currently lacks `description`, `license` and `repository` metadata, which crates.io requires, so add them first.

```bash
cargo publish --dry-run
cargo package --list      # check that no secrets or large files are included
cargo login               # token from crates.io account settings, never committed
cargo publish
```

Publishing is permanent: a version can be yanked but never deleted or overwritten.
