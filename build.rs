use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git rev-parse");
    let git_hash = String::from_utf8(output.stdout).unwrap().trim().to_string();

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("Failed to execute git status");
    let is_dirty = !String::from_utf8(output.stdout).unwrap().trim().is_empty();

    let dirty_suffix = if is_dirty { "-DIRTY" } else { "" };

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_DIRTY={}", dirty_suffix);
    // Ensure we rerun this build script if git changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
