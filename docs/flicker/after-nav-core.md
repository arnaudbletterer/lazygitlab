# Flicker after the navigation core (iter/41-nav-core)

Full run, release build, `python3 tools/flicker/detect.py --out /tmp/flicker-after --jobs 2` (54 runs, S1 to S9, 0, 100 and 300 ms, small and big fixtures). The quick subset (`--quick`) is also clean. The "before" column is `summary.md` (master 7858185).

| defect | before | after |
|---|---|---|
| ROWS_SHRINK | 72 | 0 |
| POP_IN_OUT | 0 | 0 |
| EMPTY_FLASH | 48 | 0 |
| SPINNER_STUCK | 0 | 0 |
| SELECTION_JUMP | 4 | 0 |
| SCREEN_BLEED | 0 | 0 |
| COUNT_MISMATCH | 0 | 0 |
| STALE_AFTER_SWITCH | 16 | 0 |
| EMPTY_WHILE_LOADING | 57 | 0 |

## What changed

- Every screen change goes through `App::navigate` (`src/app/nav.rs`); sidebar keys reach the same transition through `update_preview`. It resets the detail pane (selection, discussions, diffs, jobs, related items, detail tab, view mode, pending detail request) and refreshes only when the destination list is empty or stale.
- Lists survive a screen switch. A list is emptied only when its own scope changes: its filter, another project, or dashboard versus project (`mrs_scope`, `issues_scope`, `pipelines_scope`).
- An empty list shows `Loading...` while a fetch runs (including the moment between a key press and the first loading message), `Couldn't load: ... (r to retry)` after a failed fetch, and the no-match text only when idle.
- The detail column is reserved on the To-Dos, Merge Requests, Issues and Pipelines screens, so the list width no longer jumps when a row is selected.

## Not covered

The detector measures Merge Requests and Issues rows and counts only; the other screens are covered by `tests/navigation_core.rs`.
