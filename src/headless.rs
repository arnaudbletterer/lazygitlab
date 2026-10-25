use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;

use crate::events::EventSource;
use crate::models::App;

const DEFAULT_UNTIL_TIMEOUT_MS: u64 = 30_000;
const UNTIL_KEY_REPEAT: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Text(String),
    AtLeast { list: String, count: usize },
}

impl Condition {
    fn parse(spec: &str) -> Result<Self, String> {
        if let Some(text) = spec.strip_prefix("text:") {
            if text.is_empty() {
                return Err("until:text: needs a text".into());
            }
            return Ok(Condition::Text(text.to_string()));
        }
        let Some(rest) = spec.strip_prefix('@') else {
            if spec.is_empty() {
                return Err("until: needs a text".into());
            }
            return Ok(Condition::Text(spec.to_string()));
        };
        let (list, count) = rest
            .split_once(">=")
            .ok_or("expected until:@LIST>=COUNT (LIST: mrs, issues, todos, pipelines)")?;
        if !matches!(list, "mrs" | "issues" | "todos" | "pipelines") {
            return Err(format!("unknown list: {list}"));
        }
        let count = count.parse().map_err(|_| "invalid count")?;
        Ok(Condition::AtLeast {
            list: list.to_string(),
            count,
        })
    }

    fn holds(&self, obs: &Observation) -> bool {
        match self {
            Condition::Text(text) => obs.frame.contains(text.as_str()),
            Condition::AtLeast { list, count } => {
                let have = match list.as_str() {
                    "mrs" => obs.counts.mrs,
                    "issues" => obs.counts.issues,
                    "todos" => obs.counts.todos,
                    _ => obs.counts.pipelines,
                };
                have >= *count
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptItem {
    Event(Event),
    Wait(Duration),
    Mark(String),
    Until {
        condition: Condition,
        press: Option<(Event, u32)>,
        timeout: Duration,
    },
}

enum Active {
    Wait {
        deadline: Instant,
    },
    Until {
        condition: Condition,
        press: Option<(Event, u32)>,
        deadline: Instant,
        last_press: Option<Instant>,
        burst_left: u32,
        spec: String,
    },
}

#[derive(Default)]
struct Script {
    items: VecDeque<ScriptItem>,
    active: Option<Active>,
}

pub struct ScriptedEventSource {
    script: Mutex<Script>,
}

impl ScriptedEventSource {
    pub fn from_text(text: &str) -> Result<Self, String> {
        Ok(Self {
            script: Mutex::new(Script {
                items: parse_script_items(text)?.into(),
                active: None,
            }),
        })
    }

    pub fn empty() -> Self {
        Self {
            script: Mutex::new(Script::default()),
        }
    }
}

impl EventSource for ScriptedEventSource {
    async fn next_event(&self, timeout: Duration) -> std::io::Result<Option<Event>> {
        enum Step {
            Return(Option<Event>),
            Sleep(Duration),
        }
        let step = {
            let mut script = self.script.lock().unwrap();
            loop {
                match script.active.take() {
                    Some(Active::Wait { deadline }) => {
                        let now = Instant::now();
                        if now < deadline {
                            script.active = Some(Active::Wait { deadline });
                            break Step::Sleep(timeout.min(deadline - now));
                        }
                    }
                    Some(Active::Until {
                        condition,
                        press,
                        deadline,
                        last_press,
                        mut burst_left,
                        spec,
                    }) => {
                        let satisfied = observe_state(|obs| condition.holds(obs));
                        let now = Instant::now();
                        if satisfied {
                            bench_mark("until", &spec, true);
                        } else if now >= deadline {
                            bench_mark("until", &spec, false);
                        } else {
                            let due = burst_left > 0
                                || last_press
                                    .is_none_or(|t| now.duration_since(t) >= UNTIL_KEY_REPEAT);
                            let mut last_press = last_press;
                            let key = match (&press, due) {
                                (Some((event, count)), true) => {
                                    if burst_left == 0 {
                                        burst_left = *count;
                                    }
                                    burst_left -= 1;
                                    if burst_left == 0 {
                                        last_press = Some(now);
                                    }
                                    Some(event.clone())
                                }
                                _ => None,
                            };
                            script.active = Some(Active::Until {
                                condition,
                                press,
                                deadline,
                                last_press,
                                burst_left,
                                spec,
                            });
                            break match key {
                                Some(key) => Step::Return(Some(key)),
                                None => Step::Sleep(timeout),
                            };
                        }
                    }
                    None => match script.items.pop_front() {
                        None => break Step::Return(None),
                        Some(ScriptItem::Event(event)) => break Step::Return(Some(event)),
                        Some(ScriptItem::Mark(name)) => bench_mark("mark", &name, true),
                        Some(ScriptItem::Wait(d)) => {
                            script.active = Some(Active::Wait {
                                deadline: Instant::now() + d,
                            });
                        }
                        Some(ScriptItem::Until {
                            condition,
                            press,
                            timeout,
                        }) => {
                            let spec = format!("{condition:?}");
                            script.active = Some(Active::Until {
                                condition,
                                press,
                                deadline: Instant::now() + timeout,
                                last_press: None,
                                burst_left: 0,
                                spec,
                            });
                        }
                    },
                }
            }
        };
        match step {
            Step::Return(event) => Ok(event),
            Step::Sleep(d) => {
                tokio::time::sleep(d).await;
                Ok(None)
            }
        }
    }
}

pub fn parse_script(text: &str) -> Result<Vec<Event>, String> {
    parse_script_items(text)?
        .into_iter()
        .map(|item| match item {
            ScriptItem::Event(event) => Ok(event),
            _ => Err("wait:, until: and timeout: need a ScriptedEventSource".to_string()),
        })
        .collect()
}

pub fn parse_script_items(text: &str) -> Result<Vec<ScriptItem>, String> {
    let mut items = Vec::new();
    let mut until_timeout = Duration::from_millis(DEFAULT_UNTIL_TIMEOUT_MS);
    for (line_no, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = |e: String| format!("line {}: {}", line_no + 1, e);
        if let Some(ms) = line.strip_prefix("wait:") {
            let ms: u64 = ms
                .trim()
                .parse()
                .map_err(|_| at("invalid wait ms".into()))?;
            items.push(ScriptItem::Wait(Duration::from_millis(ms)));
        } else if let Some(name) = line.strip_prefix("mark:") {
            items.push(ScriptItem::Mark(name.trim().to_string()));
        } else if let Some(ms) = line.strip_prefix("timeout:") {
            let ms: u64 = ms
                .trim()
                .parse()
                .map_err(|_| at("invalid timeout ms".into()))?;
            until_timeout = Duration::from_millis(ms);
        } else if let Some(spec) = line.strip_prefix("until_key:") {
            let (key, cond) = spec
                .split_once(':')
                .ok_or_else(|| at("expected until_key:KEY:CONDITION".into()))?;
            let (key, count) = match key.split_once('*') {
                Some((k, n)) => {
                    let n: u32 = n
                        .trim()
                        .parse()
                        .map_err(|_| at("invalid key repeat".into()))?;
                    if n < 1 {
                        return Err(at("key repeat must be at least 1".into()));
                    }
                    (k, n)
                }
                None => (key, 1),
            };
            items.push(ScriptItem::Until {
                condition: Condition::parse(cond.trim()).map_err(at)?,
                press: Some((parse_event(key.trim()).map_err(at)?, count)),
                timeout: until_timeout,
            });
        } else if let Some(spec) = line.strip_prefix("until:") {
            items.push(ScriptItem::Until {
                condition: Condition::parse(spec.trim()).map_err(at)?,
                press: None,
                timeout: until_timeout,
            });
        } else {
            items.push(ScriptItem::Event(parse_event(line).map_err(at)?));
        }
    }
    Ok(items)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListCounts {
    pub mrs: usize,
    pub issues: usize,
    pub todos: usize,
    pub pipelines: usize,
}

#[derive(Default)]
pub struct Observation {
    pub frame: String,
    pub counts: ListCounts,
}

struct BenchOut {
    file: std::io::BufWriter<fs::File>,
    last_key: Option<(String, ListCounts, bool)>,
}

static OBSERVING: AtomicBool = AtomicBool::new(false);
static OBSERVATION: OnceLock<Mutex<Observation>> = OnceLock::new();
static BENCH_OUT: OnceLock<Mutex<Option<BenchOut>>> = OnceLock::new();

fn observation() -> &'static Mutex<Observation> {
    OBSERVATION.get_or_init(|| Mutex::new(Observation::default()))
}

fn bench_out() -> &'static Mutex<Option<BenchOut>> {
    BENCH_OUT.get_or_init(|| Mutex::new(None))
}

fn observe_state<T>(f: impl FnOnce(&Observation) -> T) -> T {
    f(&observation().lock().unwrap())
}

pub fn observing() -> bool {
    OBSERVING.load(Ordering::Relaxed)
}

pub fn start_observing(bench_path: Option<&Path>) -> std::io::Result<()> {
    crate::trace::mark_start();
    if let Some(path) = bench_path {
        let file = crate::logger::open_log_file(path)?;
        file.set_len(0)?;
        *bench_out().lock().unwrap() = Some(BenchOut {
            file: std::io::BufWriter::new(file),
            last_key: None,
        });
        bench_write(serde_json::json!({"event": "start"}));
    }
    OBSERVING.store(true, Ordering::Relaxed);
    Ok(())
}

fn elapsed_ms() -> u64 {
    crate::trace::elapsed().as_millis() as u64
}

fn bench_write(mut value: serde_json::Value) {
    let mut guard = bench_out().lock().unwrap();
    let Some(out) = guard.as_mut() else {
        return;
    };
    let (requests, bytes) = crate::trace::totals();
    if let Some(obj) = value.as_object_mut() {
        obj.insert("t_ms".into(), elapsed_ms().into());
        obj.insert("requests".into(), requests.into());
        obj.insert("bytes".into(), bytes.into());
    }
    let _ = writeln!(out.file, "{value}");
    let _ = out.file.flush();
}

fn bench_mark(name: &str, detail: &str, ok: bool) {
    bench_write(serde_json::json!({"event": name, "detail": detail, "ok": ok}));
}

pub fn finish_observing() {
    bench_write(serde_json::json!({"event": "end"}));
}

pub fn observe(buffer: &Buffer, app: &App) {
    let frame = buffer_to_string(buffer);
    let counts = ListCounts {
        mrs: app.merge_requests.len(),
        issues: app.issues.len(),
        todos: app.todos.len(),
        pipelines: if app.active_context == crate::models::Context::Project {
            app.pipelines.len()
        } else {
            app.cross_pipelines.len()
        },
    };
    {
        let mut obs = observation().lock().unwrap();
        obs.frame = frame.clone();
        obs.counts = counts;
    }
    let screen = format!("{:?}", app.active_selection);
    let mut guard = bench_out().lock().unwrap();
    let Some(out) = guard.as_mut() else {
        return;
    };
    let key = (frame.clone(), counts, app.is_loading);
    if out.last_key.as_ref() == Some(&key) {
        return;
    }
    out.last_key = Some(key);
    drop(guard);
    bench_write(serde_json::json!({
        "event": "frame",
        "screen": screen,
        "loading": app.is_loading,
        "mrs": counts.mrs,
        "issues": counts.issues,
        "todos": counts.todos,
        "pipelines": counts.pipelines,
        "frame": frame,
    }));
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
    buffer_to_string(terminal.backend().buffer())
}

pub fn buffer_to_string(buffer: &Buffer) -> String {
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
    pub bench_out: Option<PathBuf>,
}

pub fn load_script(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("failed to read script {}: {}", path.display(), e))
}

#[allow(dead_code)]
pub fn _placeholder_keymap() -> HashMap<String, String> {
    HashMap::new()
}
