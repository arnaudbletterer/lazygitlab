use lazygitlab::cli::CliOptions;
use lazygitlab::config::{
    FileConfig, Settings, load_file_config, normalize_url, parse_file_config, save_gitlab_section,
};
use lazygitlab::ui::widgets::Theme;

fn file(url: &str, token: &str) -> FileConfig {
    parse_file_config(&format!(
        "[gitlab]\nurl = \"{url}\"\ntoken = \"{token}\"\nproject = \"g/file\"\n"
    ))
    .unwrap()
}

#[test]
fn defaults_apply_without_any_source() {
    let s = Settings::resolve(&CliOptions::default(), None, None, &FileConfig::default());
    assert_eq!(s.url, "gitlab.com");
    assert_eq!(s.token, None);
    assert!(!s.insecure);
    assert_eq!(s.project, None);
}

#[test]
fn file_beats_defaults() {
    let s = Settings::resolve(&CliOptions::default(), None, None, &file("a.example", "ft"));
    assert_eq!(s.url, "a.example");
    assert_eq!(s.token.as_deref(), Some("ft"));
    assert_eq!(s.project.as_deref(), Some("g/file"));
}

#[test]
fn env_beats_file() {
    let s = Settings::resolve(
        &CliOptions::default(),
        Some("env.example"),
        Some("et"),
        &file("a.example", "ft"),
    );
    assert_eq!(s.url, "env.example");
    assert_eq!(s.token.as_deref(), Some("et"));
}

#[test]
fn cli_beats_env_and_file() {
    let cli = CliOptions {
        url: Some("cli.example".into()),
        token: Some("ct".into()),
        project: Some("g/cli".into()),
        ..Default::default()
    };
    let s = Settings::resolve(
        &cli,
        Some("env.example"),
        Some("et"),
        &file("a.example", "ft"),
    );
    assert_eq!(s.url, "cli.example");
    assert_eq!(s.token.as_deref(), Some("ct"));
    assert_eq!(s.project.as_deref(), Some("g/cli"));
}

#[test]
fn empty_env_values_fall_through() {
    let s = Settings::resolve(
        &CliOptions::default(),
        Some(""),
        Some(""),
        &file("a.example", "ft"),
    );
    assert_eq!(s.url, "a.example");
    assert_eq!(s.token.as_deref(), Some("ft"));
}

#[test]
fn url_normalization_and_insecure() {
    assert_eq!(normalize_url("https://x.com//"), ("x.com".into(), false));
    assert_eq!(normalize_url("http://x.com/"), ("x.com".into(), true));
    let s = Settings::resolve(
        &CliOptions::default(),
        Some("http://x.com/"),
        Some("t"),
        &FileConfig::default(),
    );
    assert_eq!(s.url, "x.com");
    assert!(s.insecure);
    let cli = CliOptions {
        insecure: true,
        ..Default::default()
    };
    assert!(Settings::resolve(&cli, None, None, &FileConfig::default()).insecure);
    let f = parse_file_config("[gitlab]\ninsecure = true\n").unwrap();
    assert!(Settings::resolve(&CliOptions::default(), None, None, &f).insecure);
}

#[test]
fn load_missing_file_is_default_unless_required() {
    let dir = tempdir("missing");
    let path = dir.join("nope.toml");
    assert_eq!(
        load_file_config(&path, false).unwrap(),
        FileConfig::default()
    );
    assert!(load_file_config(&path, true).is_err());
}

#[test]
fn invalid_toml_is_an_error() {
    let dir = tempdir("invalid");
    let path = dir.join("c.toml");
    std::fs::write(&path, "[gitlab\n").unwrap();
    assert!(load_file_config(&path, false).is_err());
}

#[test]
fn theme_only_and_mixed_files_parse_both_ways() {
    let mixed = "theme = \"gruvbox\"\n[gitlab]\nurl = \"x\"\ntoken = \"t\"\n";
    assert!(parse_file_config(mixed).is_ok());
    assert!(Theme::parse_theme_content(mixed).is_some());
    let theme_only = "theme = \"gruvbox\"\n";
    assert_eq!(
        parse_file_config(theme_only).unwrap(),
        FileConfig::default()
    );
}

#[test]
fn save_creates_file_and_merges_preserving_other_tables() {
    let dir = tempdir("save");
    let path = dir.join("nested").join("config.toml");
    save_gitlab_section(&path, "git.example", "tok").unwrap();
    let f = load_file_config(&path, true).unwrap();
    assert_eq!(f.gitlab.url.as_deref(), Some("git.example"));
    assert_eq!(f.gitlab.token.as_deref(), Some("tok"));

    std::fs::write(
        &path,
        "theme = \"gruvbox\"\n[gitlab]\nproject = \"g/r\"\ninsecure = true\n[extra]\nk = 1\n",
    )
    .unwrap();
    save_gitlab_section(&path, "new.example", "tok2").unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let table: toml::Table = text.parse().unwrap();
    assert_eq!(table["theme"].as_str(), Some("gruvbox"));
    assert_eq!(table["extra"]["k"].as_integer(), Some(1));
    assert_eq!(table["gitlab"]["project"].as_str(), Some("g/r"));
    assert_eq!(table["gitlab"]["insecure"].as_bool(), Some(true));
    assert_eq!(table["gitlab"]["url"].as_str(), Some("new.example"));
    assert_eq!(table["gitlab"]["token"].as_str(), Some("tok2"));
    assert!(Theme::parse_theme_content(&text).is_some());
}

#[test]
fn save_refuses_to_overwrite_invalid_toml() {
    let dir = tempdir("refuse");
    let path = dir.join("c.toml");
    std::fs::write(&path, "not [valid").unwrap();
    assert!(save_gitlab_section(&path, "u", "t").is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "not [valid");
}

#[cfg(unix)]
#[test]
fn saved_file_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempdir("perm");
    let path = dir.join("c.toml");
    std::fs::write(&path, "theme = \"gruvbox\"\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    save_gitlab_section(&path, "u", "t").unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lazygitlab-config-test-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn malformed_config_errors_never_leak_content() {
    let dir = tempdir("leak");
    let path = dir.join("c.toml");
    std::fs::write(&path, "[gitlab]\nurl = \"x\"\ntoken = glpat-topsecret\n").unwrap();
    let load = load_file_config(&path, false).unwrap_err();
    let save = save_gitlab_section(&path, "u", "t").unwrap_err();
    for msg in [&load, &save] {
        assert!(!msg.contains("glpat"), "{msg}");
        assert!(!msg.contains("topsecret"), "{msg}");
        assert!(msg.contains("line 3"), "{msg}");
    }
    assert!(load.contains("c.toml"));
}

#[test]
fn wrong_type_error_does_not_leak_value() {
    let err = parse_file_config("[gitlab]\ntoken = 12345678\n").unwrap_err();
    assert!(!err.contains("12345678"), "{err}");
}

#[test]
fn file_insecure_does_not_override_explicit_https_url() {
    let f = parse_file_config("[gitlab]\ninsecure = true\n").unwrap();
    let cli = CliOptions {
        url: Some("https://secure.example".into()),
        ..Default::default()
    };
    assert!(!Settings::resolve(&cli, None, None, &f).insecure);
    assert!(
        !Settings::resolve(&CliOptions::default(), Some("https://e.example"), None, &f).insecure
    );
    let f2 = parse_file_config("[gitlab]\nurl = \"h.example\"\ninsecure = true\n").unwrap();
    assert!(Settings::resolve(&CliOptions::default(), None, None, &f2).insecure);
}

#[test]
fn save_leaves_no_temp_files_and_replaces_atomically() {
    let dir = tempdir("atomic");
    let path = dir.join("c.toml");
    std::fs::write(&path, "theme = \"gruvbox\"\n").unwrap();
    save_gitlab_section(&path, "u", "t").unwrap();
    let names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["c.toml".to_string()]);
}

#[test]
fn insecure_flag_skips_certificate_verification_over_https() {
    use lazygitlab::config::Transport;
    let cli = CliOptions {
        insecure: true,
        url: Some("https://gitlab.internal.corp".into()),
        ..Default::default()
    };
    let s = Settings::resolve(&cli, None, Some("t"), &FileConfig::default());
    assert_eq!(s.transport(), Transport::HttpsSkipVerify);
    let s = Settings::resolve(
        &CliOptions::default(),
        Some("http://x.com"),
        Some("t"),
        &FileConfig::default(),
    );
    assert_eq!(s.transport(), Transport::PlainHttp);
    let s = Settings::resolve(
        &CliOptions::default(),
        Some("https://x.com"),
        Some("t"),
        &FileConfig::default(),
    );
    assert_eq!(s.transport(), Transport::Https);
}

#[test]
fn url_with_path_prefix_and_scheme_case_is_normalized() {
    assert_eq!(
        normalize_url("https://host.example/gitlab/"),
        ("host.example/gitlab".into(), false)
    );
    assert_eq!(
        normalize_url("HTTPS://Host.example/gitlab/api/v4/"),
        ("Host.example/gitlab".into(), false)
    );
    assert_eq!(normalize_url("HTTP://x.com"), ("x.com".into(), true));
}
