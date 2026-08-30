# lazygitlab Roadmap

This document tracks the current state of `lazygitlab` and what is left to do. The original four-phase plan that previously lived here is **complete** and has been merged into the **Current State** section below; new work is tracked in **What's Next**.

## Current State
- Intelligent Git Integration, Smart Project Selection, CLI Flags, and Security Sanitization.

`lazygitlab` is a mature, daily-driver TUI for GitLab. The original phased plan (multi-line editor, inline code review, repository tree, Kanban boards, global search, real-time refresh) is fully implemented, as is the entire P0-P3 backlog derived from the original roadmap and the five-persona expert review round. Recent work focused on: cross-project pipeline triage, reviewer queue filters, render-path performance, real approvals, schedules/variables management, and ETag-based polling.

### What's Built

Grouped by area. File pointers are the canonical location for the implementation.

#### Dashboard & Navigation
- **Personal dashboard tabs**: Todos, MRs, Issues, Pipelines (cross-project), Projects, Groups (`src/app/mod.rs`)
- **Cross-project Pipelines tab** with project labels and status segments (`src/app/pipelines.rs::fetch_cross_pipelines`)
- **Membership-scoped project listing** (`src/app/global.rs`)
- **Jump shortcuts** `g t` / `g m` / `g i` / `g p` / `g r` (`src/app/keymap.rs`)
- **3-column tabbed layout** (sidebar / list / detail) with focus toggling (`src/ui/mod.rs`, `src/ui/list.rs`)
- **Global filter** per list (`src/app/mod.rs::filter_lists`, shortcut `/`) that types `j`/`k` correctly
- **Group multi-project elements** in dashboard views; issues groupable by project or milestone with counts (`V`)
- **Involvement-only toggle** (`I`)

#### Search
- **Hybrid global search**: Elasticsearch-backed `GlobalSearch` + scoped `PagedProjects` / `GlobalIssues` / `GlobalMergeRequests` with dedup (`src/app/search.rs`, shortcut `f`)

#### Merge Requests
- **Inline code review on diffs**: line/range-scoped comments with `base/start/head_sha` (`src/app/mrs/review.rs`, shortcut `c`)
- **Approve / Unapprove** (`A` / `R`), **Merge / Rebase** with confirmation popups (`M` / `E`), **Squash toggle** (`S`), **Draft toggle** (`D`), **Open / Close** (`c`)
- **Real approvals display** (`/approvals`): required/left + approver names in the diff header and merge confirmation
- **Edit metadata**: title, description, labels, assignees, reviewers, milestone, target branch (`src/app/mrs/meta.rs`)
- **Discussion resolution** on MRs and issues; thread navigation sweeps all files (`n` / `p`)
- **Related MRs + closing issues** in the metadata panel (`related_merge_requests` / `closes_issues`, lazy-fetched)
- **Review queue**: `[`/`]` filter segments `Opened | Review | Assigned | Merged | Closed | All` backed by server-side `assignee_id` / `reviewer_id`
- **Throttled concurrent diff fetching** + local diff fallback for collapsed diffs (`src/app/mrs/diffs.rs`)
- **MR to head-pipeline jump** (`P`)

#### Issues
- **Create / edit / state toggle** (`src/app/issues.rs`)
- **Edit labels / assignees / milestone / due & start date / weight / confidential**
- **Issue discussions** with notes and resolution (`resolve_issue_discussion`)
- **Linked issues + closing MRs** in the metadata panel (`links` / `closed_by`, lazy-fetched)
- **Milestone chips and due dates** on issue rows; overdue dates highlighted red

#### CI/CD
- **Pipelines list** with pagination, retry, cancel, status tracking and status filter segments `All | Running | Failed | Passed` (`[` / `]`)
- **Job log viewer**: ANSI rendering, section collapsing, follow-mode, multi-line copy, `/` search with jump-to-match, `G`/`Home`/`PgUp`/`PgDn` paging
- **Launch pipeline** with ref + custom variables (`src/app/pipelines.rs`)
- **Pipeline schedules**: view and run now (`S`)
- **CI variables**: create, edit value, delete (`v`)
- **Job actions**: play, retry, cancel

#### Project view
- **Repository tree** with **branch browsing** (`b`; tree + file fetches scoped to the selected ref)
- **File viewer** with cached syntect highlighting and independent scroll state
- **Create / delete branch UI** with delete confirmation popup
- **Commits history**, **Branches** list, **Project metadata cache**
- **Kanban boards** with drag between columns; **Epics** per group with child-issue drill-in

#### Quality of life
- **Adaptive refresh**: 2s live / 5s active / 30s idle back-off (`src/runner.rs`)
- **ETag conditional GET** for Todos polling: repeated polls send `If-None-Match`, `304` short-circuits (`fetch_todos_conditional`)
- **Render-path caching**: viewport-sliced job log, cached file highlighting, memoized markdown; dirty-flag rendering in the event loop; debounced detail fetches
- **Clipboard**: local `arboard` + OSC 52 for remote/tmux; `Y` copies MR source branch
- **Self-expiring footer feedback** with outcome-colored status messages
- **GFM markdown**: aligned tables, task lists, blockquotes, footnotes (`src/ui/markdown.rs`)
- **Mouse support**, themable via `theme.toml`, persistent tracing logs, searchable help popup

## What's Next

Intentionally empty: everything from the original plan, the P0-P4 backlog, and the five-persona review round has been implemented or explicitly closed with a documented decision. New work items should be added here with an effort estimate before starting.

## Process

- Before merging anything that changes user-visible behavior: update `CHANGELOG.md`, and if it adds a new feature, also update `README.md` (Features).
- See [AGENTS.md](./AGENTS.md) for the full contributor / agent guide.
- See [RELEASING.md](./RELEASING.md) for the release workflow.
