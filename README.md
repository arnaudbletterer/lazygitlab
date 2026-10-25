<p align="center">
  <img src="docs/assets/logo.svg" alt="lazygitlab logo" width="160" />
</p>

# lazygitlab

A keyboard-driven terminal UI for GitLab. Triage merge requests, review diffs, watch pipelines and manage issues without leaving your terminal.

## Why

The GitLab web UI is fine for occasional use, but daily review work means a lot of tabs, clicks and page loads. `lazygitlab` keeps your todos, merge requests, issues and pipelines in one screen, navigated with the keyboard, and refreshes in the background. It talks to the GitLab REST API, so it works with gitlab.com and self-hosted instances.

## Screenshots

![Demo](https://arnaudbletterer.github.io/lazygitlab/assets/marketing/demo.gif)

![Dashboard](https://arnaudbletterer.github.io/lazygitlab/assets/marketing/dashboard.png)
![Merge request review](https://arnaudbletterer.github.io/lazygitlab/assets/marketing/mr-review.png)
![Pipelines](https://arnaudbletterer.github.io/lazygitlab/assets/marketing/pipelines.png)

## Features

**Dashboard and navigation**
- Personal dashboard with Todos, Merge Requests, Issues, Projects and Groups
- Jump shortcuts (`g t`, `g m`, `g i`, `g b`, `g p`, `g r`) and a project switcher
- Global search (`f`) and filter-as-you-type on every list (`/`)
- Shows the current local branch and a dirty marker when run inside a git repository

**Merge requests**
- Side-by-side or unified diffs (`w`) with inline comments on a line or line range
- Approve, unapprove, merge, rebase, squash, toggle draft, open or close
- Resolve discussion threads, see related MRs and closing issues
- Edit title, description, labels, assignees, reviewers, milestone and target branch in place

**Issues and boards**
- Create, edit and close issues; edit labels, assignees, milestone, dates and weight
- Confidentiality toggle, linked issues and closing MRs
- Kanban boards with multi-column filter and card moves with `H` / `L`
- Epics per group

**CI/CD**
- Pipelines list with retry and cancel, and launching new pipelines with a ref and variables
- Job log viewer with ANSI colors, section collapsing, search, follow mode and multi-line copy
- Pipeline schedules and CI variables

**Repository**
- File tree with syntax-highlighted viewer, branch switching, commit history
- Create and delete branches

**Everything else**
- Markdown rendering (tables, task lists, code blocks), mouse support
- Clipboard through the system clipboard or OSC 52 over SSH and tmux
- Adaptive background refresh
- Nine built-in color themes, or define your own
- Tokens are redacted from logs, error messages and the status bar

## Install

| Method | Status |
| --- | --- |
| `cargo install --git https://github.com/arnaudbletterer/lazygitlab` | Available (Rust 1.85 or newer) |
| `cargo install lazygitlab` | Planned, not yet on crates.io |
| Release binaries on the GitHub Releases page | Planned |
| Homebrew: `brew install arnaudbletterer/tap/lazygitlab` | Planned |
| AUR: `lazygitlab-bin` (binary) and a source package | Planned |
| Scoop (Windows): bucket `arnaudbletterer/scoop-bucket` | Planned |
| `install.sh` script | Planned |

To build from a checkout: `cargo build --release`, then run `target/release/lazygitlab`.

## Quickstart

1. Create a personal access token in GitLab under User Settings, Access Tokens. `read_api` is enough for read-only use; grant `api` to approve, merge, comment and edit.
2. Export your credentials:

```bash
export GITLAB_TOKEN=glpat-...
export GITLAB_URL=https://gitlab.com   # optional, this is the default
lazygitlab
```

3. Press `?` for the searchable keybinding list. The terminal must be at least 50x14.

Self-hosted instance:

```bash
lazygitlab --url https://gitlab.example.com
```

`--insecure` skips TLS certificate verification, for self-signed certificates. It prints a warning at every start and the header shows `TLS NOT VERIFIED`; prefer adding your CA to the system trust store. For plain HTTP, use an `http://` URL. With no credentials configured, a first-run prompt asks for the URL and token and can save them to `~/.config/lazygitlab/config.toml`.

Logs are written to `~/.local/state/lazygitlab/lazygitlab.log` (or under `$XDG_STATE_HOME`), readable only by you. Set `LAZYGITLAB_LOG` to choose another path. To find out why a list is slow on your instance, run with `LAZYGITLAB_TRACE=1` and summarize the log with `scripts/summarize_trace.py`: it records only endpoint templates and timings, never the host, token or project names (see [docs/performance.md](docs/performance.md)).

Settings are resolved in this order: command-line flags, environment variables (`.env` files are not read), then the config file:

```toml
# ~/.config/lazygitlab/config.toml
[gitlab]
url = "https://gitlab.example.com"
token = "glpat-..."
# insecure = true
# project = "group/project"
```

Flags: `--help`, `--version`, `--project <PATH|ID>`, `--url`, `--token <TOKEN|->`, `--config <PATH>`, `--insecure`. `--token -` reads the token from stdin, since `--token <TOKEN>` is visible in the process list. See the [CLI reference](docs/reference/cli.md).

## Security

- The token is never written to logs, error messages or the status bar.
- It is sent as an `Authorization: Bearer` header, which is not forwarded on cross-host redirects.
- Skipping TLS verification needs `--insecure` or `insecure = true`, prints a warning at every start and shows `TLS NOT VERIFIED` in the header.
- No configuration is read from the working directory: no `.env` file and no local `theme.toml`.
- Escape and control sequences in job logs, Markdown and error text are stripped, so remote content cannot drive your terminal.

## Keys

| Key | Action |
| --- | --- |
| `?` | Help and command palette |
| `j` / `k` | Move down / up |
| `Tab` | Cycle panels |
| `Enter` / `Esc` | Open / go back |
| `g t` `g m` `g i` `g b` `g p` `g r` | Todos, MRs, issues, branches, pipelines, repository |
| `p` | Switch project |
| `/` | Filter the current list |
| `f` | Global search |
| `o` | Open in browser |
| `y` | Copy URL or selection |
| `r` | Refresh |
| `q` | Quit |

In a merge request: `A` approve, `R` unapprove, `M` merge, `E` rebase, `S` toggle squash, `D` toggle draft, `c` open or close. In a diff: `w` side-by-side, `c` comment on the selected line, `s` resolve a thread. Full list in the [keybindings reference](docs/reference/keybindings.md).

## Theming

Pick a preset in `theme.toml`, placed in `~/.config/lazygitlab/` (it is not read from the working directory):

```toml
theme = "tokyo-night"
accent = "#ff007f"   # optional override
```

Presets: `catppuccin-mocha` (default), `catppuccin-latte`, `tokyo-night`, `nord`, `dracula`, `gruvbox-dark`, `gruvbox-light`, `github-dark`, `github-light`. See the [theming guide](docs/reference/theming.md) and `theme.toml.example` for all color keys.

## Documentation

Guides and references live in [docs/](docs/index.md). To work on the code, see [CONTRIBUTING.md](CONTRIBUTING.md). Release history is in [CHANGELOG.md](CHANGELOG.md).

## Support

If lazygitlab saves you time, you can support its development on [GitHub Sponsors](https://github.com/sponsors/arnaudbletterer) or [Ko-fi](https://ko-fi.com/abletterer). Bug reports and feedback are just as welcome.

## Acknowledgements

Built on [ratatui](https://ratatui.rs), [crossterm](https://github.com/crossterm-rs/crossterm), [tokio](https://tokio.rs), the [gitlab](https://crates.io/crates/gitlab) crate, [tui-tree-widget](https://crates.io/crates/tui-tree-widget), [ratatui-textarea](https://crates.io/crates/ratatui-textarea), [syntect](https://github.com/trishume/syntect), and [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark).

## License

MIT, see [LICENSE](LICENSE).
