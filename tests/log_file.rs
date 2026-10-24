use lazygitlab::logger::{ensure_private_dir, open_log_file, resolve_log_path};
use std::io::Write;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lazygitlab-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(unix)]
#[test]
fn log_file_is_private_even_if_it_already_exists_world_readable() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("perm");
    let path = dir.join("lazygitlab.log");
    std::fs::write(&path, "old\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    let mut file = open_log_file(&path).unwrap();
    writeln!(file, "new").unwrap();
    drop(file);

    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "old\nnew\n");

    let fresh = dir.join("fresh.log");
    drop(open_log_file(&fresh).unwrap());
    let mode = std::fs::metadata(&fresh).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
#[test]
fn symlinked_log_is_refused_and_target_untouched() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("symlink");
    let target = dir.join("victim.txt");
    std::fs::write(&target, "precious").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
    let link = dir.join("lazygitlab.log");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    assert!(open_log_file(&link).is_err());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "precious");
    let mode = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o644);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn directory_is_not_a_log_file() {
    let dir = scratch("isdir");
    assert!(open_log_file(&dir).is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unwritable_location_is_an_error_not_a_panic() {
    let missing = std::env::temp_dir()
        .join("lazygitlab-no-such-dir-xyz")
        .join("lazygitlab.log");
    assert!(open_log_file(&missing).is_err());
}

#[test]
fn log_path_resolution_prefers_explicit_then_state_dirs() {
    let r = |e, x, h, l, w| resolve_log_path(e, x, h, l, w, "1000");
    assert_eq!(
        r(Some("/x/my.log"), Some("/s"), Some("/h"), None, false),
        PathBuf::from("/x/my.log")
    );
    assert_eq!(
        r(None, Some("/s"), Some("/h"), None, false),
        PathBuf::from("/s/lazygitlab/lazygitlab.log")
    );
    assert_eq!(
        r(None, Some(" "), Some("/h"), None, false),
        PathBuf::from("/h/.local/state/lazygitlab/lazygitlab.log")
    );
    assert_eq!(
        r(None, None, None, Some("/l"), false),
        PathBuf::from("/l/lazygitlab/lazygitlab.log")
    );
}

#[test]
fn windows_prefers_local_app_data_over_user_profile() {
    assert_eq!(
        resolve_log_path(None, None, Some("C:/Users/u"), Some("C:/L"), true, "u"),
        PathBuf::from("C:/L/lazygitlab/lazygitlab.log")
    );
    assert_eq!(
        resolve_log_path(None, None, Some("/h"), Some("/l"), false, "u"),
        PathBuf::from("/h/.local/state/lazygitlab/lazygitlab.log")
    );
}

#[test]
fn temp_fallback_is_per_user() {
    let p = resolve_log_path(None, None, None, None, false, "1234");
    assert!(p.to_string_lossy().contains("lazygitlab-1234"), "{p:?}");
}

#[cfg(unix)]
#[test]
fn fifo_cannot_block_startup() {
    let dir = scratch("fifo");
    let fifo = dir.join("lazygitlab.log");
    let c = std::ffi::CString::new(fifo.to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
    let started = std::time::Instant::now();
    assert!(open_log_file(&fifo).is_err());
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
#[test]
fn private_dir_is_created_0700_and_tightened() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("privdir");
    let nested = dir.join("a/b");
    ensure_private_dir(&nested).unwrap();
    let mode = std::fs::metadata(&nested).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    std::fs::set_permissions(&nested, std::fs::Permissions::from_mode(0o755)).unwrap();
    ensure_private_dir(&nested).unwrap();
    let mode = std::fs::metadata(&nested).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
#[test]
fn symlinked_directory_is_refused() {
    let dir = scratch("dirlink");
    let real = dir.join("real");
    std::fs::create_dir_all(&real).unwrap();
    let link = dir.join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    assert!(ensure_private_dir(&link).is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
#[test]
fn symlinked_explicit_log_prints_a_clear_message() {
    let dir = scratch("explicit");
    let target = dir.join("target.txt");
    std::fs::write(&target, "x").unwrap();
    let link = dir.join("my.log");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_lazygitlab"))
        .env("HOME", &dir)
        .env("LAZYGITLAB_LOG", &link)
        .env_remove("GITLAB_TOKEN")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("symbolic link"), "{stderr}");
    assert!(stderr.contains("LAZYGITLAB_LOG"), "{stderr}");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "x");
    std::fs::remove_dir_all(&dir).unwrap();
}
