use lazygitlab::setup::run_wizard;
use std::io::Cursor;

fn run(input: &str) -> (Option<lazygitlab::setup::WizardOutcome>, String) {
    run_with_default(input, "gitlab.com")
}

fn run_with_default(
    input: &str,
    default_url: &str,
) -> (Option<lazygitlab::setup::WizardOutcome>, String) {
    let mut reader = Cursor::new(input.as_bytes().to_vec());
    let mut out = Vec::new();
    let outcome = run_wizard(&mut reader, &mut out, default_url, false).unwrap();
    (outcome, String::from_utf8(out).unwrap())
}

#[test]
fn accepts_custom_url_token_and_saves_by_default() {
    let (outcome, out) = run("git.example.com\nsecret\n\n");
    let o = outcome.unwrap();
    assert_eq!(o.url, "git.example.com");
    assert_eq!(o.token, "secret");
    assert!(o.save);
    assert!(out.contains("Save to config.toml? [Y/n]"));
    assert!(!out.contains("secret"));
}

#[test]
fn empty_url_uses_default() {
    let (outcome, _) = run("\nsecret\ny\n");
    assert_eq!(outcome.unwrap().url, "gitlab.com");
}

#[test]
fn declining_save_is_respected() {
    for answer in ["n", "N", "no"] {
        let (outcome, _) = run(&format!("\nsecret\n{answer}\n"));
        assert!(!outcome.unwrap().save, "answer {answer}");
    }
}

#[test]
fn empty_token_reprompts_then_accepts() {
    let (outcome, out) = run("\n\nsecret\n\n");
    assert_eq!(outcome.unwrap().token, "secret");
    assert!(out.contains("cannot be empty"));
}

#[test]
fn repeated_empty_tokens_abort() {
    let (outcome, _) = run("\n\n\n\n\n");
    assert!(outcome.is_none());
}

#[test]
fn eof_aborts_without_panic() {
    assert!(run("").0.is_none());
    assert!(run("gitlab.com\n").0.is_none());
}

#[test]
fn missing_save_answer_defaults_to_yes() {
    let (outcome, _) = run("\nsecret\n");
    assert!(outcome.unwrap().save);
}

#[test]
fn prompt_default_is_the_resolved_url() {
    let (outcome, out) = run_with_default("\nsecret\n\n", "git.corp.example");
    assert!(out.contains("[git.corp.example]"));
    assert_eq!(outcome.unwrap().url, "git.corp.example");
}
