use lazygitlab::cli::{CliError, parse_args, usage, version_line};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn empty_args_yield_defaults() {
    let opts = parse_args(&[]).unwrap();
    assert!(!opts.help && !opts.version && !opts.insecure && !opts.headless);
    assert!(opts.project.is_none() && opts.url.is_none() && opts.token.is_none());
}

#[test]
fn parses_all_flags() {
    let opts = parse_args(&args(&[
        "--project",
        "group/repo",
        "--url",
        "https://git.example.com",
        "--token",
        "abc",
        "--config",
        "/tmp/c.toml",
        "--insecure",
    ]))
    .unwrap();
    assert_eq!(opts.project.as_deref(), Some("group/repo"));
    assert_eq!(opts.url.as_deref(), Some("https://git.example.com"));
    assert_eq!(opts.token.as_deref(), Some("abc"));
    assert_eq!(opts.config.as_deref(), Some("/tmp/c.toml"));
    assert!(opts.insecure);
}

#[test]
fn parses_inline_values_and_short_flags() {
    let opts = parse_args(&args(&["--project=42", "-h", "-V"])).unwrap();
    assert_eq!(opts.project.as_deref(), Some("42"));
    assert!(opts.help && opts.version);
}

#[test]
fn missing_value_is_an_error() {
    assert_eq!(
        parse_args(&args(&["--token"])),
        Err(CliError::MissingValue("--token".into()))
    );
    assert_eq!(
        parse_args(&args(&["--url="])),
        Err(CliError::EmptyValue("--url".into()))
    );
}

#[test]
fn unknown_flag_is_an_error() {
    let err = parse_args(&args(&["--bogus"])).unwrap_err();
    assert_eq!(err, CliError::UnknownFlag("--bogus".into()));
    assert!(err.to_string().contains("--bogus"));
}

#[test]
fn headless_flags_mix_with_app_flags() {
    let opts = parse_args(&args(&[
        "--headless",
        "--width",
        "100",
        "--height=30",
        "--script=s.txt",
        "--frames-dir",
        "out",
        "--project",
        "g/r",
    ]))
    .unwrap();
    assert!(opts.headless);
    assert_eq!(opts.project.as_deref(), Some("g/r"));
    let h = opts.headless_options();
    assert_eq!((h.width, h.height), (100, 30));
    assert_eq!(h.script_path.unwrap().to_str(), Some("s.txt"));
    assert_eq!(h.frames_dir.unwrap().to_str(), Some("out"));
}

#[test]
fn headless_defaults_and_inline_width() {
    let h = parse_args(&args(&["--headless", "--width=80"]))
        .unwrap()
        .headless_options();
    assert_eq!((h.width, h.height), (80, 40));
}

#[test]
fn invalid_dimension_is_an_error() {
    assert_eq!(
        parse_args(&args(&["--width", "abc"])),
        Err(CliError::InvalidValue("--width".into()))
    );
}

#[test]
fn errors_never_echo_values() {
    for list in [
        &["--headless", "--tokn=glpat-secret"][..],
        &["--insecure=glpat-secret"],
        &["--width=glpat-secret"],
        &["--bogus", "--token=glpat-secret"],
    ] {
        let err = parse_args(&args(list)).unwrap_err().to_string();
        assert!(!err.contains("glpat-secret"), "{err}");
    }
    assert_eq!(
        parse_args(&args(&["--tokn=x"])),
        Err(CliError::UnknownFlag("--tokn".into()))
    );
}

#[test]
fn headless_flag_missing_value_is_an_error() {
    assert!(parse_args(&args(&["--headless", "--width"])).is_err());
}

#[test]
fn usage_and_version_mention_version() {
    assert!(usage().contains(env!("CARGO_PKG_VERSION")));
    assert!(usage().contains("--project"));
    assert_eq!(
        version_line(),
        format!("lazygitlab {}", env!("CARGO_PKG_VERSION"))
    );
}
