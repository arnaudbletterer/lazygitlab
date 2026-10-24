use lazygitlab::logger::{open_log_file, resolve_log_path};
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
    assert_eq!(
        resolve_log_path(Some("/x/my.log"), Some("/s"), Some("/h"), None),
        PathBuf::from("/x/my.log")
    );
    assert_eq!(
        resolve_log_path(None, Some("/s"), Some("/h"), None),
        PathBuf::from("/s/lazygitlab/lazygitlab.log")
    );
    assert_eq!(
        resolve_log_path(None, Some(" "), Some("/h"), None),
        PathBuf::from("/h/.local/state/lazygitlab/lazygitlab.log")
    );
    assert_eq!(
        resolve_log_path(None, None, None, Some("/l")),
        PathBuf::from("/l/lazygitlab/lazygitlab.log")
    );
    assert!(resolve_log_path(None, None, None, None).ends_with("lazygitlab/lazygitlab.log"));
}
