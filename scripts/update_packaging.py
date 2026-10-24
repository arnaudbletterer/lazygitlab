#!/usr/bin/env python3
"""Fill versions and sha256 hashes in the packaging manifests for a release tag.

Usage:
    python3 scripts/update_packaging.py v0.16.0 [--dry-run] [--sums-file PATH]
"""

import argparse
import re
import sys
import urllib.request
from pathlib import Path

REPO = "arnaudbletterer/lazygitlab"
TAG_RE = re.compile(r"^v\d+\.\d+\.\d+$")
ROOT = Path(__file__).resolve().parent.parent

TARGETS = {
    "x86_64-unknown-linux-gnu": "tar.gz",
    "aarch64-apple-darwin": "tar.gz",
    "x86_64-apple-darwin": "tar.gz",
    "x86_64-pc-windows-msvc": "zip",
}

FILES = {
    "packaging/homebrew/lazygitlab.rb": "homebrew",
    "packaging/aur/lazygitlab-bin/PKGBUILD": "aur-bin",
    "packaging/aur/lazygitlab/PKGBUILD": "aur-src",
    "packaging/scoop/lazygitlab.json": "scoop",
}


def parse_sums(text):
    sums = {}
    for line in text.splitlines():
        parts = line.split(None, 1)
        if len(parts) != 2:
            continue
        digest, name = parts
        name = name.strip().lstrip("*")
        if re.fullmatch(r"[0-9a-f]{64}", digest):
            sums[name] = digest
    return sums


def asset_hash(sums, tag, target):
    name = f"lazygitlab-{tag}-{target}.{TARGETS[target]}"
    try:
        return sums[name]
    except KeyError:
        raise SystemExit(f"error: {name} not found in SHA256SUMS")


def sub_once(pattern, repl, text, flags=0):
    new, count = re.subn(pattern, repl, text, count=1, flags=flags)
    if count != 1:
        raise SystemExit(f"error: pattern not found: {pattern}")
    return new


def update_homebrew(text, tag, sums):
    version = tag[1:]
    text = sub_once(r'(^\s*version ")[^"]*(")', rf"\g<1>{version}\g<2>", text, re.M)
    for target in ("aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu"):
        digest = asset_hash(sums, tag, target)
        text = sub_once(
            rf'(-{re.escape(target)}\.tar\.gz"\s*\n\s*sha256 ")[0-9a-f]+(")',
            rf"\g<1>{digest}\g<2>",
            text,
        )
    return text


def update_aur_bin(text, tag, sums):
    digest = asset_hash(sums, tag, "x86_64-unknown-linux-gnu")
    text = sub_once(r"^pkgver=.*$", f"pkgver={tag[1:]}", text, re.M)
    text = sub_once(r"^pkgrel=.*$", "pkgrel=1", text, re.M)
    return sub_once(
        r"^(sha256sums_x86_64=\(')[0-9a-f]+('\))$", rf"\g<1>{digest}\g<2>", text, re.M
    )


def update_aur_src(text, tag, sums):
    text = sub_once(r"^pkgver=.*$", f"pkgver={tag[1:]}", text, re.M)
    return sub_once(r"^pkgrel=.*$", "pkgrel=1", text, re.M)


def update_scoop(text, tag, sums):
    version = tag[1:]
    target = "x86_64-pc-windows-msvc"
    digest = asset_hash(sums, tag, target)
    text = sub_once(r'^(\s*"version": ")[^"]*(")', rf"\g<1>{version}\g<2>", text, re.M)
    text = sub_once(
        rf'(download/)v[^/"]+(/lazygitlab-)v[^"]*?(-{target}\.zip",\s*\n\s*"hash": ")[0-9a-f]+(")',
        rf"\g<1>{tag}\g<2>{tag}\g<3>{digest}\g<4>",
        text,
    )
    return sub_once(
        rf'^(\s*"extract_dir": ")lazygitlab-v[^"$]*(-{target}")',
        rf"\g<1>lazygitlab-{tag}\g<2>",
        text,
        re.M,
    )


UPDATERS = {
    "homebrew": update_homebrew,
    "aur-bin": update_aur_bin,
    "aur-src": update_aur_src,
    "scoop": update_scoop,
}


def update_all(root, tag, sums, dry_run=False):
    changed = []
    for rel, kind in FILES.items():
        path = Path(root) / rel
        old = path.read_text()
        new = UPDATERS[kind](old, tag, sums)
        if new != old:
            changed.append(rel)
            if not dry_run:
                path.write_text(new)
    return changed


def fetch_sums(tag):
    url = f"https://github.com/{REPO}/releases/download/{tag}/SHA256SUMS"
    with urllib.request.urlopen(url, timeout=30) as resp:
        return resp.read().decode()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("tag", help="release tag, e.g. v0.16.0")
    parser.add_argument("--dry-run", action="store_true", help="report changes without writing")
    parser.add_argument("--sums-file", help="read SHA256SUMS from a local file instead of GitHub")
    parser.add_argument("--root", default=str(ROOT), help="repository root")
    args = parser.parse_args(argv)

    if not TAG_RE.match(args.tag):
        parser.error("tag must look like vX.Y.Z")

    text = Path(args.sums_file).read_text() if args.sums_file else fetch_sums(args.tag)
    sums = parse_sums(text)
    changed = update_all(args.root, args.tag, sums, args.dry_run)

    verb = "would update" if args.dry_run else "updated"
    for rel in changed:
        print(f"{verb} {rel}")
    if not changed:
        print("nothing to change")
    return 0


if __name__ == "__main__":
    sys.exit(main())
