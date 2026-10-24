use lazygitlab::logger::open_log_file;
use std::io::Write;

#[cfg(unix)]
#[test]
fn log_file_is_private_even_if_it_already_exists_world_readable() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("lazygitlab-log-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
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

#[test]
fn unwritable_location_is_an_error_not_a_panic() {
    let missing = std::env::temp_dir()
        .join("lazygitlab-no-such-dir-xyz")
        .join("lazygitlab.log");
    assert!(open_log_file(&missing).is_err());
}
