use crate::cli::CliOptions;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const DEFAULT_URL: &str = "gitlab.com";

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct GitlabSection {
    pub url: Option<String>,
    pub token: Option<String>,
    pub insecure: Option<bool>,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub gitlab: GitlabSection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub url: String,
    pub token: Option<String>,
    pub insecure: bool,
    pub plain_http: bool,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Https,
    PlainHttp,
    HttpsSkipVerify,
}

pub const SKIP_VERIFY_WARNING: &str = "WARNING: TLS certificate verification is disabled (--insecure / insecure = true). \
Your token can be intercepted by anyone on the network path. Prefer adding your CA to the system trust store.";

pub fn default_config_path() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        return Some(
            PathBuf::from(home)
                .join(".config")
                .join("lazygitlab")
                .join("config.toml"),
        );
    }
    std::env::var("APPDATA")
        .ok()
        .map(|d| PathBuf::from(d).join("lazygitlab").join("config.toml"))
}

pub fn describe_toml_error(content: &str, e: &toml::de::Error) -> String {
    let offset = e.span().map(|s| s.start).unwrap_or(0).min(content.len());
    let before = content.get(..offset).unwrap_or("");
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    let message = e.message();
    let end = message.find(['`', '"', '\'']).unwrap_or(message.len());
    let message = message[..end].trim_end();
    format!("invalid config at line {line}, column {column}: {message}")
}

pub fn parse_file_config(content: &str) -> Result<FileConfig, String> {
    toml::from_str(content).map_err(|e| describe_toml_error(content, &e))
}

pub fn load_file_config(path: &Path, required: bool) -> Result<FileConfig, String> {
    match std::fs::read_to_string(path) {
        Ok(content) => parse_file_config(&content).map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && !required => {
            Ok(FileConfig::default())
        }
        Err(e) => Err(format!("cannot read {}: {e}", path.display())),
    }
}

pub fn normalize_url(raw: &str) -> (String, bool) {
    let mut url = raw.trim().to_string();
    let mut insecure = false;
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("http://") {
        insecure = true;
        url.drain(.."http://".len());
    } else if lower.starts_with("https://") {
        url.drain(.."https://".len());
    }
    while url.ends_with('/') {
        url.pop();
    }
    if url.to_ascii_lowercase().ends_with("/api/v4") {
        url.truncate(url.len() - "/api/v4".len());
    }
    while url.ends_with('/') {
        url.pop();
    }
    (url, insecure)
}

fn non_empty(v: Option<&str>) -> Option<String> {
    v.map(str::trim).filter(|s| !s.is_empty()).map(String::from)
}

impl Settings {
    pub fn resolve(
        cli: &CliOptions,
        env_url: Option<&str>,
        env_token: Option<&str>,
        file: &FileConfig,
    ) -> Self {
        let explicit_url = non_empty(cli.url.as_deref()).or_else(|| non_empty(env_url));
        let from_file = explicit_url.is_none();
        let raw_url = explicit_url
            .or_else(|| non_empty(file.gitlab.url.as_deref()))
            .unwrap_or_else(|| DEFAULT_URL.to_string());
        let (mut url, http) = normalize_url(&raw_url);
        if url.is_empty() {
            url = DEFAULT_URL.to_string();
        }
        let token = non_empty(cli.token.as_deref())
            .or_else(|| non_empty(env_token))
            .or_else(|| non_empty(file.gitlab.token.as_deref()));
        let file_insecure = from_file && file.gitlab.insecure.unwrap_or(false);
        let insecure = cli.insecure || http || file_insecure;
        let project =
            non_empty(cli.project.as_deref()).or_else(|| non_empty(file.gitlab.project.as_deref()));
        Self {
            url,
            token,
            insecure,
            plain_http: http,
            project,
        }
    }

    pub fn transport(&self) -> Transport {
        if self.plain_http {
            Transport::PlainHttp
        } else if self.insecure {
            Transport::HttpsSkipVerify
        } else {
            Transport::Https
        }
    }

    pub fn from_env(cli: &CliOptions, file: &FileConfig) -> Self {
        let url = std::env::var("GITLAB_URL").ok();
        let token = std::env::var("GITLAB_TOKEN").ok();
        Self::resolve(cli, url.as_deref(), token.as_deref(), file)
    }
}

pub fn save_gitlab_section(path: &Path, url: &str, token: &str) -> Result<(), String> {
    let mut root = match std::fs::read_to_string(path) {
        Ok(content) => content.parse::<toml::Table>().map_err(|e| {
            format!(
                "{}: {}; not overwriting",
                path.display(),
                describe_toml_error(&content, &e)
            )
        })?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => toml::Table::new(),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    let entry = root
        .entry("gitlab")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    let section = entry
        .as_table_mut()
        .ok_or_else(|| format!("{}: [gitlab] is not a table", path.display()))?;
    section.insert("url".into(), toml::Value::String(url.to_string()));
    section.insert("token".into(), toml::Value::String(token.to_string()));
    let body =
        toml::to_string_pretty(&root).map_err(|e| format!("cannot serialize config: {e}"))?;

    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    write_private(path, &body).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

fn write_private(path: &Path, body: &str) -> std::io::Result<()> {
    use std::io::Write;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config.toml".into());
    let tmp = path.with_file_name(format!(".{name}.tmp-{}", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&tmp)?;
        file.write_all(body.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}
