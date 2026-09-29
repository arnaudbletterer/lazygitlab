use lazygitlab::redaction::{REDACTED, redact_secrets, redacting, register_secret};
use std::io;
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

const PAT: &str = "glpat-AbCdEf0123456789_xy";
const SELF_HOSTED: &str = "plainSelfHostedSecret42";

#[derive(Clone, Default)]
struct Sink(Arc<Mutex<Vec<u8>>>);

impl io::Write for Sink {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if let Ok(mut g) = self.0.lock() {
            g.extend_from_slice(data);
        }
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Sink {
    type Writer = Sink;
    fn make_writer(&'a self) -> Sink {
        self.clone()
    }
}

#[test]
fn masks_gitlab_token_prefixes() {
    for t in [PAT, "gldt-abcdefgh1234", "glrt-abcdefgh1234", "glcbt-abcdefgh1234"] {
        let input = format!("failed with {t} here");
        let out = redact_secrets(&input);
        assert!(!out.contains(t), "{out}");
        assert!(out.contains(REDACTED));
    }
}

#[test]
fn masks_bearer() {
    let out = redact_secrets("Authorization: Bearer abc.DEF-123_xyz==");
    assert_eq!(out, "Authorization: Bearer [REDACTED]");
}

#[test]
fn masks_private_token_header_and_query() {
    assert_eq!(
        redact_secrets("PRIVATE-TOKEN: hunter2hunter2"),
        "PRIVATE-TOKEN: [REDACTED]"
    );
    assert_eq!(
        redact_secrets("GET /api?private_token=abc123&per_page=5"),
        "GET /api?private_token=[REDACTED]&per_page=5"
    );
    assert_eq!(
        redact_secrets(r#"{"private-token": "abc123"}"#),
        r#"{"private-token": "[REDACTED]"}"#
    );
}

#[test]
fn leaves_normal_text_untouched() {
    for s in [
        "Projects fetch failed: 404 Not Found",
        "global-configuration-management is a group",
        "the bearer of bad news",
        "token expired, please retry",
        "glob-pattern glpat- alone",
    ] {
        assert!(
            matches!(redact_secrets(s), std::borrow::Cow::Borrowed(_)),
            "{s}"
        );
    }
}

#[test]
fn is_idempotent() {
    let s = format!("Bearer abcdefghij {PAT} PRIVATE-TOKEN: xyz private_token=abc");
    let once = redact_secrets(&s).into_owned();
    assert_eq!(redact_secrets(&once), once);
}

#[test]
fn registered_token_never_reaches_log_output() {
    register_secret(SELF_HOSTED);
    assert!(!redact_secrets(&format!("bad {SELF_HOSTED}")).contains(SELF_HOSTED));

    let sink = Sink::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(redacting(sink.clone()))
        .with_ansi(false)
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        tracing::error!("request failed key={SELF_HOSTED} {PAT} Bearer abcdefghijkl");
    });
    let bytes = sink.0.lock().map(|g| g.clone()).unwrap_or_default();
    let written = String::from_utf8_lossy(&bytes).into_owned();
    assert!(written.contains("request failed"));
    assert!(!written.contains(SELF_HOSTED));
    assert!(!written.contains(PAT));
    assert!(!written.contains("abcdefghij12"));
}

#[test]
fn masks_pat_after_newline_and_routable_form() {
    let out = redact_secrets("error:\nglpat-X.01.abcdefgh12 end");
    assert_eq!(out, "error:\n[REDACTED] end");
    assert_eq!(redact_secrets("glpat-abcdefgh1234"), "[REDACTED]");
}

#[test]
fn masks_escaped_json_and_url_encoded_keys() {
    let a = redact_secrets(r#"body: {\"private_token\": \"s3cretvalue\"}"#);
    assert!(!a.contains("s3cretvalue"), "{a}");
    let b = redact_secrets("GET /x?private_token%3Ds3cretvalue&a=1");
    assert_eq!(b, "GET /x?private_token%3D[REDACTED]&a=1");
}

#[test]
fn short_secrets_are_not_registered() {
    register_secret("abc");
    assert!(matches!(
        redact_secrets("abc abc abc"),
        std::borrow::Cow::Borrowed(_)
    ));
}

#[test]
fn bearer_prose_is_untouched() {
    for s in ["Bearer authentication is required", "bearer abcdefghij12"] {
        assert!(
            matches!(redact_secrets(s), std::borrow::Cow::Borrowed(_)),
            "{s}"
        );
    }
}
