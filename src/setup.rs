use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, BufRead, Write};

const BOLD_CYAN: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const MAX_TOKEN_ATTEMPTS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WizardOutcome {
    pub url: String,
    pub token: String,
    pub save: bool,
}

pub fn run_wizard<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    default_url: &str,
    hide_token: bool,
) -> io::Result<Option<WizardOutcome>> {
    writeln!(out, "\n{BOLD_CYAN}Welcome to lazygitlab{RESET}")?;
    writeln!(
        out,
        "{DIM}No GitLab token found. Let's set one up (Ctrl+C to abort).{RESET}\n"
    )?;

    write!(out, "GitLab URL {DIM}[{default_url}]{RESET}: ")?;
    out.flush()?;
    let typed = match read_line(input)? {
        Some(l) => l,
        None => return Ok(None),
    };
    let url = if typed.is_empty() {
        default_url.to_string()
    } else {
        typed
    };

    writeln!(
        out,
        "{DIM}Create a token with the read_api scope (api to allow actions) in your GitLab profile settings.{RESET}"
    )?;
    let mut token = String::new();
    for _ in 0..MAX_TOKEN_ATTEMPTS {
        write!(out, "Personal access token: ")?;
        out.flush()?;
        let entered = if hide_token {
            let t = read_hidden()?;
            writeln!(out)?;
            Some(t)
        } else {
            read_line(input)?
        };
        match entered {
            None => return Ok(None),
            Some(t) if !t.is_empty() => {
                token = t;
                break;
            }
            Some(_) => writeln!(out, "The token cannot be empty.")?,
        }
    }
    if token.is_empty() {
        return Ok(None);
    }
    if !hide_token {
        writeln!(out, "{DIM}(note: input was not hidden){RESET}")?;
    }

    write!(out, "Save to config.toml? [Y/n]: ")?;
    out.flush()?;
    let answer = read_line(input)?.unwrap_or_default().to_lowercase();
    let save = !matches!(answer.as_str(), "n" | "no");

    Ok(Some(WizardOutcome { url, token, save }))
}

fn read_line<R: BufRead>(input: &mut R) -> io::Result<Option<String>> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_string()))
}

struct RawGuard;

impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

fn read_hidden() -> io::Result<String> {
    enable_raw_mode()?;
    let _guard = RawGuard;
    let mut buf = String::new();
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Enter => break,
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "aborted"));
                }
                KeyCode::Char(c) => buf.push(c),
                _ => {}
            }
        }
    }
    Ok(buf.trim().to_string())
}
