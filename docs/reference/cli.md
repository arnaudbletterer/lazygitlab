# CLI Reference & Options

`lazygitlab` offers command-line flags to configure sessions, select projects, override environment variables, and script behaviors.

Precedence: flags > environment > config file > defaults.

---

## Command-Line Flags

| Flag | Description |
| --- | --- |
| `-h, --help` | Print the help summary and exit |
| `-V, --version` | Print the version and exit |
| `--project <PATH\|ID>` | Open this project at startup, by numeric ID or full path (`group/project`) |
| `--url <URL>` | GitLab instance URL (default: gitlab.com) |
| `--token <TOKEN>` | Personal access token |
| `--config <PATH>` | Config file (default: `~/.config/lazygitlab/config.toml`) |
| `--insecure` | Use plain HTTP and skip TLS verification (self-signed certificates) |
| `--headless` | Render off-screen, with `--width`, `--height`, `--script` and `--frames-dir` (see CONTRIBUTING.md) |

Precedence: flags, then environment (`GITLAB_URL`, `GITLAB_TOKEN`, and a `.env` file), then the config file, then defaults.

---

## Usage Examples

### Launch directly into a specific project:
```bash
lazygitlab --project my-org/backend-service
```

### Connect to a self-hosted instance with a self-signed certificate:
```bash
GITLAB_TOKEN=... lazygitlab --url https://gitlab.internal.corp --insecure
```

### GitLab served under a path prefix:
```bash
lazygitlab --url https://example.com/gitlab
```

### Logs

Diagnostics are written to `$XDG_STATE_HOME/lazygitlab/lazygitlab.log` (default `~/.local/state/lazygitlab/`, `%LOCALAPPDATA%\lazygitlab\` on Windows) with mode `0600` on Unix. Symlinks are refused. Set `LAZYGITLAB_LOG=/path/to/file.log` to choose another path. Tokens are redacted.
