use std::process::Command;

fn main() {
    let describe_output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output();

    let full_version = match describe_output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                s
            } else {
                format!("v{}", env!("CARGO_PKG_VERSION"))
            }
        }
        _ => format!("v{}", env!("CARGO_PKG_VERSION")),
    };

    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let is_dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    let dirty_suffix = if is_dirty { "-dirty" } else { "" };

    println!("cargo:rustc-env=LAZYGITLAB_VERSION={}", full_version);
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_DIRTY={}", dirty_suffix);
    // Ensure we rerun this build script if git changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
