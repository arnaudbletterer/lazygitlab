# AGENTS.md

> Contributor and AI-agent guide for `lazygitlab`. Read this before opening a PR.

## Documentation Map

| Doc | Owns | Update when |
| --- | --- | --- |
| `README.md` | Onboarding: pitch, features, install, config, keybindings cheatsheet, architecture, testing, links. | A new user-facing feature is added or an existing one changes visibly. |
| `ROADMAP.md` | Current state (what's built) + prioritized "what's next". | A new feature lands: move it from "What's Next" → "Current State". A new task is identified: add to "What's Next". |
| `CHANGELOG.md` | User-visible changes per release. | Every PR that changes user-visible behavior. Bump-style releases MUST add an entry in the same PR (see Release Workflow below). |
| `LICENSE` | Project license (MIT). | License change only. |
| `RELEASING.md` | Release process and version bumping. | Release process changes only. |

### Decision rule

Ask: *would a user running the binary notice this change?*

- **Yes** → `CHANGELOG.md` (and `README.md` if it adds/changes a feature, and `ROADMAP.md` if it lands a "what's next" item).
- **No, internal only** (refactor, perf, CI) → `CHANGELOG.md` under `### Changed` or omitted if truly invisible.
- **No, doc only** → update the relevant doc only; no `CHANGELOG.md` entry needed for a typo fix.

## Documentation Maintenance Rules

1. **Keep the three feature lists in sync.** The "Features" bullets in `README.md`, the "Current State" bullets in `ROADMAP.md`, and the file pointers throughout should refer to the same things. When you add a feature:
   1. Add the `README.md` feature bullet (one line, file:line pointer).
   2. Add the `ROADMAP.md` "Current State" entry under the appropriate area.
   3. Remove the corresponding `ROADMAP.md` "What's Next" item.
   4. Add a `CHANGELOG.md` entry under `[Unreleased]`.

2. **File pointers must be real.** Every `file.rs:LINE` reference in `README.md` and `ROADMAP.md` should resolve to a real symbol. If you move code, grep the docs and update them in the same PR.

3. **`CHANGELOG.md` is hand-curated.** `scripts/bump_version.py` does **not** update it. The release PR must include the `CHANGELOG.md` entry. See Release Workflow below.

4. **`ROADMAP.md` is not a forward-looking Gantt chart.** Keep "What's Next" as a small (≤ 15 items), prioritized, effort-estimated list. When a bucket fills up, retire stale items into a "Completed since" section or delete them.

## Release Workflow

Follow [`RELEASING.md`](./RELEASING.md) in full. Highlights and additions:

### Pre-release checklist (in addition to `RELEASING.md`)
- [ ] `git status` clean
- [ ] `cargo test` passes
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `CHANGELOG.md` has an entry for the new version (move from `[Unreleased]`)
- [ ] `ROADMAP.md` "Current State" reflects what shipped in this release
- [ ] `README.md` "Features" is up to date

### Bumping
- Use `python scripts/bump_version.py [patch|minor|major]`. Never edit `Cargo.toml` version by hand.
- The bump PR must include:
  - The `Cargo.toml` / `Cargo.lock` changes (produced by the script)
  - A new `CHANGELOG.md` entry under the bumped version, dated today
  - Any doc sync required by the checklist above
- `AGENTS.md` itself is not bumped.

### Push
- `git push origin master --tags` after the script runs.

## Coding Conventions

Lifted from the existing codebase; enforced loosely.

### Style
- **No new top-level dependencies without justification.** The current set is deliberately small. If a new crate is needed, add a one-line rationale to the PR description.
- **No comments unless asked.** Code should be self-documenting via naming. Use `tracing` for runtime explanations.
- **Async via `tokio::spawn` + `mpsc`.** Long-running API calls live in `src/app/*.rs` and post results back through `DataUpdate`. See `src/main.rs:75-164` for the event loop and `src/app/mod.rs:197-603` for the dispatcher.
- **Errors: `Result<_, Box<dyn std::error::Error>>` for fallible helpers, `tracing` for background tasks, `DataUpdate::Error` for user-facing failures.**
- **API endpoints are hand-written in `src/api.rs`.** ~65 types implementing `gitlab::api::Endpoint`. New endpoint = new struct there + a fetcher in the relevant `src/app/*.rs` file.

### UI
- **Test with `ratatui::backend::TestBackend`.** See `tests/integration_ui.rs` and `tests/ui_exhaustive.rs` for patterns.
- **Popups live in `src/ui/popup.rs` and are gated by `App` boolean flags** (`show_*_popup`). Add the flag to `App::new` (`src/models.rs`) and to the popup switch in `src/ui/mod.rs:93-108`.
- **No `unwrap` in user-facing paths.** Background tasks may `unwrap` on the inner `send` or semaphore — those are the only acceptable cases.

### State
- **`App` is a single struct in `src/models.rs`.** All new UI state goes there with `#[serde(skip)]` if it must round-trip through serialization. Consider whether the state really belongs to `App` before adding another field — the struct is already ~3,700 lines and is a known refactor target (see `ROADMAP.md` P3).

### Testing
- Integration tests in `tests/` use `wiremock` for API mocking and `ratatui::TestBackend` for UI assertions. Prefer adding a test file over mutating an existing one when introducing a new domain.
- New public API = new `src/api.rs` type = new test in `tests/`.
- New keybinding = entry in the `HelpItem` list in `src/app/mod.rs:1607-1646` and ideally a test in `tests/shortcuts.rs`.

## Project Structure (Quick Ref)

```
src/
  main.rs         # event loop, terminal setup
  app/            # stateful App logic
    mod.rs        # App impl: state, keymap, dispatcher
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
  api.rs          # GitLab REST endpoint types
  models.rs       # DTOs + the App struct definition
  logger.rs       # tracing setup
tests/            # wiremock + TestBackend integration tests
scripts/          # bump_version.py, populate_stress_test.py
```

## Common Tasks

- **Add a new MR action** (e.g. "lock discussion"): add API struct in `src/api.rs` → add `pub async fn` in `src/app/mrs.rs` → wire key in `src/app/mod.rs::handle_key` → add `HelpItem` → add test in `tests/mr_actions.rs`.
- **Add a new dashboard tab**: extend `MenuSelection` (`src/models.rs`) → add a `DataUpdate::*` variant → add a fetcher in `src/app/<area>.rs` → extend `process_update_single` → add a render branch in `src/ui/list.rs` → add a `g <letter>` jump shortcut.
- **Add a new popup**: add a `show_*_popup: bool` field on `App` → handler in the relevant `src/app/*.rs` file → renderer in `src/ui/popup.rs` → branch in `src/ui/mod.rs:93-108` → set/clear the flag from the keymap.

## Things To Avoid

- Do not add `unwrap()` to user-facing code paths.
- Do not introduce a new dependency without a one-line justification.
- Do not edit `Cargo.toml` version by hand — use the script.
- Do not commit secrets. `.env` is in `.gitignore`.
- Do not force-push to `master`.
- Do not open a "what's next" item without first checking it is not already done.
