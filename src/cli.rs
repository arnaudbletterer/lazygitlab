#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliOptions {
    pub help: bool,
    pub version: bool,
    pub project: Option<String>,
    pub url: Option<String>,
    pub token: Option<String>,
    pub config: Option<String>,
    pub insecure: bool,
    pub headless: bool,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub script: Option<String>,
    pub frames_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    UnknownFlag(String),
    MissingValue(String),
    EmptyValue(String),
    InvalidValue(String),
    UnexpectedValue(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::UnknownFlag(flag) => write!(f, "unknown argument: {flag}"),
            CliError::MissingValue(flag) => write!(f, "{flag} requires a value"),
            CliError::EmptyValue(flag) => write!(f, "{flag} must not be empty"),
            CliError::InvalidValue(flag) => write!(f, "invalid value for {flag}"),
            CliError::UnexpectedValue(flag) => write!(f, "{flag} does not take a value"),
        }
    }
}

impl std::error::Error for CliError {}

pub fn usage() -> String {
    format!(
        "lazygitlab {version}\n\
A terminal UI for GitLab.\n\
\n\
USAGE:\n    lazygitlab [OPTIONS]\n\
\n\
OPTIONS:\n\
\x20   -h, --help               Print this help and exit\n\
\x20   -V, --version            Print the version and exit\n\
\x20       --project <PATH|ID>  Open this project at startup\n\
\x20       --url <URL>          GitLab instance URL (default: gitlab.com)\n\
\x20       --token <TOKEN>      Personal access token\n\
\x20       --config <PATH>      Config file (default: ~/.config/lazygitlab/config.toml)\n\
\x20       --insecure           Use plain HTTP / skip TLS verification\n\
\x20       --headless           Render off-screen (with --width, --height, --script, --frames-dir)\n\
\n\
ENVIRONMENT:\n\
\x20   GITLAB_URL, GITLAB_TOKEN (a .env file is honoured)\n\
\n\
Precedence: flags > environment > config file > defaults.\n",
        version = env!("CARGO_PKG_VERSION")
    )
}

pub fn version_line() -> String {
    format!("lazygitlab {}", env!("CARGO_PKG_VERSION"))
}

fn split_inline(arg: &str) -> (&str, Option<&str>) {
    match arg.split_once('=') {
        Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
        _ => (arg, None),
    }
}

fn dimension(flag: &str, v: &str) -> Result<u16, CliError> {
    v.parse()
        .map_err(|_| CliError::InvalidValue(flag.to_string()))
}

pub fn parse_args(args: &[String]) -> Result<CliOptions, CliError> {
    let mut opts = CliOptions::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let (flag, inline) = split_inline(arg);
        let mut value = |name: &str| -> Result<String, CliError> {
            let v = match inline {
                Some(v) => v.to_string(),
                None => iter
                    .next()
                    .cloned()
                    .ok_or_else(|| CliError::MissingValue(name.to_string()))?,
            };
            if v.is_empty() {
                return Err(CliError::EmptyValue(name.to_string()));
            }
            Ok(v)
        };
        match flag {
            "-h" | "--help" | "-V" | "--version" | "--insecure" | "--headless"
                if inline.is_some() =>
            {
                return Err(CliError::UnexpectedValue(flag.to_string()));
            }
            "-h" | "--help" => opts.help = true,
            "-V" | "--version" => opts.version = true,
            "--insecure" => opts.insecure = true,
            "--headless" => opts.headless = true,
            "--project" => opts.project = Some(value("--project")?),
            "--url" => opts.url = Some(value("--url")?),
            "--token" => opts.token = Some(value("--token")?),
            "--config" => opts.config = Some(value("--config")?),
            "--width" => opts.width = Some(dimension(flag, &value(flag)?)?),
            "--height" => opts.height = Some(dimension(flag, &value(flag)?)?),
            "--script" => opts.script = Some(value("--script")?),
            "--frames-dir" => opts.frames_dir = Some(value("--frames-dir")?),
            _ => {
                let shown = if arg.starts_with('-') { flag } else { arg.as_str() };
                return Err(CliError::UnknownFlag(shown.to_string()));
            }
        }
    }
    Ok(opts)
}

impl CliOptions {
    pub fn headless_options(&self) -> crate::headless::HeadlessOptions {
        crate::headless::HeadlessOptions {
            width: self.width.unwrap_or(120),
            height: self.height.unwrap_or(40),
            script_path: self.script.as_ref().map(std::path::PathBuf::from),
            frames_dir: self.frames_dir.as_ref().map(std::path::PathBuf::from),
        }
    }
}
