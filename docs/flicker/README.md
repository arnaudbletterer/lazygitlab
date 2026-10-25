# Flicker measurements

Measured on master at `7858185` (release build), headless, against the fictional mock. `summary.md` has the defect counts per scenario, latency and fixture set plus the first three examples of each defect; `report.json` has every defect with its frame numbers, keys and frame lines. Re-run with `python3 tools/flicker/detect.py --out docs/flicker` (about 8 minutes with `--jobs 3`) or `make flicker` (quick subset).

## Method

- Scenarios S1 to S9 in `tools/flicker/scenarios.py`, each at 0, 100 and 300 ms mock latency, on the default fixtures (one project, a handful of items) and on `--big` (40 projects, 500 merge requests, 500 issues). S8 (projects that fail or hang) is big only. S9 switches Merge Requests and Issues through the sidebar (`Tab`, `j`, `k`); it is an addition to the requested list because S6 showed that path behaves differently from `g m` / `g i`.
- Every frame written by `--bench-out` is analysed, and every key is preceded by a `mark:K|kind|target|desc` line so a change can be tied to a key press. Filter, refresh and project keys explain list changes on their screen for `1.5 s + 8 x latency`. Mock mutations (`/__mutate`) explain a count drop of the same size for 40 s.
- Counts are defect events. A frame is only written when the text, the counts or the loading flag changes.

## What reproduces

- The instability the user describes reproduces, but not through `g m` / `g i`. S1, S2, S3 and S5 find no shrink, no pop in or out, no selection jump and no stale list: with the `g` shortcuts the lists of both screens stay in memory and come back in the next frame.
- Through the sidebar (S9, and the sidebar leg of S6) every screen change clears the lists the app holds: the counts of Merge Requests, Issues and To-Dos fall to 0, the sidebar counts disappear, and the target screen shows "No ... match this view or filter." (EMPTY_FLASH, EMPTY_WHILE_LOADING) before rows stream back in. Coming back to a screen takes more than 300 ms at 300 ms latency (STALE_AFTER_SWITCH, 7 per S9 run). In S6 on `--big`, while the list streams back in, the selection moves from one project group to the next with no key (SELECTION_JUMP).
- Changing a filter on one list wipes another list (S4): `]` on Merge Requests drops the To-Dos count from 50 to 0, `]` on Issues drops the Merge Requests count from 500 to 0 (ROWS_SHRINK, 2 per run). The screen itself keeps drawing the old rows while it reloads.
- The first visit to Issues, Pipelines or any list screen shows the no-match message while loading, before any rows (EMPTY_WHILE_LOADING, in every scenario except S7).

## What does not reproduce

POP_IN_OUT, SPINNER_STUCK (S8 hangs a project for 60 s but requests stay outstanding, so the spinner is not stuck), SCREEN_BLEED and COUNT_MISMATCH outside filter windows were never seen in any run. A merged or closed item that the mock removes in S7 disappears at the next refresh and is not reported.

## Limits

- Rows are identified by `!iid title` and `#iid title`; other screens are judged by row counts and the empty message only, so POP_IN_OUT and SELECTION_JUMP cover Merge Requests and Issues.
- Rows that vanish because a list grew above them (viewport displacement) are not POP_IN_OUT.
- Frame times are headless draw times, quantised by the 100 ms input poll; STALE_AFTER_SWITCH at 303 ms is within that noise.
- In S7 the background refresh produced no loading frame between 57 s and 87 s in the small fixture run at 0 ms (a refresh normally logs a loading frame every 5 s), so a mutation made at 72 s showed up only at 87 s. This is an observation, not a defect class of the detector.
