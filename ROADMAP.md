# lazygitlab Roadmap

This document tracks the current state of `lazygitlab` and what is left to do. The original four-phase plan that previously lived here is **complete** and has been merged into the **Current State** section below; new work is tracked in **What's Next**.

## Current State (v0.14.7)

`lazygitlab` is a mature, daily-driver TUI for GitLab. The original phased plan (multi-line editor, inline code review, repository tree, Kanban boards, global search, real-time refresh) is fully implemented. Recent releases (0.13.x–0.14.x) focused on stabilization: 3-column tabbed detail view, membership-scoped listings, lazy metadata/diff loading, optimistic updates, and massive-MR handling.

### What's Built

Grouped by area. File pointers are the canonical location for the implementation; refer to `git log --oneline` for the introduction commit.

#### Dashboard & Navigation
- **Personal dashboard tabs**: Todos, MRs, Issues, Projects, Groups (`src/app/mod.rs`)
- **Membership-scoped project listing** (`src/app/global.rs:266`, `src/app/mod.rs:204`) — 0.14.4
- **Jump shortcuts** `g t` / `g m` / `g i` / `g p` / `g r` (`src/app/mod.rs:1678-1729`)
- **3-column tabbed layout** (sidebar / list / detail) with focus toggling — 0.14.0 (`src/ui/mod.rs`, `src/ui/list.rs`)
- **Global filter** per list (`src/app/mod.rs:873`, shortcut `/`)
- **Group multi-project elements** in dashboard views — 0.10.9
- **Involvement-only toggle** (`I`) — 0.10.7

#### Search
- **Hybrid global search**: Elasticsearch-backed `GlobalSearch` + scoped `PagedProjects` / `GlobalIssues` / `GlobalMergeRequests` with dedup (`src/app/search.rs:6`, shortcut `f`) — 0.8.0

#### Merge Requests
- **Inline code review on diffs**: line/range-scoped comments with `base/start/head_sha` (`src/app/mrs.rs:316`, shortcut `c`) — 0.10.18
- **Approve / Unapprove** (`A` / `R`) (`src/app/mrs.rs:439`, `src/app/mrs.rs:463`)
- **Merge / Rebase** with confirmation popups and pre-flight status (`M` / `E`) — 0.12.0
- **Squash toggle** (`S`) — 0.11.0
- **Draft toggle** with `draft → wip → title-prefix` fallback chain (`D`) — 0.14.1
- **Open / Close** (`c`)
- **Edit metadata**: title, description, labels, assignees, reviewers, milestone, target branch (`src/app/mrs.rs:614-816`)
- **Discussion resolution** (`src/app/mrs.rs:871`)
- **Throttled concurrent diff fetching** + **local diff fallback** for GitLab-collapsed diffs — 0.10.14 / 0.10.15
- **Reaction / participant metadata** (`src/app/mrs.rs:209`)

#### Issues
- **Create / edit / state toggle** (`src/app/issues.rs:449`, `src/app/issues.rs:392`)
- **Edit labels / assignees / milestone / due & start date / weight / confidential** (`src/app/issues.rs:221-446`)
- **Issue discussions** with notes and resolution (`src/app/issues.rs:471`)

#### CI/CD
- **Pipelines list** with pagination, retry, cancel, status tracking (`src/app/pipelines.rs`)
- **Job log viewer**: ANSI rendering, section collapsing, follow-mode, multi-line copy (`src/ui/mod.rs:150`)
- **Launch pipeline** with ref + custom variables (`src/app/pipelines.rs:152`)
- **Job actions**: play, retry, cancel (`src/app/pipelines.rs:215-269`)

#### Project view
- **Repository tree** (`src/app/global.rs:168`, `src/ui/mod.rs:315`)
- **Commits history** with pagination (`src/app/global.rs:210`)
- **Branches** list (`src/app/global.rs:250`)
- **Project variables** (`src/app/pipelines.rs:82`)
- **Kanban boards**: horizontal scroll, multi-column filter, drag issues between columns with `H` / `L` (`src/app/mod.rs:3456`, `src/ui/boards.rs`) — Phase 3
- **Epics** per group (`src/app/boards.rs:107`, `src/ui/boards.rs:173`)
- **Project metadata cache** (labels, milestones, members) with deduplication and lazy fetch (`src/app/mod.rs:826`)

#### Quality of life
- **Adaptive refresh**: 2s live / 5s active / 30s idle back-off (`src/main.rs:126`)
- **Clipboard**: local `arboard` + OSC 52 sequence for remote/tmux (`src/app/mod.rs:3527`) — 0.9.0
- **Mouse support** with scroll-aware list navigation — 0.10.2
- **Themable** via `theme.toml`
- **Persistent tracing logs** to `lazygitlab.log` (`src/logger.rs`)
- **Searchable help popup** (`src/app/mod.rs:1607-1646`)
- **Optimistic metadata updates** (update local state, then API call) — 0.14.3

## What's Next

Prioritized. Effort is rough and assumes a single contributor. **Do not start work without a corresponding `AGENTS.md` update.**

### P0 — Documentation (this PR)
- [x] Add `LICENSE` (MIT)
- [x] Add `README.md` (onboarding)
- [x] Add `CHANGELOG.md` (Keep-a-Changelog format, hand-curated)
- [x] Add `AGENTS.md` (maintenance instructions)
- [x] Rewrite `ROADMAP.md` (this file)

### P1 — Roadmap residue
Original-plan items that were deferred. Each is a self-contained feature.

- [ ] **File viewer polish** — `src/ui/mod.rs:315` currently renders raw base64-decoded text. `syntect` is already a dep and `App.syntax_set` / `App.theme_set` are loaded but unused here. Add syntax highlighting, branch selection, proper scroll/keyboard handling (it currently reuses `app.job_log_scroll`).
- [ ] **Related Issues / Linked MRs** in MR / Issue detail metadata. No code exists; would call `/related_issues` and parse `references` cross-links.
- [ ] **Create / delete branch UI**. `CreateBranch` API exists at `src/api.rs:853`; `target_branch_popup` only *selects* existing branches. Add a popup + handler.
- [ ] **Resolve issue discussion** binding. `resolve_discussion` in `src/app/mrs.rs:871` is MR-only; add a `resolve_issue_discussion` path in `src/app/issues.rs`.

### P2 — UX polish
- [ ] **GFM markdown**: tables, checklists, blockquotes in `src/ui/markdown.rs`. `pulldown-cmark` is loaded; needs render tweaks.
- [ ] **Optimistic insert** for newly created Issue / MR into local lists. `create_issue` and `create_mr` currently only send `TriggerRefresh`.
- [ ] **Unused API endpoints**: `CreateGroup` (`src/api.rs:787`), `CreateMilestone` (`src/api.rs:801`), `CreateLabel` (`src/api.rs:816`), `CreateFile` (`src/api.rs:833`), `CreateProject` (`src/api.rs:869`) are defined but have no UI. Either wire them or remove the dead code.

### P3 — Maintainability
Pure refactor. Gated by the existing 72 tests; no behavior change intended.

- [ ] **Split `src/app/mod.rs`** (3,720 LOC) into focused modules. Candidates: `app/keymap.rs`, `app/state_init.rs`, `app/menus.rs`, `app/popups.rs`. The `handle_key` function alone is ~960 lines.
- [ ] **Split `src/app/mrs.rs`** (1,067 LOC) — separate diff-fetching from metadata editing from review actions.
- [ ] **Split `src/ui/popup.rs`** (803 LOC) — one file per popup type.

### P4 — Real-time (optional, low ROI)
- [ ] Investigate GitLab streaming endpoints or webhooks for live updates; current adaptive polling back-off is sufficient for most use cases.

## Process

- Before merging anything that changes user-visible behavior: update `CHANGELOG.md`, and if it adds a new feature, also update `README.md` (Features) and move the item from **What's Next** to **Current State** in this file.
- See [AGENTS.md](./AGENTS.md) for the full contributor / agent guide.
- See [RELEASING.md](./RELEASING.md) for the release workflow.
