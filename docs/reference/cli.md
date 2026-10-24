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

--- | --- |
| `-h, --help` | Display the help summary and exit |
| `-V, --version` | Display version number (`lazygitlab 0.16.0`) and exit |
| `--project <PATH\|ID>` | Open a project by ID (`81221057`) or full path (`group/project`) on launch |
| `--url <URL>` | GitLab instance URL (e.g. `https://gitlab.example.com`). A path prefix is supported (`https://host/gitlab`) |
| `--token <TOKEN>` | Personal access token. Use `--token -` to read it from the first line of stdin |
| `--config <FILE>` | Path to a custom configuration file |
| `--insecure` | Skip TLS certificate verification (last resort; prefer adding your CA to the system trust store). Warns at every start. An `http://` URL uses plain HTTP |
| `--headless` | Render off-screen, with `--width`, `--height`, `--script` and `--frames-dir` |

!!! warning "Keep your token out of the process list"
    A value passed as `--token <TOKEN>` is visible to every user on the machine in `ps` output and
    may be saved in your shell history. Prefer the `GITLAB_TOKEN` environment variable, the
    `config.toml` file (created with mode `0600` by the setup wizard), or read it from stdin:

    ```bash
    pass show gitlab/token | lazygitlab --token -
    ```

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
