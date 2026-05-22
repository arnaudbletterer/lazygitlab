#!/usr/bin/env python3
import sys
import re
import subprocess
import os

def run_command(command, description):
    print(f"--- {description} ---")
    try:
        subprocess.run(command, check=True, shell=True)
    except subprocess.CalledProcessError as e:
        print(f"Error: Command failed with exit code {e.returncode}")
        sys.exit(1)

def main():
    if len(sys.argv) != 2 or sys.argv[1] not in ["major", "minor", "patch"]:
        print("Usage: python scripts/bump_version.py [major|minor|patch]")
        sys.exit(1)

    bump_type = sys.argv[1]
    cargo_file = "Cargo.toml"

    if not os.path.exists(cargo_file):
        print(f"Error: {cargo_file} not found.")
        sys.exit(1)

    with open(cargo_file, "r") as f:
        content = f.read()

    # Match version = "X.Y.Z"
    match = re.search(r'version\s*=\s*"(\d+)\.(\d+)\.(\d+)"', content)
    if not match:
        print("Error: Could not find version in Cargo.toml")
        sys.exit(1)

    major, minor, patch = map(int, match.groups())

    if bump_type == "major":
        major += 1
        minor = 0
        patch = 0
    elif bump_type == "minor":
        minor += 1
        patch = 0
    elif bump_type == "patch":
        patch += 1

    new_version = f"{major}.{minor}.{patch}"
    print(f"Bumping version: {match.group(1)}.{match.group(2)}.{match.group(3)} -> {new_version}")

    # Update content
    new_content = re.sub(
        r'version\s*=\s*"\d+\.\d+\.\d+"',
        f'version = "{new_version}"',
        content,
        count=1
    )

    with open(cargo_file, "w") as f:
        f.write(new_content)

    # Synchronize Cargo.lock
    run_command("cargo check", "Synchronizing Cargo.lock")

    # Git Commit
    run_command(f'git add Cargo.toml Cargo.lock && git commit -m "chore: bump version to {new_version}"', "Committing version bump")

    # Git Tag
    run_command(f'git tag -a v{new_version} -m "chore: bump version to {new_version}"', f"Tagging v{new_version}")

    print(f"\nSuccessfully bumped and tagged v{new_version}!")
    print("Run 'git push origin master --tags' to publish both the commit and the tag.")

if __name__ == "__main__":
    main()
