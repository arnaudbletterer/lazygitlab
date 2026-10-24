#!/usr/bin/env python3
"""
Master documentation build & asset generation pipeline for lazygitlab.

Usage:
    python scripts/build_docs.py              # Generate assets & build static site
    python scripts/build_docs.py --serve      # Generate assets & run local dev server
    python scripts/build_docs.py --skip-assets # Build without regenerating SVGs/PDF
"""

import argparse
import json
import os
import re
import subprocess
import sys

# Ensure UTF-8 output on Windows consoles
if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
from datetime import datetime, timezone
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
DOCS_DIR = ROOT_DIR / "docs"
ASSETS_DIR = DOCS_DIR / "assets"
SCREENSHOTS_DIR = ASSETS_DIR / "screenshots"
MARKETING_DIR = ASSETS_DIR / "marketing"
MARKETING_FILES = ["dashboard.png", "mr-review.png", "pipelines.png", "boards.png", "demo.gif"]


def get_git_version():
    """Extracts git tag, commit sha, and dirty status using standard git describe."""
    is_dirty = False
    commit_sha = "unknown"
    tag_name = None
    describe_str = "dev"

    try:
        status_out = subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=ROOT_DIR, text=True
        ).strip()
        is_dirty = len(status_out) > 0
    except Exception:
        pass

    try:
        commit_sha = subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"], cwd=ROOT_DIR, text=True
        ).strip()
    except Exception:
        pass

    try:
        tag_name = subprocess.check_output(
            ["git", "describe", "--tags", "--exact-match"],
            cwd=ROOT_DIR,
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
    except Exception:
        tag_name = None

    try:
        describe_str = subprocess.check_output(
            ["git", "describe", "--tags", "--always", "--dirty"], cwd=ROOT_DIR, text=True
        ).strip()
    except Exception:
        describe_str = f"v0.16.0{'-dirty' if is_dirty else ''}"

    return {
        "tag": tag_name,
        "commit": commit_sha,
        "describe": describe_str,
        "is_dirty": is_dirty,
        "version_type": "release" if tag_name and not is_dirty else "development",
        "display": describe_str,
        "full": describe_str,
        "built_at": datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%SZ"),
    }


def inject_version_metadata(version_info):
    """Writes docs/assets/version.json with the resolved version."""
    print(f"📌 Resolving documentation version: {version_info['full']}")
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)
    version_file = ASSETS_DIR / "version.json"
    version_file.write_text(json.dumps(version_info, indent=2), encoding="utf-8")


def generate_assets():
    """Runs screenshot, PDF cheatsheet, and preview generators."""
    print("🎨 Generating vector screenshots (dark & light) & cheatsheet markdown...")
    SCREENSHOTS_DIR.mkdir(parents=True, exist_ok=True)
    subprocess.check_call(["cargo", "run", "--example", "generate_docs_screenshots"], cwd=ROOT_DIR)

    print("📄 Generating printable 1-page PDF cheatsheet...")
    subprocess.check_call([sys.executable, str(ROOT_DIR / "scripts" / "generate_cheatsheet_pdf.py")], cwd=ROOT_DIR)

    print("🖼️ Generating cheatsheet preview thumbnail...")
    try:
        import fitz  # PyMuPDF
        pdf_path = ASSETS_DIR / "lazygitlab-cheatsheet.pdf"
        out_png = ASSETS_DIR / "lazygitlab-cheatsheet-preview.png"
        doc = fitz.open(str(pdf_path))
        pix = doc[0].get_pixmap(dpi=150)
        pix.save(str(out_png))
        print(f"Thumbnail saved to {out_png}")
    except ImportError:
        print("PyMuPDF not installed, skipping thumbnail generation.")


def generate_marketing():
    """Builds the binary and renders the fictional-data screenshots and demo.gif."""
    print("🎬 Generating marketing screenshots and demo.gif (needs Pillow and ffmpeg)...")
    subprocess.check_call(["cargo", "build"], cwd=ROOT_DIR)
    subprocess.check_call([sys.executable, str(ROOT_DIR / "tools" / "demo" / "make_demo.py")], cwd=ROOT_DIR)


def publish_marketing():
    """Copies marketing assets into the built site and fails if any is missing or empty."""
    import shutil

    target = ROOT_DIR / "site" / "assets" / "marketing"
    target.mkdir(parents=True, exist_ok=True)
    problems = []
    for name in MARKETING_FILES:
        src = MARKETING_DIR / name
        if not src.is_file() or src.stat().st_size == 0:
            problems.append(name)
            continue
        shutil.copy2(src, target / name)
    if problems:
        sys.exit(f"Marketing assets missing or empty: {', '.join(problems)}")
    print(f"🖼️ Marketing assets published to {target}")


def inject_version_into_site_html(version_info):
    """Bakes the version badge directly into compiled static HTML files for zero-blink instant rendering."""
    site_dir = ROOT_DIR / "site"
    if not site_dir.exists():
        return

    display_version = version_info["display"]
    is_dirty = version_info.get("is_dirty", False) or "-dirty" in display_version
    badge_class = "header-version-badge badge-dirty" if is_dirty else "header-version-badge badge-tag"
    badge_html = f'<span class="header-version-container"><span class="{badge_class}"><code>{display_version}</code></span></span>'

    for html_path in site_dir.glob("**/*.html"):
        content = html_path.read_text(encoding="utf-8")
        
        # Target primary header topic
        target_topic = '<div class="md-header__topic">\n          <span class="md-ellipsis">\n            lazygitlab'
        if target_topic in content and "header-version-container" not in content:
            replacement = f'<div class="md-header__topic">\n          <span class="md-ellipsis">\n            lazygitlab\n            {badge_html}'
            content = content.replace(target_topic, replacement, 1)

        html_path.write_text(content, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description="Build lazygitlab documentation suite.")
    parser.add_argument("--serve", action="store_true", help="Start local development server.")
    parser.add_argument("--skip-assets", action="store_true", help="Skip SVG/PDF asset generation.")
    parser.add_argument("--port", default="8000", help="Port for local dev server.")
    args = parser.parse_args()

    version_info = get_git_version()
    inject_version_metadata(version_info)

    if not args.skip_assets:
        generate_assets()
        generate_marketing()

    if args.serve:
        print(f"🚀 Starting Zensical dev server on http://127.0.0.1:{args.port} ...")
        subprocess.call([sys.executable, "-m", "zensical", "serve", "--dev-addr", f"127.0.0.1:{args.port}"], cwd=ROOT_DIR)
    else:
        print("📦 Building static documentation site with Zensical...")
        subprocess.check_call([sys.executable, "-m", "zensical", "build"], cwd=ROOT_DIR)
        inject_version_into_site_html(version_info)
        publish_marketing()
        print("✅ Documentation build complete.")


if __name__ == "__main__":
    main()
