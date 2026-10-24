use std::process::{Command, Stdio};

#[test]
fn env_file_in_working_directory_is_never_read() {
    let dir = std::env::temp_dir().join(format!("lazygitlab-envtest-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(".env"),
        "GITLAB_URL=http://127.0.0.1:9\nGITLAB_TOKEN=glpat-fromDotEnvFile123\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_lazygitlab"))
        .current_dir(&dir)
        .env_remove("GITLAB_URL")
        .env_remove("GITLAB_TOKEN")
        .env("HOME", &dir)
        .env("XDG_STATE_HOME", dir.join("state"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("No GitLab token found"), "{stderr}");
    assert!(!stderr.contains("fromDotEnvFile"), "{stderr}");
    std::fs::remove_dir_all(&dir).unwrap();
}
