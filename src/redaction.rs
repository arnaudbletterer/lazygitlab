use regex::{Captures, Regex};
use std::borrow::Cow;
use std::io::{self, Write};
use std::sync::OnceLock;
use tracing_subscriber::fmt::MakeWriter;

pub const REDACTED: &str = "[REDACTED]";

const MIN_SECRET_LEN: usize = 8;

static CONFIGURED_TOKEN: OnceLock<String> = OnceLock::new();
static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
static BEARER: OnceLock<Option<Regex>> = OnceLock::new();

pub fn register_secret(token: &str) {
    let token = token.trim();
    if token.len() >= MIN_SECRET_LEN {
        let _ = CONFIGURED_TOKEN.set(token.to_string());
    }
}

fn patterns() -> &'static Vec<(Regex, &'static str)> {
    PATTERNS.get_or_init(|| {
        let specs: [(&str, &'static str); 2] = [
            (
                r"(^|[^A-Za-z0-9]|%[0-9A-Fa-f]{2}|\\[nrt])gl(?:pat|dt|ptt|rt|cbt|imt|ft|soat|agent|oas|ffct|rtr)-[A-Za-z0-9_.-]{8,}",
                "${1}[REDACTED]",
            ),
            (
                r#"(?i)((?:private[-_]token|access_token)(?:\\?"|%22)?\s*(?:[:=]|%3D|%3A)\s*(?:\\?"|%22)?)[^\s"&,;\\]+"#,
                "${1}[REDACTED]",
            ),
        ];
        specs
            .into_iter()
            .filter_map(|(re, rep)| Regex::new(re).ok().map(|r| (r, rep)))
            .collect()
    })
}

fn redact_bearer(input: &str) -> Cow<'_, str> {
    let re = BEARER.get_or_init(|| Regex::new(r"\bBearer\s+([A-Za-z0-9._~+/=-]{8,})").ok());
    match re {
        Some(re) => re.replace_all(input, |caps: &Captures| {
            let tokenish = caps[1]
                .chars()
                .any(|c| c.is_ascii_digit() || "._~+/=-".contains(c));
            if tokenish {
                "Bearer [REDACTED]".to_string()
            } else {
                caps[0].to_string()
            }
        }),
        None => Cow::Borrowed(input),
    }
}

pub fn redact_secrets(input: &str) -> Cow<'_, str> {
    let mut out = Cow::Borrowed(input);
    if let Some(token) = CONFIGURED_TOKEN.get()
        && out.contains(token.as_str())
    {
        out = Cow::Owned(out.replace(token.as_str(), REDACTED));
    }
    for (re, rep) in patterns() {
        if re.is_match(&out) {
            out = Cow::Owned(re.replace_all(&out, *rep).into_owned());
        }
    }
    if let Cow::Owned(s) = redact_bearer(&out)
        && s != out
    {
        out = Cow::Owned(s);
    }
    out
}

pub struct RedactingMakeWriter<M>(M);

pub fn redacting<M>(inner: M) -> RedactingMakeWriter<M> {
    RedactingMakeWriter(inner)
}

pub struct RedactingWriter<W: Write> {
    inner: W,
    buf: Vec<u8>,
}

impl<W: Write> RedactingWriter<W> {
    fn drain(&mut self) -> io::Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        let text = String::from_utf8_lossy(&self.buf).into_owned();
        self.buf.clear();
        self.inner.write_all(redact_secrets(&text).as_bytes())
    }
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.drain()?;
        self.inner.flush()
    }
}

impl<W: Write> Drop for RedactingWriter<W> {
    fn drop(&mut self) {
        let _ = self.drain();
        let _ = self.inner.flush();
    }
}

impl<'a, M> MakeWriter<'a> for RedactingMakeWriter<M>
where
    M: MakeWriter<'a>,
{
    type Writer = RedactingWriter<M::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter {
            inner: self.0.make_writer(),
            buf: Vec::new(),
        }
    }
}
