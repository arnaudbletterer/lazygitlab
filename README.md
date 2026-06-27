# lazygitlab

A fast, keyboard-driven terminal UI for [GitLab](https://about.gitlab.com/). Triage merge requests, review diffs, watch pipelines, and manage issues without leaving the terminal.

`lazygitlab` talks to any GitLab instance via the REST API and renders everything with [ratatui](https://ratatui.rs). It is designed to be a daily-driver replacement for the web UI's most common workflows.

## Features

### Dashboard & Navigation
- **Personal dashboard** with tabs for Todos, Merge Requests, Issues, Projects, Groups (`src/app/mod.rs:1`)
- **Membership-scoped listing** so you only see projects you belong to (`src/app/mod.rs:266`, fixed in 0.14.4)
- **Jump shortcuts** (`g t` / `g m` / `g i` / `g p`) and contextual sidebar/list/detail focus (`1`/`2`/`3` / `Tab`)
- **Global search** with a dual-strategy (Elasticsearch + scoped) hybrid (`src/app/search.rs:6`, shortcut `f`)
- **Filter-as-you-type** per list (`/`)

### Merge Request review
- **Diff view with inline comments** scoped to a specific line or line range (`src/app/mrs.rs:316`, shortcut `c` in diff)
- **Approve / Unapprove / Merge / Rebase / Squash / Draft toggle / Open-Close** (`A` / `R` / `M` / `E` / `S` / `D` / `c` in detail) — see `src/app/mrs.rs:439-870`
- **Edit metadata in place**: title, description, labels, assignees, reviewers, milestone, target branch
- **Local diff fallback** for GitLab-collapsed diffs and throttled concurrent fetching (`src/app/mrs.rs:249`)

### Issues
- **Create / edit / state toggle** (`src/app/issues.rs:449`)
- **Label, assignee, milestone, due/start date, weight** editing
- **Confidentiality toggle** (`src/app/issues.rs:417`)

### CI/CD
- **Pipelines list** with retry/cancel (`src/app/pipelines.rs`)
- **Job log viewer** with ANSI rendering, section collapsing, follow-mode, multi-line copy (`src/ui/mod.rs:150`)
- **Run new pipeline** with ref + custom variables (`src/app/pipelines.rs:152`)

### Project view
- **Repository tree** with file content viewer (`src/app/global.rs:168`, `src/ui/mod.rs:315`)
- **Commits history** with pagination (`src/app/global.rs:210`)
- **Branches** list (`src/app/global.rs:250`)
- **Kanban boards** with horizontal scroll, multi-column filter, drag issues between columns with `H` / `L` (`src/app/mod.rs:3456`, `src/ui/boards.rs`)
- **Epics** list per group (`src/app/boards.rs:107`)

### Quality of life
- **Adaptive refresh**: 2s live / 5s active / 30s idle back-off (`src/main.rs:126`)
- **Clipboard**: local `arboard` + OSC 52 sequence for remote/tmux sessions (`src/app/mod.rs:3527`)
- **Mouse support** throughout, including scroll-aware list navigation
- **Themable** via `theme.toml` (see `theme.toml.example`)
- **Persistent logging** to `lazygitlab.log` (`src/logger.rs`)
- **Command palette via `?`** — searchable help popup with category grouping

## Installation

```bash
# Requires Rust 1.85+ (edition 2024)
cargo install --path .

# Or just build a release binary
cargo build --release
./target/release/lazygitlab
```

## Configuration

Copy and edit the example env file:

```bash
cp .env.example .env
# Then set:
#   GITLAB_URL=https://gitlab.com        (or your self-hosted instance)
#   GITLAB_TOKEN=<your personal access token>
```

The token needs the `api` and `read_user` scopes at minimum.

Optional theming: copy `theme.toml.example` to `theme.toml` next to the binary and edit colors. See the example file for available keys.

Logs are written to `lazygitlab.log` in the working directory (see `.gitignore`).

## Quickstart

```bash
export GITLAB_URL=https://gitlab.com
export GITLAB_TOKEN=glpat-...
cargo run --release
```

Press `?` in the app for the full keybinding list. The most common starting points:

| Key | Action |
| --- | --- |
| `?` | Help / command palette |
| `g t` / `g m` / `g i` / `g p` | Jump to Todos / MRs / Issues / Pipelines |
| `/` | Filter current list |
| `f` | Global search |
| `Enter` | Open detail / select |
| `Esc` | Back / close popup |
| `y` | Copy URL or content to clipboard |
| `o` | Open current item in browser |
| `q` | Quit |

## Architecture

```
src/
  main.rs         # tokio event loop, terminal setup
  app/            # stateful App logic (one file per domain)
    mod.rs        # App struct, state transitions, keymap (~3700 LOC)
    mrs.rs        # Merge requests, diffs, discussions
    issues.rs     # Issues, discussions
    pipelines.rs  # CI/CD
    global.rs     # Projects, groups, todos, repo
    boards.rs     # Kanban, epics
    search.rs     # Global search hybrid
  ui/             # ratatui renderers
    mod.rs        # top-level frame, popups, job log, file viewer
    list.rs       # list/detail/diff rendering
    detail.rs     # MR/issue metadata panel
    boards.rs     # Kanban
    popup.rs      # All modal popups
    markdown.rs   # pulldown-cmark wrapper
    widgets.rs    # Theme, status styles, helpers
  api.rs          # ~65 hand-rolled GitLab REST endpoint types
  models.rs       # DTOs + the App struct definition
  logger.rs       # tracing setup
```

The `App` is a single large struct that owns all UI state. Background API work runs in `tokio::spawn`ed tasks and pushes results back through a `tokio::sync::mpsc::UnboundedSender<DataUpdate>` channel. The main loop drains the channel each frame and triggers a redraw. See `src/main.rs:75-164` for the event loop and `src/app/mod.rs:197-603` for the update dispatcher.

## Testing

```bash
cargo test
```

The test suite has ~50 integration tests across `tests/` covering:

- API mocking with `wiremock` (e.g. `tests/e2e_remote.rs`, `tests/global_search.rs`)
- UI rendering with `ratatui::backend::TestBackend` (e.g. `tests/integration_ui.rs`, `tests/ui_exhaustive.rs`)
- End-to-end key handling (`tests/shortcuts.rs`, `tests/navigation_cycling.rs`)
- Stress tests for massive MRs (`tests/test_massive_mr_diffs.rs`, `tests/stress_test_exhaustive_e2e.rs`)
- Membership / metadata caching (`tests/test_project_selection_and_lazy_metadata.rs`)

See [AGENTS.md](./AGENTS.md) for the agent / contributor guide, [ROADMAP.md](./ROADMAP.md) for current state and what's next, and [RELEASING.md](./RELEASING.md) for the release process.

## Acknowledgements

Built on [ratatui](https://ratatui.rs), [crossterm](https://github.com/crossterm-rs/crossterm), [tokio](https://tokio.rs), the [gitlab](https://crates.io/crates/gitlab) crate, [tui-tree-widget](https://crates.io/crates/tui-tree-widget), [ratatui-textarea](https://crates.io/crates/ratatui-textarea), [syntect](https://github.com/trishume/syntect), and [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark).

## License

MIT — see [LICENSE](./LICENSE).
