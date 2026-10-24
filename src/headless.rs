use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use crate::events::EventSource;

pub struct ScriptedEventSource {
    events: std::sync::Mutex<VecDeque<Event>>,
}

impl ScriptedEventSource {
    pub fn from_text(text: &str) -> Result<Self, String> {
        let events: VecDeque<Event> = parse_script(text)?.into();
        Ok(Self {
            events: std::sync::Mutex::new(events),
        })
    }

    pub fn empty() -> Self {
        Self {
            events: std::sync::Mutex::new(VecDeque::new()),
        }
    }
}

impl EventSource for ScriptedEventSource {
    async fn next_event(&self, _timeout: Duration) -> std::io::Result<Option<Event>> {
        let mut events = self.events.lock().unwrap();
        Ok(events.pop_front())
    }
}

pub fn parse_script(text: &str) -> Result<Vec<Event>, String> {
    let mut events = Vec::new();
    for (line_no, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let event = parse_event(line).map_err(|e| format!("line {}: {}", line_no + 1, e))?;
        events.push(event);
    }
    Ok(events)
}

fn parse_event(line: &str) -> Result<Event, String> {
    if let Some(rest) = line.strip_prefix("mouse:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() < 3 {
            return Err("expected mouse:KIND:COL:ROW".into());
        }
        let kind = match parts[0] {
            "down" => MouseEventKind::Down(MouseButton::Left),
            "up" => MouseEventKind::Up(MouseButton::Left),
            "scroll_up" => MouseEventKind::ScrollUp,
            "scroll_down" => MouseEventKind::ScrollDown,
            other => return Err(format!("unknown mouse kind: {other}")),
        };
        let col: u16 = parts[1].parse().map_err(|_| "invalid column")?;
        let row: u16 = parts[2].parse().map_err(|_| "invalid row")?;
        return Ok(Event::Mouse(MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }));
    }
    if let Some(rest) = line.strip_prefix("key:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() < 2 {
            return Err("expected key:CODE[:MODIFIERS]".into());
        }
        let code = parse_key_code(parts[0])?;
        let modifiers = parse_modifiers(parts.get(1).copied().unwrap_or(""));
        return Ok(Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));
    }
    let (modifiers_str, code_str) = if let Some(rest) = line.strip_prefix('<') {
        let end = rest.find('>').ok_or("unterminated <...>")?;
        let tag = &rest[..end];
        let after = rest[end + 1..].trim_start();
        let (modifiers, code_part) = if after.is_empty() {
            let (mods, code) = split_modifiers_and_code(tag);
            (mods, code.to_string())
        } else {
            (parse_modifiers(tag), after.to_string())
        };
        (modifiers, code_part)
    } else {
        (KeyModifiers::NONE, line.to_string())
    };
    if code_str == "quit" {
        return Ok(Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));
    }
    let code = parse_key_code(&code_str)?;
    Ok(Event::Key(KeyEvent {
        code,
        modifiers: modifiers_str,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    }))
}

fn parse_key_code(s: &str) -> Result<KeyCode, String> {
    static ALIASES: &[(&str, KeyCode)] = &[
        ("enter", KeyCode::Enter),
        ("esc", KeyCode::Esc),
        ("escape", KeyCode::Esc),
        ("tab", KeyCode::Tab),
        ("backtab", KeyCode::BackTab),
        ("backspace", KeyCode::Backspace),
        ("bs", KeyCode::Backspace),
        ("delete", KeyCode::Delete),
        ("del", KeyCode::Delete),
        ("home", KeyCode::Home),
        ("end", KeyCode::End),
        ("up", KeyCode::Up),
        ("down", KeyCode::Down),
        ("left", KeyCode::Left),
        ("right", KeyCode::Right),
        ("pageup", KeyCode::PageUp),
        ("pagedown", KeyCode::PageDown),
        ("space", KeyCode::Char(' ')),
        ("spc", KeyCode::Char(' ')),
    ];
    let lower = s.to_ascii_lowercase();
    for (alias, code) in ALIASES {
        if *alias == lower {
            return Ok(*code);
        }
    }
    if lower.starts_with('f')
        && lower.len() > 1
        && let Ok(n) = lower[1..].parse::<u8>()
    {
        return Ok(KeyCode::F(n));
    }
    let mut chars = s.chars();
    if let Some(c) = chars.next()
        && chars.next().is_none()
    {
        return Ok(KeyCode::Char(c));
    }
    Err(format!("unknown key code: {s}"))
}

fn parse_modifiers(s: &str) -> KeyModifiers {
    let mut out = KeyModifiers::NONE;
    for part in s.split('-').filter(|p| !p.is_empty()) {
        match part.to_ascii_uppercase().as_str() {
            "S" | "SHIFT" => out |= KeyModifiers::SHIFT,
            "C" | "CTRL" | "CONTROL" => out |= KeyModifiers::CONTROL,
            "A" | "ALT" => out |= KeyModifiers::ALT,
            _ => {}
        }
    }
    out
}

fn split_modifiers_and_code(tag: &str) -> (KeyModifiers, &str) {
    let mut mods = KeyModifiers::NONE;
    let mut code_start = 0;
    for (i, part) in tag.split('-').enumerate() {
        if i == tag.split('-').count() - 1 {
            code_start = tag.len() - part.len();
            break;
        }
        let recognized = matches!(
            part.to_ascii_uppercase().as_str(),
            "S" | "SHIFT" | "C" | "CTRL" | "CONTROL" | "A" | "ALT"
        );
        if recognized {
            mods |= parse_modifiers(part);
        } else {
            code_start = tag.len() - part.len();
            break;
        }
    }
    (mods, &tag[code_start..])
}

pub struct FrameCapture {
    dir: PathBuf,
    tick: usize,
    last_frame: Option<String>,
}

impl FrameCapture {
    pub fn new(dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            tick: 0,
            last_frame: None,
        })
    }

    pub fn disabled() -> Self {
        Self {
            dir: PathBuf::new(),
            tick: 0,
            last_frame: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.dir.as_os_str().is_empty()
    }

    pub fn record(&mut self, terminal: &Terminal<TestBackend>) -> std::io::Result<Option<PathBuf>> {
        if !self.is_enabled() {
            return Ok(None);
        }
        let frame = render_to_string(terminal);
        if self.last_frame.as_deref() == Some(&frame) {
            return Ok(None);
        }
        self.last_frame = Some(frame.clone());
        let path = self.dir.join(format!("frame_{:06}.txt", self.tick));
        self.tick += 1;
        fs::write(&path, frame)?;
        Ok(Some(path))
    }
}

pub fn render_to_string(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer().clone();
    let mut out = String::new();
    let area = buffer.area;
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buffer[(x, y)];
            let symbol = cell.symbol();
            if symbol.is_empty() {
                out.push(' ');
            } else {
                out.push_str(symbol);
            }
        }
        out.push('\n');
    }
    out
}

pub fn headless_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("failed to create TestBackend terminal")
}

#[derive(Default)]
pub struct HeadlessOptions {
    pub width: u16,
    pub height: u16,
    pub script_path: Option<PathBuf>,
    pub frames_dir: Option<PathBuf>,
}

pub fn load_script(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("failed to read script {}: {}", path.display(), e))
}

#[allow(dead_code)]
pub fn _placeholder_keymap() -> HashMap<String, String> {
    HashMap::new()
}
