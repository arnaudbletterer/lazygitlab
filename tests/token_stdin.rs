use lazygitlab::cli::{read_token_line, usage};

#[test]
fn reads_first_line_and_trims() {
    let mut input = std::io::Cursor::new("glpat-abc123\nsecond\n");
    assert_eq!(read_token_line(&mut input).unwrap(), "glpat-abc123");
}

#[test]
fn empty_stdin_is_an_actionable_error() {
    let mut input = std::io::Cursor::new("\n");
    let err = read_token_line(&mut input).unwrap_err();
    assert!(err.contains("first line of stdin"), "{err}");
    let mut input = std::io::Cursor::new("");
    assert!(read_token_line(&mut input).is_err());
}

#[test]
fn help_warns_about_token_flag() {
    let text = usage();
    assert!(text.contains("process list"), "{text}");
    assert!(text.contains("--token -"), "{text}");
}
