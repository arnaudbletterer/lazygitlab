# Changelog

All notable changes to `lazygitlab` are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/).

> **Note:** `scripts/bump_version.py` does not update this file automatically. Add a new entry in the same PR that bumps the version. The most recent releases are at the top.

## [Unreleased]

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

[Unreleased]: https://git.abletterer.synology.me/abletterer/lazygitlab/compare/v0.14.8...HEAD
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
