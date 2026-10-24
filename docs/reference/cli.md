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
| `--token <TOKEN>` | Personal access token. Visible in the process list; prefer `--token -`, which reads the token from the first line of stdin, or the `GITLAB_TOKEN` variable |
| `--config <PATH>` | Config file (default: `~/.config/lazygitlab/config.toml`) |
| `--insecure` | Skip TLS certificate verification over `https://`. Prints a warning at every start and shows `TLS NOT VERIFIED` in the header. For plain HTTP, use an `http://` URL instead. Prefer adding your CA to the system trust store |
| `--headless` | Render off-screen, with `--width`, `--height`, `--script` and `--frames-dir` (see CONTRIBUTING.md) |

Environment variables are `GITLAB_URL` and `GITLAB_TOKEN`. `.env` files are not read.

---

## Usage Examples

### Launch directly into a specific project:
```bash
lazygitlab --project my-org/backend-service
```

### Self-hosted instance with a self-signed certificate:
```bash
lazygitlab --url https://gitlab.internal.corp --insecure
```

### Plain HTTP instance:
```bash
lazygitlab --url http://gitlab.internal.corp
```

### Pass the token on stdin:
```bash
pass show gitlab/token | lazygitlab --token -
```

### GitLab served under a path prefix:
```bash
lazygitlab --url https://example.com/gitlab
```

### Logs

Diagnostics are written to `$XDG_STATE_HOME/lazygitlab/lazygitlab.log` (default `~/.local/state/lazygitlab/`, `%LOCALAPPDATA%\lazygitlab\` on Windows) with mode `0600` on Unix. Symlinks are refused. Set `LAZYGITLAB_LOG=/path/to/file.log` to choose another path. Tokens are redacted.
