# Changelog

All notable changes to `lazygitlab` are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/).

> **Note:** `scripts/bump_version.py` does not update this file automatically. Add a new entry in the same PR that bumps the version. The most recent releases are at the top.

## [Unreleased]

### Changed
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
