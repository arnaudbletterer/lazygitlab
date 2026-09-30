# CLI Reference & Options

`lazygitlab` offers command-line flags to configure sessions, select projects, override environment variables, and script behaviors.

---

## Command-Line Flags

| Flag | Description |
| --- | --- |
| `-h, --help` | Display the help summary and exit |
| `-v, -V, --version` | Display version number (`lazygitlab 0.16.0`) and exit |
| `-p, --project <PROJECT>` | Automatically select a project by ID (`81221057`) or full path (`group/project`) on launch |
| `--url <URL>` | Override GitLab API URL (e.g. `https://gitlab.example.com`) |
| `--token <TOKEN>` | Provide or override GitLab Personal Access Token / CI Token |
| `-c, --config <FILE>` | Path to custom configuration file |
| `-k, --insecure` | Allow insecure SSL connections (self-signed certificates) |

---

## Usage Examples

### Launch directly into a specific project:
```bash
lazygitlab --project my-org/backend-service
```

### Connect to a self-hosted instance with self-signed certificate:
```bash
lazygitlab --url https://gitlab.internal.corp --token $CORP_GITLAB_TOKEN --insecure
```

### Auto-Detection in Git Repositories:
When run inside any local git clone:
```bash
cd ~/Projects/my-service
lazygitlab
```
`lazygitlab` reads the `origin` remote URL from `.git/config`, matches it against your GitLab projects, and opens the project automatically!
