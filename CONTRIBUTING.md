# Contributing

See [AGENTS.md](./AGENTS.md) for the contributor guide, [ROADMAP.md](./ROADMAP.md) for current state and what is next, and [RELEASING.md](./RELEASING.md) for the release process.

## Architecture

```
src/
  main.rs         # binary entry, terminal setup
  runner.rs       # tokio event loop (run_app), adaptive refresh cadence
  app/            # stateful App logic (one module per domain)
    mod.rs        # App struct impl: dispatcher, refresh, shared helpers
    keymap.rs     # handle_key + help/action catalog
    menus.rs      # menu navigation & drill-in flows
    popups.rs     # popup keyboard handling
    mrs/          # merge requests: mod.rs (listing/discussions),
                  # diffs.rs, meta.rs, review.rs
    issues.rs     # Issues, discussions
    pipelines.rs  # CI/CD
    global.rs     # Projects, groups, todos, repo tree/branches
    boards.rs     # Kanban, epics
    search.rs     # Global search hybrid
  ui/             # ratatui renderers
    mod.rs        # top-level frame, popups, job log, file viewer
    list.rs       # list/detail/metadata rendering
    detail.rs     # tabbed detail panel
    diff.rs       # diff view
    boards.rs     # Kanban
    popup/        # one file per popup type (+ tree.rs branch picker)
    markdown.rs   # pulldown-cmark + syntect renderer
    widgets.rs    # Theme, status styles, helpers
  api.rs          # ~65 hand-rolled GitLab REST endpoint types
  models.rs       # DTOs + the App struct definition
  logger.rs       # tracing setup
```

The `App` is a single large struct that owns all UI state. Background API work runs in `tokio::spawn`ed tasks and pushes results back through a `tokio::sync::mpsc::UnboundedSender<DataUpdate>` channel. The event loop (`src/runner.rs`) drains the channel each frame and triggers a redraw.

## Testing

The test suite is split into a hermetic tier (default `cargo test`) and a live tier that talks to a real GitLab instance. All live tests are gated behind `#[ignore]` and resolved via `data/test_ground.json` (a single canonical test ground project).

```bash
# Hermetic: wiremock + unit, no network.
cargo test

# One-time seed of the canonical test ground (idempotent, requires GITLAB_TOKEN).
python scripts/populate_test_ground.py
# Add `--with-massive-mr` to also seed a 120-file MR for `tests/test_massive_mr_diffs.rs`.

# Live smoke: quick end-to-end checks (issue/MR updates, reply to a thread).
# Requires GITLAB_TOKEN and data/test_ground.json.
cargo test -- --ignored live_smoke

# Live full: exhaustive sweep (massive-MR diffs, all entities, all features).
cargo test -- --ignored live_full
```

A `Makefile` (and a `scripts/test.ps1` mirror for Windows) wraps these:

```bash
make test              # wiremock + unit
make seed              # populate test ground
make seed-massive      # populate + 120-file MR
make test-live-smoke   # live smoke
make test-live-full    # live full
make lint              # cargo clippy --all-targets -- -D warnings
make check             # fmt + lint + test (pre-merge gate)
make coverage          # cargo tarpaulin (Linux only)
```

`tests/common/mod.rs` reads `data/test_ground.json` and exposes a single `test_ground()` accessor; every live test must use it (no hardcoded project ids or slugs).

### Headless mode

For snapshot testing and headless CI, the binary supports a `--headless` flag that drives the TUI from a text event script and writes each rendered frame to a directory:

```bash
lazygitlab --headless --width 120 --height 40 --script events.txt --frames-dir out/
```

`events.txt` is a newline-delimited script:
- `q`, `j`, `k`, `<Enter>`, `<Esc>`, `<F1>`, `up`, `down`, ... (named keys)
- `<S-q>`, `<C-r>`, `<A-x>`, ... (modifiers: Shift, Ctrl, Alt)
- `mouse:down:10:5`, `mouse:scroll_down:0:0` (mouse events)
- `#` starts a comment; blank lines are ignored
- `quit` is a shortcut for `q`

Snapshot tests in `tests/headless_snapshots.rs` use [`insta`](https://insta.rs/) to diff the rendered TUI buffer against `tests/snapshots/*.snap`. To accept a new snapshot, run with `INSTA_UPDATE=always cargo test --test headless_snapshots` (or `cargo insta review`).

The test suite has ~80 integration tests across `tests/` covering:

- API mocking with `wiremock` (e.g. `tests/global_search.rs`, `tests/phase3_boards.rs`)
- UI rendering with `ratatui::backend::TestBackend` (e.g. `tests/integration_ui.rs`, `tests/ui_exhaustive.rs`)
- End-to-end key handling (`tests/shortcuts.rs`, `tests/navigation_cycling.rs`)
- User stories (`tests/user_stories.rs`)
- Stress / massive-MR (`tests/test_massive_mr_diffs.rs`, `tests/stress_test_exhaustive_e2e.rs`)
- Membership / metadata caching (`tests/test_project_selection_and_lazy_metadata.rs`)
- API equivalence oracle (`tests/api_equivalence.rs`, `tests/data_update_round_trip.rs`)
- Headless script parser (`tests/headless_script.rs`)
- Headless snapshots via `insta` (`tests/headless_snapshots.rs`)

CI is defined in `.gitlab-ci.yml`: `fmt` + `clippy` + `test:wiremock` on every push, `test:live-smoke` on merge requests and the default branch, `test:live-full` on a nightly schedule and manual trigger, and `coverage:tarpaulin` on a schedule (Linux only).


## Demo screenshots

The screenshots and animation in the README are generated against a fictional GitLab, so no real instance, name or project appears in them. CI generates them on the docs site. To build them locally, see [tools/demo/README.md](tools/demo/README.md).
