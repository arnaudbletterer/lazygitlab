# Performance measurements

How long list screens take to appear, measured on a reproducible setup, and what the numbers say. Nothing here changes behaviour: the numbers describe the code at commit `1c73c76` (v0.16.0). Fixes to list stability live on other branches and are not included.

## Method

- The app runs headless (`--headless --script --bench-out`) against `tools/demo/mock_gitlab.py --big`: 40 projects, 500 merge requests and 500 issues (skewed: the largest project holds 117 of each, so a few projects need several pages), 120 todos, 400 pipelines, real GitLab pagination headers, and a fixed delay per request (`--latency-ms`).
- Each scenario is a fresh process. `tools/bench/run_bench.py` waits for startup to settle (1 s + 3 x latency), marks the navigation instant, jumps to the screen (`g t`, `g m`, `g i`, `g p`), then presses `j` in bursts of 15 every 200 ms until the full list is shown or 20 s pass.
- Times are milliseconds. "First frame" is since process start. "First rows" and "full list" are since the navigation key. "Rows reached" is the highest row count the screen held, against the target (500 for merge requests and issues, 120 for todos, 50 for pipelines, which the app caps at 50). Request and byte totals are counted at the mock and include requests triggered by the scrolling itself.
- Release build, one run per cell, Linux 7.2 on 8 cores, loopback, Python mock in a single process. Expect run-to-run noise of a few tens of ms. Another compile was running on the machine during part of the session, so treat small differences as noise.
- Reproduce with `make bench` (about 8 minutes). `make bench` builds in release mode and runs the three latencies.

## Baseline

Warm: navigation after startup has settled.

| screen | latency ms | first frame | first rows after nav | reqs to first rows | rows reached | full list | total reqs | total KB |
|---|---|---|---|---|---|---|---|---|
| todos | 0 | 11 | 0 | 0 | 100/120 | n/a | 3435 | 1593 |
| merge_requests | 0 | 11 | 102 | 40 | 117/500 | n/a | 6064 | 6068 |
| issues | 0 | 15 | 102 | 40 | 117/500 | n/a | 1854 | 1853 |
| pipelines | 0 | 13 | 106 | 40 | 50/50 | 106 | 48 | 152 |
| todos | 100 | 123 | 2 | 0 | 50/120 | n/a | 3437 | 1573 |
| merge_requests | 100 | 111 | 204 | 40 | 50/500 | n/a | 5625 | 5520 |
| issues | 100 | 114 | 204 | 40 | 117/500 | n/a | 1899 | 1853 |
| pipelines | 100 | 111 | 608 | 40 | 50/50 | 608 | 48 | 152 |
| todos | 300 | 311 | 1 | 0 | 50/120 | n/a | 3272 | 1514 |
| merge_requests | 300 | 330 | 408 | 40 | 50/500 | n/a | 5193 | 5322 |
| issues | 300 | 322 | 407 | 40 | 117/500 | n/a | 1822 | 1853 |
| pipelines | 300 | 320 | 1627 | 40 | 50/50 | 1627 | 48 | 152 |

Cold: navigation key sent immediately at startup (`--settle-ms 0`, 5 s timeout).

| screen | latency ms | first frame | first rows after nav | reqs to first rows | rows reached | full list | total reqs | total KB |
|---|---|---|---|---|---|---|---|---|
| todos | 0 | 24 | 104 | 7 | 100/120 | n/a | 796 | 480 |
| merge_requests | 0 | 17 | 207 | 48 | 117/500 | n/a | 1431 | 2215 |
| issues | 0 | 52 | 327 | 48 | 117/500 | n/a | 515 | 899 |
| pipelines | 0 | 25 | 5226 | 46 | 50/50 | 5226 | 48 | 152 |
| todos | 100 | 125 | 205 | 9 | 50/120 | n/a | 837 | 474 |
| merge_requests | 100 | 125 | 406 | 48 | 50/500 | n/a | 1339 | 2101 |
| issues | 100 | 131 | 408 | 48 | 117/500 | n/a | 500 | 899 |
| pipelines | 100 | 117 | 5707 | 48 | 50/50 | 5707 | 49 | 152 |
| todos | 300 | 331 | 407 | 9 | 50/120 | n/a | 780 | 458 |
| merge_requests | 300 | 319 | 818 | 48 | 50/500 | n/a | 1273 | 2077 |
| issues | 300 | 328 | 814 | 48 | 117/500 | n/a | 554 | 951 |
| pipelines | 300 | 312 | 6610 | 48 | 50/50 | 6610 | 49 | 152 |

Reading the tables:

- Todos are already loaded by the Home screen prefetch in the warm run, hence about 0 ms. Cold, they take one round trip plus one 100 ms tick.
- Merge requests and issues show their first 50 rows after about one round trip plus a 100 ms tick (204 ms at 100 ms latency, 408 ms at 300 ms), using 40 parallel requests, one per project.
- Pipelines take 106, 608 and 1627 ms warm, and 5.2 to 6.6 s cold.
- No scenario reached the full 500 rows in 20 s. Merge requests stayed at 50 (117 at 0 ms latency), issues at 117, todos at 50 (100 at 0 ms).

## Top five time sinks

Ranked by what a user waits for. Each item cites the measurement that supports it.

1. **List loading is gated on a serial startup chain.** Startup costs one request before the first frame (first frame equals about one latency), then the project membership sync (`/projects`), and only then the per-project fan-out. When the user opens Pipelines before the sync has finished, nothing is fetched until the next 5 s background refresh: cold Pipelines first rows take 5226, 5707 and 6610 ms at 0, 100 and 300 ms latency, against 106, 608 and 1627 ms when warm. The list requests of every cross-project screen wait on this chain.
2. **Per-project fan-out with no cap for issues and merge requests, a cap of 8 for pipelines.** Opening a global list issues 40 requests (one per member project), whatever the project holds: first rows cost 40 requests on every list screen. Pipelines limit this to 8 in flight, so 40 projects need 5 waves: 608 ms at 100 ms latency and 1627 ms at 300 ms, against 204 and 408 ms for merge requests with the same 40 requests unbounded.
3. **Pagination of the global lists re-runs the fan-out and stalls.** Each next page re-requests page N from all 40 projects (also the ones with fewer than 50 items), merges, sorts and keeps only 50 rows, so rows beyond the cut are lost for good. With 100 ms latency the merge request list made 160 list requests (four fan-outs) and still held 50 of 500 rows; issues made 480 and held 117; todos held 50 of 120 at 100 and 300 ms. A full 500-row list is not reachable by scrolling.
4. **N+1 detail prefetch for every highlighted row.** Moving the cursor fetches the item and its extras: per merge request row one request each for the merge request, participants, award emoji and discussions (1371, 1371, 1359 and 1356 requests in the 100 ms merge request run), per issue row one discussions request (1411 in the issues run). Scrolling a list therefore multiplies traffic by about 4 per row. A cheap alternative is to fetch details on dwell or on open only.
5. **Redundant fetches and a 100 ms input poll.** The event loop applies network updates only after its 100 ms input poll returns, so every result waits up to 100 ms: at 0 ms latency merge requests and issues take 102 ms for first rows, which is exactly one poll. Page size is fixed at 50 although GitLab allows 100, so todos need three pages for 120 rows. The Home screen prefetches todos and merge requests even when the user goes elsewhere, and the background refresh re-runs the same fan-out every 5 s while a list is open.

## Trade-offs of the measurement tooling

- The request counter and the optional trace wrap the HTTP client (`src/trace.rs`). The counters are two atomic adds per request; the trace line is built only when `LAZYGITLAB_TRACE=1`.
- The headless observer renders each frame to a string and compares it with the previous one, which costs a few hundred microseconds per drawn frame. It only runs in headless mode.
- The mock runs in one Python process, so absolute numbers at 0 ms latency include its service time (a few ms per request). Compare cells with each other, not with a real GitLab. Use the trace below to get real numbers.

## Measuring against a real GitLab

```bash
LAZYGITLAB_TRACE=1 lazygitlab
scripts/summarize_trace.py ~/.local/state/lazygitlab/lazygitlab.log
scripts/summarize_trace.py --extract ~/.local/state/lazygitlab/lazygitlab.log > trace.jsonl
```

The trace has one JSON line per API request: method, endpoint template (numeric ids replaced by `:id`, project and group paths by `:path`), status, duration in ms, response bytes, page and whether the answer was a 304. The host, token, project names and query values are never written, so `trace.jsonl` is safe to send. The rest of the log can contain project paths, so send only the extracted file.
