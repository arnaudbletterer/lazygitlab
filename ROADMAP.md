# lazygitlab Roadmap

This document tracks the current state of `lazygitlab` and what is left to do. The original four-phase plan that previously lived here is **complete** and has been merged into the **Current State** section below; new work is tracked in **What's Next**.

## Current State (v0.15.0)

`lazygitlab` is a mature, daily-driver TUI for GitLab. The original phased plan (multi-line editor, inline code review, repository tree, Kanban boards, global search, real-time refresh) is fully implemented. Recent releases focused on stabilization (3-column tabbed detail view, membership-scoped listings, lazy metadata/diff loading, optimistic updates, massive-MR handling) and on closing out the P0–P3 backlog below.

### What's Built

Grouped by area. File pointers are the canonical location for the implementation; refer to `git log --oneline` for the introduction commit.

#### Dashboard & Navigation
- **Personal dashboard tabs**: Todos, MRs, Issues, Projects, Groups (`src/app/mod.rs`)
- **Membership-scoped project listing** (`src/app/global.rs:266`, `src/app/mod.rs:204`) — 0.14.4
- **Jump shortcuts** `g t` / `g m` / `g i` / `g p` / `g r` (`src/app/keymap.rs:806`)
- **3-column tabbed layout** (sidebar / list / detail) with focus toggling — 0.14.0 (`src/ui/mod.rs`, `src/ui/list.rs`)
- **Global filter** per list (`src/app/mod.rs::filter_lists`, shortcut `/`)
- **Group multi-project elements** in dashboard views — 0.10.9
- **Involvement-only toggle** (`I`) — 0.10.7

#### Search
- **Hybrid global search**: Elasticsearch-backed `GlobalSearch` + scoped `PagedProjects` / `GlobalIssues` / `GlobalMergeRequests` with dedup (`src/app/search.rs:6`, shortcut `f`) — 0.8.0

#### Merge Requests
- **Inline code review on diffs**: line/range-scoped comments with `base/start/head_sha` (`src/app/mrs/review.rs`, shortcut `c`) — 0.10.18
- **Approve / Unapprove** (`A` / `R`) (`src/app/mrs/review.rs`) 
- **Merge / Rebase** with confirmation popups and pre-flight status (`M` / `E`) — 0.12.0
- **Squash toggle** (`S`) — 0.11.0
- **Draft toggle** with `draft → wip → title-prefix` fallback chain (`D`) — 0.14.1
- **Open / Close** (`c`)
- **Edit metadata**: title, description, labels, assignees, reviewers, milestone, target branch (`src/app/mrs/meta.rs`)
- **Discussion resolution** (`src/app/mrs/review.rs`)
- **Related MRs + closing issues** in the metadata panel (`related_merge_requests` / `closes_issues`, lazy-fetched) — Unreleased
- **Throttled concurrent diff fetching** + **local diff fallback** for GitLab-collapsed diffs — 0.10.14 / 0.10.15 (`src/app/mrs/diffs.rs`)
- **Reaction / participant metadata** (`src/app/mrs/mod.rs`)

#### Issues
- **Create / edit / state toggle** (`src/app/issues.rs:449`, `src/app/issues.rs:392`)
- **Edit labels / assignees / milestone / due & start date / weight / confidential** (`src/app/issues.rs:221-446`)
- **Issue discussions** with notes and resolution (`s` resolves via `resolve_issue_discussion`) — Unreleased
- **Linked issues + closing MRs** in the metadata panel (`links` / `closed_by`, lazy-fetched) — Unreleased

#### CI/CD
- **Pipelines list** with pagination, retry, cancel, status tracking (`src/app/pipelines.rs`)
- **Job log viewer**: ANSI rendering, section collapsing, follow-mode, multi-line copy (`src/ui/mod.rs:150`)
- **Launch pipeline** with ref + custom variables (`src/app/pipelines.rs:152`)
- **Job actions**: play, retry, cancel (`src/app/pipelines.rs:215-269`)

#### Project view
- **Repository tree** (`src/app/global.rs:168`, `src/ui/mod.rs:315`) with **branch browsing** (`b` opens a branch picker; tree + file fetches scoped to the selected ref, title shows it) — Unreleased
- **File viewer** with syntect highlighting and independent scroll state — 0.15.0
- **Create / delete branch UI** on the Branches list (`n` popup, `D` delete) — 0.15.0
- **Commits history** with pagination (`src/app/global.rs:210`)
- **Branches** list (`src/app/global.rs:250`)
- **Project variables** (`src/app/pipelines.rs:82`)
- **Kanban boards**: horizontal scroll, multi-column filter, drag issues between columns with `H` / `L` (`src/app/mod.rs`, `src/ui/boards.rs`) — Phase 3
- **Epics** per group (`src/app/boards.rs:107`, `src/ui/boards.rs:173`)
- **Project metadata cache** (labels, milestones, members) with deduplication and lazy fetch (`src/app/mod.rs:826`)

#### Quality of life
- **Adaptive refresh**: 2s live / 5s active / 30s idle back-off (`src/runner.rs`)
- **Clipboard**: local `arboard` + OSC 52 sequence for remote/tmux (`src/app/mod.rs:3527`) — 0.9.0
- **Mouse support** with scroll-aware list navigation — 0.10.2
- **Themable** via `theme.toml`
- **Persistent tracing logs** to `lazygitlab.log` (`src/logger.rs`)
- **Searchable help popup** (`src/app/keymap.rs::get_available_actions`)
- **Optimistic metadata updates** (update local state, then API call) — 0.14.3
- **Optimistic insert** of newly created Issues / MRs into local lists — 0.15.0
- **GFM markdown**: tables (box-drawn, aligned), task-list checkboxes, blockquotes, footnotes (`src/ui/markdown.rs`) — Unreleased / 0.15.0
- **Self-expiring footer feedback**: transient status messages clear after 6 s, errors after 15 s (`App::tick_status`) — Unreleased

## What's Next

Prioritized. Effort is rough and assumes a single contributor. **Do not start work without a corresponding `AGENTS.md` update.**

### P0 — Test autonomy (Iterations 1–4 of 4)
- [x] Consolidate to a single canonical test ground project (`abletterer/lazygitlab-test`, id resolved via `data/test_ground.json`)
- [x] Idempotent `scripts/populate_test_ground.py` (GET-first, skip-if-exists) replacing `scripts/populate_stress_test.py`
- [x] `tests/common/mod.rs` reading `data/test_ground.json`; every live test uses it
- [x] `#[ignore = "live_smoke" | "live_full"]` gating on every live test
- [x] `Makefile` + `scripts/test.ps1` mirroring it
- [x] `.gitlab-ci.yml` with `fmt` + `clippy` + `test:wiremock` on every push, `test:live-smoke` on MRs, `test:live-full` on nightly + manual, `coverage:tarpaulin` on schedule
- [x] API-equivalence oracle (`tests/api_equivalence.rs`, `tests/data_update_round_trip.rs`) — 15 live tests, 11 entity families
- [x] `--headless` mode in the binary (TestBackend-driven event loop with synthesized events, optional `--frames-dir` capture)
- [x] Snapshot tests with `insta` for the rendered buffer (`tests/headless_snapshots.rs`)
- [x] Three-tier CI: wiremock (default) / live-smoke on every push + MRs (`test:live-smoke`, `test:live-smoke:push`) / live-full (nightly) — Unreleased

### P1 — Roadmap residue
Original-plan items that were deferred. Each is a self-contained feature.
- [x] Add `LICENSE` (MIT)
- [x] Add `README.md` (onboarding)
- [x] Add `CHANGELOG.md` (Keep-a-Changelog format, hand-curated)
- [x] Add `AGENTS.md` (maintenance instructions)
- [x] Rewrite `ROADMAP.md` (this file)
- [x] **File viewer polish** — syntax highlighting via `syntect`, independent scroll state, branch selection (`b`) — 0.15.0 / Unreleased
- [x] **Related Issues / Linked MRs** in MR / Issue detail metadata — Unreleased
- [x] **Create / delete branch UI** (`CreateBranch` + new `DeleteBranch`, popup + handlers) — 0.15.0
- [x] **Resolve issue discussion** binding (`resolve_issue_discussion` in `src/app/issues.rs`) — 0.15.0

### P2 — UX polish
- [x] **GFM markdown**: tables, checklists, blockquotes in `src/ui/markdown.rs` — 0.15.0 / Unreleased
- [x] **Optimistic insert** for newly created Issue / MR into local lists — 0.15.0
- [x] **Unused API endpoints**: dead `CreateGroup` / `CreateMilestone` / `CreateLabel` / `CreateFile` / `CreateProject` removed — 0.15.0

### P3 — Maintainability
Pure refactor. Gated by the existing test suite; no behavior change intended.

- [x] **Split `src/app/mod.rs`** (3,720 LOC) into focused modules: `app/keymap.rs`, `app/menus.rs`, `app/popups.rs`
- [x] **Split `src/app/mrs.rs`** into `mrs/mod.rs` (listing/discussions), `mrs/diffs.rs`, `mrs/meta.rs`, `mrs/review.rs` — Unreleased
- [x] **Split `src/ui/popup.rs`** (803 LOC) — one file per popup type

### P4 — Real-time
- [x] Investigated GitLab streaming endpoints / webhooks for live updates. **Findings:** there is no public websocket/streaming REST API for entity changes; webhooks require an inbound HTTP server a terminal app cannot host. GitLab's sanctioned mechanism for pollers is *conditional GET*: endpoints registered in its ETag-caching router honor `If-None-Match` (→ `304 Not Modified`) and return a server-hinted `Poll-Interval` header (see docs.gitlab.com/development/polling). **Decision:** keep the current adaptive back-off (2s live / 5s active / 30s idle); a future optimization is ETag-based conditional GET for the hottest endpoints (todos, MR discussions, pipeline status) to cut payload work when nothing changed. Revisit only if the polling load ever becomes a problem.

## What's Next

Rebuilt from the five-persona expert review round (DevOps / MR-reviewer / PM / UX designer / Rust performance). Prioritized by user impact ÷ effort. Quick wins already implemented from that round are in the CHANGELOG.

### P0 — Reviewer & triage gaps
- [ ] **Render-path caching** — cache syntect-highlighted file lines, job-log `Vec<Line>`, and markdown output on `App`, invalidated on data change; render path becomes O(viewport) instead of O(document) per frame (`src/ui/mod.rs:237-387`, `src/ui/markdown.rs` callers). Biggest perf win.
- [ ] **"Assigned to me / awaiting my review" MR filter** — `scope=assigned-to-me` + `reviewer_id` on the existing global MR endpoint (`src/api.rs:512`, `src/models.rs:923`); today `I` mixes in every participant.
- [ ] **Failed-pipeline filter** — pass `status=` to `ProjectPipelines` (`src/api.rs:380`) and add a `[`/`]` segment for Pipelines.
- [ ] **Job-log search + G/gg/PageUp/PageDown** — reuse the filter input to jump `job_log_cursor` to the next match (`src/app/keymap.rs:234-298`).

### P1 — Workflow depth
- [ ] **Cross-project pipeline overview** on the dashboard (failed-first sort) — DevOps' primary daily view is missing entirely (`src/app/pipelines.rs:7`).
- [ ] **MR → head_pipeline jump** keybinding from detail (`src/ui/list.rs:515`).
- [ ] **Next/prev thread across files** in the diff (`n`/`p` currently stop at file boundary, `src/app/keymap.rs:570-593`).
- [ ] **Milestone grouping + counts for issues** client-side via existing grouping machinery (`src/app/mod.rs::filter_lists`), plus milestone/due-date columns on issue rows.
- [ ] **Epic drill-in** — Enter on an epic is a no-op (`src/app/mod.rs:2117`); fetch child issues and show progress.
- [ ] **Dirty-flag rendering** — stop unconditional 10 Hz redraws in `src/runner.rs`; draw only on input/update/spinner tick.
- [ ] **Debounce detail fetches** on fast list navigation (~200 ms) to avoid stale-task pileups and rate-limit burn.

### P2 — Depth & polish
- [ ] Pipeline schedules list + "run now"; runners visibility; CI variable editing (`src/app/pipelines.rs` — all absent today).
- [ ] Real approvals display (`/approvals` endpoint) next to reactions in the diff header; show approval state in merge confirmation.
- [ ] Confirm popup for destructive branch delete (`D`, `src/app/keymap.rs:870-876`), matching merge/rebase safety.
- [ ] Truncation/ellipsis helper for long titles and metadata lines on narrow terminals (`src/ui/list.rs:205`).
- [ ] ETag conditional GET (`If-None-Match` → 304) for todos/discussions/pipeline-status polling (P4 follow-up).

## Process

- Before merging anything that changes user-visible behavior: update `CHANGELOG.md`, and if it adds a new feature, also update `README.md` (Features) and move the item from **What's Next** to **Current State** in this file.
- See [AGENTS.md](./AGENTS.md) for the full contributor / agent guide.
- See [RELEASING.md](./RELEASING.md) for the release workflow.
