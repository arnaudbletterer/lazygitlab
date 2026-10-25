# Changelog

All notable changes to `lazygitlab` are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/).

> **Note:** `scripts/bump_version.py` does not update this file automatically. Add a new entry in the same PR that bumps the version. The most recent releases are at the top.

## [Unreleased]

### Added
- **Request timing trace:** `LAZYGITLAB_TRACE=1` writes one JSON line per API request to the log (method, endpoint template, status, duration, response bytes, page, 304). The host, token, project names and query values are never recorded. `make bench` measures list loading against a generated mock at several latencies (`docs/performance.md`). `scripts/summarize_trace.py` prints per-endpoint count, p50, p95 and total time (`src/trace.rs`).

### Fixed
- **`--insecure` with self-signed certificates:** `--insecure` (and `insecure = true`) now skips TLS certificate verification over `https://` instead of silently downgrading to plain HTTP; use an `http://` URL for plain HTTP. Scheme matching is case-insensitive and a trailing `/api/v4` is ignored (`src/config.rs`).
- **Actionable error messages:** 401, 403, 429, untrusted certificates, a wrong URL (including a missing path prefix such as `https://host/gitlab`), redirects, 5xx and network failures now say what to check, and startup connection failures exit with a clear message instead of a debug dump (`src/errors.rs`).
- **Terminal restore:** the terminal leaves raw mode and the alternate screen on panic and on early errors (`src/terminal.rs`).
- **Tiny terminals:** terminals under 50x14 show a resize message instead of crashing; the calendar popup no longer underflows (`src/ui/mod.rs`).
- **Escape-sequence injection:** job logs and Markdown from MR, issue and comment text have OSC, cursor and screen-control sequences stripped; only colors (SGR) are kept (`src/sanitize.rs`).
- **Log privacy:** `lazygitlab.log` is created with mode `0600` on Unix, and an unwritable working directory no longer aborts startup (`src/logger.rs`).
- **Panics:** a partial diff position and a syntax-highlighting failure no longer panic; test-binary detection no longer misfires on install paths containing `deps`.
- **Documentation:** `docs/reference/cli.md` listed flags that do not exist; the git auto-detection claim was removed because only the local branch and dirty state are shown.

- **`.env` no longer read:** a `.env` file in the working directory or its parents is ignored, so a cloned repository cannot point `GITLAB_URL` at another host and capture your token. Use real environment variables or `config.toml`.
- **Log location:** `lazygitlab.log` now lives in `$XDG_STATE_HOME/lazygitlab/` (default `~/.local/state/lazygitlab/`), is opened without following symlinks and only as a regular file, and can be moved with `LAZYGITLAB_LOG` (`src/logger.rs`).
- **Skip-verify visibility:** when TLS verification is disabled, from `--insecure` or `insecure = true`, a warning is printed at every start and the header shows `TLS NOT VERIFIED`; the certificate error hint now recommends the system trust store (`src/main.rs`, `src/ui/mod.rs`).
- **Text sanitizing:** Markdown is sanitized after parsing so entity-encoded escapes cannot re-form, and bidi overrides and zero-width characters are removed from everything drawn (`src/sanitize.rs`).
- **Error text:** every error is redacted and stripped of escape sequences before it reaches stderr or the status bar, and `--token -` hides input when stdin is a terminal.

- **Token header:** the token is now sent as `Authorization: Bearer`, which reqwest strips on cross-host redirects, instead of the `PRIVATE-TOKEN` header, which it forwards (`src/client.rs`).

- **Invisible characters:** zero-width joiner and non-joiner are kept, so emoji sequences and Persian or Indic text render correctly; bidi overrides, zero-width space, directional marks and line separators are still removed (`src/sanitize.rs`).
- **Error redaction order:** control characters are stripped before secrets are redacted, so a token split by escape sequences cannot slip through (`src/errors.rs`).
- **Theme location:** `theme.toml` is read only from the config directory, not the working directory (`src/ui/widgets.rs`).
- **Log hardening:** the log opens non-blocking so a FIFO cannot hang startup, the state directory must be owned by you and is tightened to `0700`, the temp fallback is per user, Windows prefers `LOCALAPPDATA`, and a refused symlink prints a clear message (`src/logger.rs`).

### Added
- **Token from stdin:** `--token -` reads the token from the first line of stdin. `--help`, the README and the CLI reference now warn that `--token <TOKEN>` is visible in the process list (`src/cli.rs`).
- **Package metadata:** description, license, keywords and an exclude list so local data files are not published (`Cargo.toml`).
- **Non-blocking UI**: The Todos background poll and opening a job log now run on spawned tasks and report through `DataUpdate`, so key input no longer freezes on slow networks; only one todos poll is in flight at a time (`src/app/global.rs`, `src/app/menus.rs`, `src/app/mod.rs`).

- **Bounded Cross-Project Pipelines Refresh**: The Pipelines tab now keeps at most 8 per-project requests in flight and skips a refresh while the previous one is still running, avoiding self-inflicted GitLab rate limiting with many projects (`src/app/pipelines.rs`).
- **Bounded Cross-Project Pipelines Refresh**: The Pipelines tab now keeps at most 8 per-project requests in flight and skips background refreshes while the previous one is still running (a manual refresh queues exactly one rerun), abandons projects still unanswered after 30 s while showing partial results, avoiding self-inflicted GitLab rate limiting with many projects (`src/app/pipelines.rs`).
- **In-App Home Launchpad & Quick Access**: Added `MenuSelection::Home` as the default launchpad on startup with live project/branch context and 8 interactive shortcut cards (MRs, To-Dos, Issues, Boards, CI/CD, Repository, Switch Project, Help/Command Palette) navigable via `h`/`j`/`k`/`l`/arrows or direct hotkeys, plus global `g h` shortcut (`src/models.rs`, `src/app/mod.rs`, `src/ui/list.rs`, `src/app/keymap.rs`).
- **Brand Identity & TUI Geometric Logo**: Redesigned logo as an independent geometric vector mark combining terminal velocity (`>`) in Electric Sky Blue and Git branch node (`ᚴ`) in Git Emerald Green without third-party mascots or window decorations (`docs/assets/logo.svg`, `docs/assets/logo.png`, `favicon.png`, `favicon.ico`).
- **Documentation Theme & Monochrome Aesthetic**: Aligned documentation palette with logo brand colors (Electric Sky Blue primary, Git Emerald Green accent, Obsidian Slate dark canvas), removed colored emojis, and enforced strict monochrome icon styling across all guides (`zensical.toml`, `docs/stylesheets/extra.css`, `docs/*.md`).
- **Printable Cheatsheet Redesign**: Overhauled landscape printable PDF cheatsheet with uniform keycap button dimensions (`14.5mm x 3.6mm`), razor-straight left-aligned descriptions, balanced 4×2 card grid symmetry, and embedded logo mark (`scripts/generate_cheatsheet_pdf.py`).
- **In-Terminal ASCII Banner Polish**: Cleaned in-terminal banner to a pure FIGlet wordmark with theme-aware styling and prompt tagline (`src/ui/banner.rs`).
- **Interactive Merge Confirmation Options:** Added interactive squash commits (`s`) and delete source branch (`d`) toggles inside the merge confirmation popup (`src/ui/popup/confirm.rs`, `src/app/popups.rs`).
- **Side-by-Side Diff Viewer:** Added side-by-side dual-column diff viewer toggled with `w` in the Diff view (`src/ui/diff.rs`, `src/app/keymap.rs`).
- **Resilient Diff & File Status Handling:** Added explicit notices for binary files and empty diffs preventing infinite API reload loops, and rendered file status badges (`[-]` deleted, `[+]` added, `[R]` renamed) in the diff file tree (`src/app/mod.rs`, `src/ui/diff.rs`).
- **Optimistic Kanban Card Movement:** Moving cards across columns (`H` / `L`) now updates board state, card selection, labels, and issue state immediately with optimistic local updates, eliminating server latency and cursor snap-back (`src/app/mod.rs`).
- **Kanban Quick Issue Close / Reopen:** Added direct issue state toggling with `c` while highlighting any card on a Kanban board (`src/app/keymap.rs`).
- **Kanban Column Cursor & Footer Polish:** Maintained column selection on board updates and added a dedicated Kanban keyboard shortcuts footer bar (`src/app/mod.rs`, `src/ui/boards.rs`).
- **Gruvbox Light Theme:** Added built-in Gruvbox Light preset with authentic Gruvbox palette and high contrast.
- **Dynamic Contrast Helpers:** Added luminance-based `get_contrast_color` and updated `get_contrasting_text_color` to select dark text (`#11111b`) or light text (`#cdd6f4`) based on WCAG perceived luminance.
- **Local Git Status:** Shows the current local branch and a dirty marker when run inside a git repository.
- **Smart Project Selection:** Automatically selects the project if it is the only one available for the user/service account.
- **CLI Arguments:** Added `--help`, `--version`, `--project`, `--url`, `--token`, `--config` and `--insecure`; headless flags accept `--flag=value` (`src/cli.rs`).
- **Config File:** `[gitlab]` table (`url`, `token`, `insecure`, `project`) in `~/.config/lazygitlab/config.toml` or `--config`, with CLI > environment > file precedence (`src/config.rs`).
- **Security:** Redacted GitLab personal access tokens, Bearer tokens and the configured token in application logs, error messages, the status bar and stderr (`src/redaction.rs`).
- **Setup Wizard:** First-run terminal wizard when no token is configured; the token can be saved to `config.toml` (written atomically, mode `0600`). Without a terminal, lazygitlab now exits with a clear message instead of panicking (`src/setup.rs`).

### Changed
- **Theme & Color Contrast Optimization:** Audited all theme presets (Catppuccin Mocha/Latte, Tokyo Night, Nord, Dracula, Gruvbox Dark/Light, GitHub Dark/Light, Default) for WCAG compliance. Replaced hardcoded black popup backgrounds and clashing button/badge foregrounds with theme surfaces and luminance-adaptive foregrounds across lists, metadata inspector, diffs, boards, job logs, and modals.

## [0.16.0] - 2026-08-24

### Added
- **Render-path caching** (perf P0): job-log view is now viewport-sliced (O(viewport) instead of O(document) per frame), file-viewer syntect highlighting is cached per blob, and markdown rendering is memoized across description/notes/diff comments.
- **Dirty-flag rendering** (perf): the event loop no longer redraws at 10 Hz unconditionally; frames render only on input/updates/spinner cadence. Detail-fetch debounce stops rapid list navigation from firing a discussions fetch per keystroke (250 ms same-target debounce).
- **Cross-project Pipelines dashboard tab** with project labels and `g p` fallback — cross-project failed-pipeline triage without switching projects.
- **Epic drill-in**: `Enter` on an epic lists its child issues; `Enter` opens an issue, `Esc` backs out.
- **Milestone grouping toggle** (`V` on Issues) with open/closed counts per milestone header; issue rows now show milestone chips and due dates (overdue in red).
- **Pipeline schedules** (`S` on Pipelines): view cron/description/ref and run a schedule now (`Enter`).
- **CI variable management** (`v` on Pipelines): create (`n`), edit value (`Enter`), delete (`d`) project CI variables from a popup.
- **Real approvals** in the diff header and merge-confirmation popup (required/left + approver names), fetched alongside other MR metadata; gracefully absent on instances without the approvals API.
- **Diff thread navigation across files**: `n`/`p` now sweep the whole MR instead of stopping at file boundaries.
- **MR → head pipeline jump** (`P` on MR detail): straight to that pipeline's jobs panel.
- **ETag conditional GET for Todos polling**: repeated polls send `If-None-Match` and treat `304 Not Modified` as a no-op, cutting payload work when nothing changed (`http` crate added to drive raw requests through the gitlab client).
- **Branch delete confirmation** popup, matching merge/rebase safety.
- **List polish**: MR rows show state colors; long titles truncate with ellipsis.
- **MR filter segments `Review` and `Assigned`** (`[` / `]` or click): "Review" lists open MRs where you are a reviewer, "Assigned" those assigned to you — the reviewer's daily queue finally exists server-side via `assignee_id` / `reviewer_id` params on the MR listing (`src/api.rs::PagedMergeRequests`, `src/app/mrs/mod.rs`). Segments are now `Opened │ Review │ Assigned │ Merged │ Closed │ All`.
- **Pipeline status segments** `All │ Running │ Failed │ Passed` (`[` / `]`) with client-side status filtering (`src/app/mod.rs::filter_lists`) — failed-pipeline triage no longer means scroll-hunting.
- **Job-log search and navigation**: `/` opens a log-search prompt (Enter jumps to the next match, wrapping), `G`/`End` jump to the end, `Home` to the start, `PgUp`/`PgDn` page by viewport (`src/app/keymap.rs`, help entries added).
- **Repository-tree branch browsing**: press `b` in the Repository view (file viewer) to open a branch picker; selecting a branch re-fetches the tree and subsequent file opens are scoped to that ref. The Repository list title shows the active ref (`Repository [<branch>]`). (`src/app/global.rs::select_repository_ref`, `src/ui/popup/tree.rs`, keybinding in `src/app/keymap.rs`)
- **Related items in the detail metadata panel**: MR detail now shows related merge requests (`related_merge_requests`) and the issues it closes (`closes_issues`); issue detail shows linked issues (`links`) and closing MRs (`closed_by`). All four are lazy-fetched on selection via background tasks and guarded against stale application. New endpoint types `RelatedMergeRequests`, `MRClosesIssues`, `IssueLinks`, `IssueClosedBy` in `src/api.rs`; fetcher `App::fetch_related_items` in `src/app/mrs/mod.rs`.
- **Metadata panel no longer clips its bottom rows**: the fixed 11-row block was already cutting off Status/Participants before this release; height is now computed from the rendered rows.
- **Five-persona expert review round** (DevOps, developer/MR-reviewer, project manager, UX/UI designer, Rust performance) — quick wins implemented:
  - Filter input no longer swallows `j`/`k` — they type into the filter; arrow keys navigate (`src/app/keymap.rs`).
  - Create-branch popup now actually renders as a titled two-field form with an existing-branches dropdown; previously only an anonymous generic input box appeared (`src/ui/popup/forms.rs::render_create_branch_popup`).
  - Diff header relabeled `Approvals:` → `Reactions:` — it counted upvotes, not approvals (misleading pre-merge info).
  - `Y` on a selected MR copies its source branch name for local checkout; help entry added.
  - Related/closing items in the metadata panel show title and colored state, not bare `#iid`.
  - Overdue issue due dates render bold red with an `(overdue)` suffix.
  - Main lists (Todos/MRs/Issues/Pipelines) show an explanatory empty-state line instead of a blank pane.
  - Footer status messages are color-coded by outcome (success green / failure red / in-progress orange).
  - Startup failures surface in the footer error banner instead of being swallowed (`src/runner.rs`).
- `tests/tree_branch_select.rs` (4 wiremock tests), `tests/related_items.rs` (4 wiremock/render tests), `tests/status_expiry.rs` (3 unit-style tests).

### Fixed
- **Transient footer messages never expired.** Confirmations like "Copied to clipboard!", "Pipeline #N cancelled" or "Browsing tree at 'x'" stayed on screen until overwritten by another message. Non-progress status messages now expire after 6 s (`App::tick_status`, driven from the event loop).
- **API errors were sticky.** A single failed request stained the footer red ("Error: ...") until restart. Errors now expire after 15 s.
- Removed leftover confused comments from the `g i` jump-shortcut handler (behavior unchanged).

### Changed
- CI: added push-only live-smoke tier (`test:live-smoke:push`) so every branch push gets smoke coverage; the existing `test:live-smoke` job now covers MR pipelines and manual runs only, avoiding duplicate runs on default-branch pushes.
- Refactored `src/app/mrs.rs` (1,160 LOC) into a module directory: `mrs/mod.rs` (listing/discussions), `mrs/diffs.rs`, `mrs/meta.rs`, `mrs/review.rs`. No behavior change.
- Headless snapshot tests sanitize the version/git-hash footer so snapshots don't break on every version bump or dirty-tree build.
- Performance: `[profile.release]` gains `lto = "thin"` + `codegen-units = 1`; per-call `Regex::new` on hot paths (job-log parsing, ANSI stripping, todo URL extraction) hoisted to `LazyLock` statics.
- P4 investigation closed: GitLab offers no public streaming API for entities and webhooks need an inbound server a TUI cannot host; keeping adaptive polling, with ETag conditional GET noted as the future optimization path (see ROADMAP).

## [0.15.0] - 2026-07-06

### Added
- `--headless` flag in the binary: drives the TUI from a text event script with `TestBackend`, no terminal required. Supports `--width`, `--height`, `--script <path>`, `--frames-dir <path>`.
- `tests/headless_snapshots.rs`: 2 insta snapshot tests capturing the rendered TUI buffer for the initial dashboard and after a `g m` navigation.
- `tests/headless_script.rs`: 9 unit tests for the headless event-script parser (keys, modifiers, function keys, mouse, comments, options, **FIFO delivery order**).
- `tests/job_log_copy.rs`: 5 tests covering the job-log copy interaction (`y` on current line / selection, Shift+jk extends selection, OSC 52 escape sent) and verifying that the help popup lists the copy/extend/toggle/retry shortcuts when in JobLog or Pipelines context.
- `tests/api_equivalence.rs`: 11 live tests asserting the TUI's data model deserializes correctly from the GitLab REST API for the major entity types (Project, MergeRequest, Issue, Label, Milestone, Member, Pipeline, Discussion, Note, Todo, ProjectVariables). All gated behind `#[ignore = "live_full"]`.
- `tests/data_update_round_trip.rs`: 4 live tests that fetch an entity via API, push it through `DataUpdate::*`, and assert the resulting state matches the source. Catches post-fetch mutations in the dispatcher.
- `tests/pipeline_log_flow.rs`: 3 live end-to-end tests for the pipeline → MainDetail → JobLog drill-in. Repro the user-reported broken interaction (see Fixed).
- `tests/file_viewer.rs`: 3 wiremock tests for the file viewer (scroll and viewport are independent of the job log, `j`/`k` scroll the file by 1, scroll clamps to `line_count - viewport_height`).
- `src/ui/mod.rs`: file viewer now uses `syntect` for syntax highlighting. `App.syntax_set` and `App.theme_set` are loaded at startup (this was already the case) but were never used; the file viewer now picks the syntax by file extension and renders highlighted spans via the loaded theme.
- `src/api.rs`: added `DeleteBranch` (DELETE /projects/:id/repository/branches/:branch) alongside the existing `CreateBranch` (POST ...). Both are exposed via `App::create_branch_sync` and `App::delete_branch_sync` (in `src/app/pipelines.rs`).
- `tests/branch_create_delete.rs`: 5 live tests for the create / delete branch UI (`n` opens the popup with a pre-filled ref, `Tab` cycles the focus, validation rejects empty name / empty ref, `Esc` dismisses, `D` on a selected branch triggers the delete and updates the local list).
- `tests/issue_resolve.rs`: 4 live tests for the issue-discussion resolve path: function sets a "Resolving ..." / "Reopening ..." status, the `s` key dispatches to `resolve_issue_discussion` when an issue is selected, validation surfaces a "No issue discussion selected" status when no discussion is selected, and the help popup now lists the `s` shortcut under a new `Discussion` category when there are discussions to resolve.
- `src/ui/markdown.rs`: GFM tables now render as properly aligned box-drawing tables (`│` `─` `├` `┤`) with header rows, computed column widths, and padding -- instead of the old "--- Table ---" placeholder. Inline formatting (`**bold**`, `` `code` ``, `_italic_`) inside table cells is preserved. Two new unit tests (`test_render_markdown_table_with_aligned_columns`, `test_render_markdown_table_with_inline_formatting`) lock in the behavior.
- `tests/pipeline_actions.rs`: 6 live tests for the pipeline list actions: cancel (C), retry (R), launch new (n then p), popup dismiss (Esc), help-popup entries, and the safe "cancel finished" path that does not mutate state.
- `insta` dev-dep (justified: snapshot diffing for the rendered TUI buffer per view × per entity state).

### Fixed
- **Pipeline → JobLog drill-in was a silent no-op.** Pressing `Enter` on a pipeline opened `MainDetail` and *fired* `fetch_jobs` (async, fire-and-forget), so the jobs panel rendered as "No jobs found". Pressing `Enter` again before the fetch completed did nothing, because the MainDetail+Pipelines branch only acted `if let Some(j) = self.jobs.get(i)`. The user had to press `Enter` a third time once the fetch landed, and most just assumed the interaction was broken. Now `go_forward` awaits the fetch synchronously on the same `Enter` that switched to `MainDetail`, sets a `Loading pipeline jobs...` status message, and the next `Enter` opens the JobLog.
- **File viewer re-used the job log's scroll position.** `render_tree_view` and the `Tree` keyboard handler read/wrote `app.job_log_scroll` and `app.job_log_viewport_height` instead of file-specific fields. After viewing a file, scrolling the log or switching back to JobLog would jump to wherever the file was scrolled (or vice versa). Added `file_scroll` and `file_viewport_height` on `App`, routed the file viewer's render + `j`/`k` handlers through them, and reset `file_scroll = 0` on file open. The file viewer now has its own scroll position independent of the job log.
- **`CreateBranch` API existed but there was no UI to use it.** The `Branches` list view let you select an existing branch but the `n` key didn't open a popup to create a new one, and there was no `DeleteBranch` struct in the codebase at all. Added `show_create_branch_popup`, `create_branch_name`, `create_branch_ref_input`, the two new `InputTarget::{CreateBranchName, CreateBranchRef}` variants, a new `n`-on-`Branches` handler that opens the popup with a pre-filled ref (current pipeline's ref, or the project's default branch), `Tab` to cycle the focus, `j/k` to pick from the existing-branches dropdown in the ref field, `Ctrl+Enter` to submit, `Esc` to dismiss, and a `D`-on-`Branches` handler that deletes the selected branch. Both `create_branch_sync` and `delete_branch_sync` set status messages and update the local `project_branches` list (added / removed).
- **Issue discussions could not be resolved.** `resolve_discussion` in `src/app/mrs.rs:871` was MR-only — the `s` key on an issue-detail Discussions panel would collapse the thread but never POST to the API, so "Resolved" status never stuck. Added `resolve_issue_discussion` in `src/app/issues.rs` (mirrors the MR path: PATCH `ResolveIssueDiscussion`, then refetch the issue + the discussions list), and routed the `s` key on the Discussions panel to call it when the selected entity is an issue. The help popup now lists the `s / S` shortcut under a new `Discussion` category when there are discussions to resolve.
- **`fetch_job_log` had the same problem.** It was a fire-and-forget `tokio::spawn`, so the log panel was empty when the user tried to navigate or copy. Added `fetch_job_log_sync` and used it in the drill-in path. `y` now actually copies the current line / selection and the status bar reports `Copied N lines to clipboard!`.
- **Section toggle was unreachable.** The `parse_job_log` regex required a trailing `\r`, but `str::lines()` strips line terminators, so `is_section_start` / `is_section_end` were never set on real logs, and the parser left `section = None` for every line — so `Tab` did nothing and `y` only ever copied the line the cursor sat on, not the section context the user thought they had. Fixed the regex (`r"section_start:(\d+):([^\r\n]+)"` without the trailing `\r`) and made the parser track the current section across lines so any line in the section reports the section name (and so `toggle_job_section` works on lines between `section_start` and `section_end`).
- **Pipeline cancel / retry / launch had no user feedback.** `cancel_pipeline`, `retry_pipeline`, and `submit_pipeline_launch` were all `tokio::spawn` fire-and-forget; pressing `C` / `R` / `n p` left the user staring at the same screen until the background refresh landed. Added sync variants (`cancel_pipeline_sync`, `retry_pipeline_sync`, `submit_pipeline_launch_sync`) that set a status message (`Cancelling pipeline #N...` → `Pipeline #N cancelled` or `Cancel failed: ...`) and refresh on success. The `C` and `R` handlers now use the sync variants.
- **Pipeline popup submit (`p`) was a no-op when the popup was open.** The key was gated on `!self.input_mode`, but the popup itself sets `input_mode = true`, so pressing `p` in the popup did nothing — the user could only navigate between inputs with `Tab`/`Enter` and could not launch. Loosened the gate to `!self.input_mode || self.show_pipeline_popup`.
- **Help popup listed no Pipeline / Job Log category.** Users had no discoverable way to learn the interactions for copying from a job log, retrying a failed job, launching a new pipeline, etc. Added the categories (now also lists `C` to cancel a pipeline and `R` to retry a pipeline).
- **Headless event source delivered events in LIFO order.** `ScriptedEventSource::next_event` used `Vec::pop()`, so any script whose last line was `q` terminated the loop before any other event was seen. Switched to `VecDeque::pop_front()`.

### Changed
- Modularized `src/app/mod.rs` (3.7k LOC) by extracting keymaps to `src/app/keymap.rs` and menus/navigation to `src/app/menus.rs`.
- Modularized TUI popup rendering in `src/ui/popup.rs` (803 LOC) by splitting it into individual popup module files under a new `src/ui/popup/` directory.
- `scripts/populate_test_ground.py`: also seeds the canonical deep-tree file (`dir-0/dir-1/.../file-0.txt`) on the project's default branch so the live TUI's default-branch repository-tree fetch can find it. (Previously only seeded on `feature-branch-00`.)
- `tests/stress_test_exhaustive_e2e.rs`: now sets `app.issue_filter = IssueFilter::All` before fetching so the issues count assertion uses the full set rather than the default `Opened` filter.
- `run_app` extracted to `src/runner.rs` in the lib (was in `src/main.rs`) so integration tests can drive the full event loop. Added an optional `FrameCaptureFn<B>` parameter for per-frame capture; the binary's `run_with_terminal` passes `&mut None`.
- `src/app/pipelines.rs`: added `fetch_jobs_sync` and `fetch_job_log_sync` that block on the API call and update `self` directly. The async `fetch_jobs` / `fetch_job_log` are kept for the background refresh path; the Enter handler now uses the sync variants.
- Test infrastructure: consolidated to a single canonical test ground project (`abletterer/lazygitlab-test`, id resolved via `data/test_ground.json`). `scripts/populate_test_ground.py` replaces the old `populate_stress_test.py` and is now idempotent (GET-first, skip-if-exists). All live tests are gated behind `#[ignore = "live_smoke"]` or `#[ignore = "live_full"]`. Two-tier run via `make test-live-smoke` / `make test-live-full` (or the PowerShell `scripts/test.ps1` mirror).
- CI: added `.gitlab-ci.yml` with `fmt` + `clippy` + `test:wiremock` on every push, `test:live-smoke` on merge requests, `test:live-full` on nightly schedule + manual trigger, and `coverage:tarpaulin` (Linux-only, scheduled). Required CI variables: `GITLAB_TEST_TOKEN`, `GITLAB_TEST_URL` (masked).

## [0.14.8] - 2026-06-27

### Added
- `README.md` with onboarding, install, config, keybindings, and architecture overview.
- `CHANGELOG.md` following the Keep a Changelog format.
- `LICENSE` (MIT).
- `AGENTS.md` with maintenance instructions for AI agents and contributors (documentation map, doc-sync rules, release workflow, coding conventions).

### Changed
- `ROADMAP.md` rewritten to reflect the actual current state. The original four-phase plan is complete; remaining polish and refactor work is tracked in a prioritized "What's Next" section (P0–P4).

## [0.14.7] - 2026-06-18

### Changed
- Resolve 24 clippy lints blocking the release checklist (`collapsible_if`, `needless_return`).

## [0.14.6] - 2026-06-15

### Fixed
- Only show resolution status prefixes for resolvable discussions (avoids noise on system / unresolvable threads).

## [0.14.5] - 2026-06-12

### Fixed
- `Todo.project` is now optional to support epics and prevent `serde_json` deserialization failures.

## [0.14.4] - 2026-06-09

### Fixed
- Retrieve and list only issues and MRs from groups / projects the current user belongs to (privacy / 500-error fix).

## [0.14.3] - 2026-06-05

### Added
- Participant APIs for issues and MRs.
- Start date editing for issues.
- Optimistic metadata updates to eliminate visual flicker on label / member / milestone loads.

## [0.14.2] - 2026-06-02

### Performance
- Optimize issues and MRs label-details fetching.
- Prevent duplicate project-metadata requests under load.

## [0.14.1] - 2026-05-29

### Added
- Draft-state fallback chain: try `draft`, then `wip`, then a title prefix. Works on both modern and legacy GitLab versions.
- Non-blocking target-branch selection popup.

## [0.14.0] - 2026-05-25

### Changed
- TUI layout redesigned: 3-column tabbed detail view (sidebar / list / detail) with cleaner keyboard navigation.

## [0.13.3] - 2026-05-21

### Changed
- Hide single-tab segments, reclaim vertical space, style the tab divider.

## [0.13.2] - 2026-05-18

### Changed
- Release process and version bumping script updated (see `RELEASING.md`).

## [0.13.1] - 2026-05-15

### Performance
- Optimize MR loading.
- Lazy-load project metadata and diffs.
- Align project-selection focus state.

## [0.13.0] - 2026-05-12

### Added
- Show branches, pipeline, merge, and squash status in the merge / rebase popups and in the metadata panel.

## [0.12.0] - 2026-05-09

### Added
- Merge and rebase confirmation popups with detailed pre-flight status in the metadata panel.

## [0.11.0] - 2026-05-06

### Added
- MR rebase action.
- Squash toggle on the merge-confirmation popup.

## [0.10.21] - 2026-05-03

### Fixed
- Skip non-commentable lines in the diff view.
- Reviewers shown in the metadata panel.
- Merge MR shortcut `M`.
- Cache race conditions on concurrent metadata fetches.

## [0.10.20] - 2026-04-30

### Fixed
- Resolve MR-comment panics on edge-case line numbers.
- Robust line validation before posting.
- Board filtering and tree-scroll clamping.

## [0.10.19] - 2026-04-27

### Fixed
- Resolve all compiler warnings and `clippy` lints.

## [0.10.18] - 2026-04-24

### Added
- Finalize inline-code-review: line- and range-scoped comments with `CreateMergeRequestDiscussion`.

## [0.10.17] - 2026-04-21

### Performance
- Automatic throttled background fetching for massive MRs.

## [0.10.16] - 2026-04-18

### Fixed
- Robust uncollapsing and "Fetch All" support for massive MRs.

## [0.10.15] - 2026-04-15

### Fixed
- Preserve and correctly rebuild `display_lines` for collapsed diffs.

## [0.10.14] - 2026-04-12

### Added
- Local diff fallback: when GitLab collapses a diff, fetch the file at `base_sha` and `head_sha` and generate the diff locally with the `similar` crate.

## [0.10.13] - 2026-04-09

### Fixed
- Overhaul keyboard handling and fix calendar focus for the snooze-todo popup.

## [0.10.12] - 2026-04-06

### Fixed
- Use theme colors in the job-log view; fix test setup for the new theme keys.

## [0.10.11] - 2026-04-03

### Fixed
- Restore MR navigation state after tab switches; improve theme coherence across views.

[Unreleased]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.16.0...HEAD
[0.16.0]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.15.0...v0.16.0
[0.15.0]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.8...v0.15.0
[0.14.8]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.7...v0.14.8
[0.14.7]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.6...v0.14.7
[0.14.6]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.5...v0.14.6
[0.14.5]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.4...v0.14.5
[0.14.4]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.3...v0.14.4
[0.14.3]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.2...v0.14.3
[0.14.2]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.1...v0.14.2
[0.14.1]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.0...v0.14.1
[0.14.0]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.13.3...v0.14.0
[0.13.3]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.13.2...v0.13.3
[0.13.2]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.13.1...v0.13.2
[0.13.1]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.13.0...v0.13.1
[0.13.0]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.12.0...v0.13.0
[0.12.0]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.11.0...v0.12.0
[0.11.0]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.21...v0.11.0
[0.10.21]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.20...v0.10.21
[0.10.20]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.19...v0.10.20
[0.10.19]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.18...v0.10.19
[0.10.18]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.17...v0.10.18
[0.10.17]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.16...v0.10.17
[0.10.16]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.15...v0.10.16
[0.10.15]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.14...v0.10.15
[0.10.14]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.13...v0.10.14
[0.10.13]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.12...v0.10.13
[0.10.12]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.11...v0.10.12
[0.10.11]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.10.10...v0.10.11
