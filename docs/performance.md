# Performance measurements

How long list screens take to appear, measured on a reproducible setup, and what the numbers say. Nothing here changes behaviour: the numbers describe the code at commit `1c73c76` (v0.16.0). Fixes to list stability live on other branches and are not included.

## Method

- The app runs headless (`--headless --script --bench-out`) against `tools/demo/mock_gitlab.py --big`: 40 projects, 500 merge requests and 500 issues (skewed: the largest project holds 117 of each, so a few projects need several pages), 120 todos, 400 pipelines, real GitLab pagination headers, and a fixed delay per request (`--latency-ms`).
- Each scenario is a fresh process. `tools/bench/run_bench.py` waits for startup to settle (1 s + 3 x latency), marks the navigation instant, jumps to the screen (`g t`, `g m`, `g i`, `g p`), then presses `j` in bursts of 15 every 200 ms until the full list is shown or 20 s pass.
- Times are milliseconds. "First frame" is since process start. "First rows" and "full list" are since the navigation key. "Rows reached" is the highest row count the screen held, against the target (500 for merge requests and issues, 120 for todos, 50 for pipelines, which the app caps at 50). Request and byte totals are counted at the mock and include requests triggered by the scrolling itself.
- Times are the instants at which the headless `TestBackend` frame was drawn, not terminal paint times, and they include the up to 100 ms granularity of the input poll in the event loop (network results are applied at the next loop iteration).
- Release build, one run per cell, Linux 7.2 on 8 cores, loopback, Python mock in a single process. The run happened with a load average of 3.5 at the start falling to 1.1 at the end (other agents were working on the machine, but no compile was running), so treat differences under about 100 ms as noise: for example the 100 ms merge request cell (316 ms) is one poll tick above its neighbours.
- The raw results of this run are committed: `docs/performance-data-warm.json` and `docs/performance-data-cold.json` (one object per cell, including the request count per endpoint template in `paths`). Every figure below comes from these files or from the tables.
- Reproduce with `make bench` (about 5 minutes; it builds in release mode and runs the three latencies). The cold table uses `python3 tools/bench/run_bench.py --settle-ms 0 --timeout-ms 5000`; add `--json FILE` to keep the raw results.

## Baseline

Warm: navigation after startup has settled.

| screen | latency ms | first frame | first rows after nav | reqs to first rows | rows reached | full list | total reqs | total KB |
|---|---|---|---|---|---|---|---|---|
| todos | 0 | 22 | 1 | 0 | 100/120 | n/a | 3185 | 1493 |
| merge_requests | 0 | 18 | 103 | 40 | 117/500 | n/a | 5364 | 5722 |
| issues | 0 | 28 | 220 | 40 | 117/500 | n/a | 1838 | 2228 |
| pipelines | 0 | 106 | 104 | 40 | 50/50 | 104 | 48 | 152 |
| todos | 100 | 121 | 1 | 0 | 50/120 | n/a | 3206 | 1476 |
| merge_requests | 100 | 112 | 316 | 40 | 50/500 | n/a | 5734 | 5799 |
| issues | 100 | 111 | 204 | 40 | 117/500 | n/a | 1940 | 2078 |
| pipelines | 100 | 111 | 609 | 40 | 50/50 | 609 | 48 | 152 |
| todos | 300 | 311 | 1 | 0 | 50/120 | n/a | 3416 | 1572 |
| merge_requests | 300 | 311 | 406 | 40 | 50/500 | n/a | 5593 | 5520 |
| issues | 300 | 311 | 405 | 40 | 117/500 | n/a | 1953 | 2228 |
| pipelines | 300 | 311 | 1521 | 40 | 50/50 | 1521 | 48 | 152 |

Cold: navigation key sent immediately at startup (`--settle-ms 0`, 5 s timeout).

| screen | latency ms | first frame | first rows after nav | reqs to first rows | rows reached | full list | total reqs | total KB |
|---|---|---|---|---|---|---|---|---|
| todos | 0 | 12 | 102 | 9 | 100/120 | n/a | 867 | 508 |
| merge_requests | 0 | 11 | 205 | 47 | 117/500 | n/a | 1609 | 2299 |
| issues | 0 | 11 | 205 | 46 | 117/500 | n/a | 529 | 899 |
| pipelines | 0 | 11 | 5165 | 48 | 50/50 | 5165 | 49 | 152 |
| todos | 100 | 111 | 204 | 9 | 50/120 | n/a | 876 | 488 |
| merge_requests | 100 | 111 | 407 | 48 | 50/500 | n/a | 1491 | 2162 |
| issues | 100 | 111 | 305 | 48 | 117/500 | n/a | 530 | 899 |
| pipelines | 100 | 111 | 5577 | 48 | 50/50 | 5577 | 49 | 152 |
| todos | 300 | 311 | 407 | 9 | 50/120 | n/a | 817 | 474 |
| merge_requests | 300 | 311 | 711 | 48 | 50/500 | n/a | 1389 | 2135 |
| issues | 300 | 311 | 810 | 48 | 117/500 | n/a | 514 | 899 |
| pipelines | 300 | 311 | 6613 | 48 | 50/50 | 6613 | 49 | 152 |

Reading the tables:

- Todos are already loaded by the Home screen prefetch in the warm run, hence about 0 ms. Cold, they take one round trip plus one 100 ms tick.
- At 0 ms latency the first rows of merge requests take about 100 ms, which is the input poll granularity, not network time.
- Merge requests and issues show their first 50 rows after about one round trip plus a 100 ms tick (204 ms at 100 ms latency, 408 ms at 300 ms), using 40 parallel requests, one per project.
- Pipelines take 106, 608 and 1627 ms warm, and 5.2 to 6.6 s cold.
- No scenario reached the full 500 rows in 20 s. Merge requests stayed at 50 (117 at 0 ms latency), issues at 117, todos at 50 (100 at 0 ms).

## After the startup chain

Same method, same mock, same machine, release build at the commit that adds this section. The machine was quiet: no compile running, load average 1.7 at the start of the cold run and 0.7 at the end of the warm run. One run per cell, so differences under about 100 ms are noise. Raw results: `docs/performance-data-cold-after.json` and `docs/performance-data-warm-after.json`. The baseline column is the "Cold" and "Warm" table above (`docs/performance-data-cold.json`, `docs/performance-data-warm.json`).

What changed: the first frame is drawn before any request (the client is built without the connection check, which was a blocking `/user`), `/user` is fetched once and in parallel with the membership sync, a screen opened before the ids arrive fetches when they arrive, network results wake the event loop, and Home's merge request prefetch is delayed and dropped when you leave Home. The mock has no persistent cache here (each scenario starts from an empty state), so the warm-start cache path is covered by `tests/startup_chain.rs`, not by these numbers.

Cold (navigation key at startup), first rows after navigation in ms, baseline then after:

| screen | 0 ms | 100 ms | 300 ms |
|---|---|---|---|
| todos | 102 then 3 | 204 then 106 | 407 then 307 |
| merge_requests | 205 then 31 | 407 then 227 | 711 then 628 |
| issues | 205 then 28 | 305 then 221 | 810 then 624 |
| pipelines | 5165 then 20 | 5577 then 616 | 6613 then 1817 |

First frame (since process start): 11, 111 and 311 ms at 0, 100 and 300 ms latency before; 10, 10 and 10 ms after, whatever the latency.

Warm pipelines first rows: 104, 609, 1521 ms before; 20, 516, 1516 ms after. Warm merge requests are 1 to 2 ms after because the Home prefetch had finished during the 1 s settle; issues are 26, 121, 322 ms against 220, 204, 405 before.

Requests: `/user` 1 instead of 3 per run; the cold pipelines run needs 45 requests instead of 49.

Caveats:

- The cold Pipelines row improves by 5 s because the fetch now starts when the member ids arrive instead of at the next 5 s refresh. The remaining 616 and 1817 ms are the 8-wide pipeline fan-out (sink 2), unchanged.
- The pipelines rows at 100 and 300 ms are one round trip for `/user` and the membership sync (parallel now) plus 5 waves of 8 requests; they were not touched here.
- "Rows reached" went from 50 or 117 to 424 or 500 for merge requests and issues. This comes from applying results immediately, so the scroll script paginates faster, not from a change to the merge rules; it also makes the total request and byte counts of those scenarios larger than the baseline, so compare first rows, not totals. The warm pipelines run shows 85 requests and 823 KB against 48 and 152 KB because the Home prefetch of 40 merge request lists now completes during the settle (before, the prefetch ran with no member ids and fetched nothing).
- Warm-start numbers (a populated membership cache) are unmeasured: every scenario above starts from an empty state, and each cell is one run.
- First frame in headless mode is the `TestBackend` draw time; a real terminal adds its own paint time.

## Top five time sinks

Ranked by what a user waits for. Each item cites the measurement that supports it.

1. **List loading is gated on a serial startup chain.** Startup costs one request before the first frame (first frame equals about one latency), then the project membership sync (`/projects`), and only then the per-project fan-out. When the user opens Pipelines before the sync has finished, nothing is fetched until the next 5 s background refresh: cold Pipelines first rows take 5226, 5707 and 6610 ms at 0, 100 and 300 ms latency, against 106, 608 and 1627 ms when warm. The list requests of every cross-project screen wait on this chain.
2. **Per-project fan-out with no cap for issues and merge requests, a cap of 8 for pipelines.** Opening a global list issues 40 requests (one per member project), whatever the project holds: first rows cost 40 requests on every list screen. Pipelines limit this to 8 in flight, so 40 projects need 5 waves: 608 ms at 100 ms latency and 1627 ms at 300 ms, against 204 and 408 ms for merge requests with the same 40 requests unbounded.
3. **Pagination of the global lists re-runs the fan-out and stalls.** Each next page re-requests page N from all 40 projects (also the ones with fewer than 50 items), merges, sorts and keeps only 50 rows, so rows beyond the cut are lost for good. In the warm run at 100 ms latency (`docs/performance-data-warm.json`) the merge request screen made 175 `/projects/:id/merge_requests` list requests and still held 50 of 500 rows; issues made 504 list requests and held 117; the "rows reached" column shows todos at 50 of 120 at 100 and 300 ms. A full 500-row list is not reachable by scrolling.
4. **N+1 detail prefetch for every highlighted row.** Moving the cursor fetches the item and its extras. In the same warm 100 ms merge request run: 1390 requests for `/projects/:id/merge_requests/:id`, 1389 for `participants`, 1386 for `award_emoji` and 1386 for `discussions`, against 175 list requests. In the issues run: 1428 `discussions` requests against 504 list requests. Scrolling therefore multiplies traffic by about 4 per merge request row. A cheap alternative is to fetch details on dwell or on open only. These counts include the scroll script, which moves much faster than a person.
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

## After wiring merge requests

Same method as above (headless app against `tools/demo/mock_gitlab.py --big`, release build, one run per cell, `make bench` scenarios and the cold variant with `--settle-ms 0 --timeout-ms 5000`), run on the branch that wires the merge request lists. Raw results: `docs/performance-data-after-warm.json` and `docs/performance-data-after-cold.json`; every figure below comes from those files or from the baseline tables above. Load average was 1.7 at the start and 0.6 at the end, with no compile running, so differences under about 100 ms are noise.

What changed for merge requests: the dashboard list asks GraphQL first and falls back to REST (the mock has no GraphQL, so this run exercises the REST fallback only: `per_page=100`, pages in parallel, at most 6 requests in flight, rows shown every 100 ms as projects answer), cached rows are painted first when a cache exists (the bench runs with a throw-away home, so the cache is off and these are cold-cache numbers), background refreshes ask only for rows changed since the last fetch, and detail requests wait for the cursor to rest 250 ms.

Warm (navigation after startup has settled), merge requests only:

| latency ms | | first rows after nav | reqs to first rows | rows reached | full list | total reqs | total KB |
|---|---|---|---|---|---|---|---|
| 0 | before | 103 | 40 | 117/500 | n/a | 5364 | 5722 |
| 0 | after | 104 | 43 | 500/500 | 104 | 49 | 905 |
| 100 | before | 316 | 40 | 50/500 | n/a | 5734 | 5799 |
| 100 | after | 204 | 8 | 500/500 | 747 | 49 | 905 |
| 300 | before | 406 | 40 | 50/500 | n/a | 5593 | 5520 |
| 300 | after | 407 | 8 | 500/500 | 2221 | 49 | 905 |

Cold (navigation key at startup):

| latency ms | | first rows after nav | reqs to first rows | rows reached | full list | total reqs | total KB |
|---|---|---|---|---|---|---|---|
| 0 | before | 205 | 47 | 117/500 | n/a | 1609 | 2299 |
| 0 | after | 205 | 49 | 500/500 | 205 | 49 | 905 |
| 100 | before | 407 | 48 | 50/500 | n/a | 1491 | 2162 |
| 100 | after | 406 | 16 | 500/500 | 951 | 50 | 906 |
| 300 | before | 711 | 48 | 50/500 | n/a | 1389 | 2135 |
| 300 | after | 712 | 16 | 500/500 | 2430 | 50 | 906 |

Reading the tables:

- The full 500 rows are now reached in every cell, where no cell of the baseline got past 117. The list is complete several round trips after the first rows at 100 and 300 ms latency, which is 41 list requests (40 projects, one with a second page) through a budget of 6 in flight.
- First rows keep the cost of one round trip plus a 100 ms tick, but need 8 to 16 requests instead of 40 at the moment they appear, because the budget of 6 releases rows as the first waves answer.
- Total requests over the scripted run fall from 5364-5734 to 49, and bytes from 5.5-5.8 MB to 0.9 MB. Most of the drop is the detail debounce: the script presses `j` 15 times per burst and the baseline sent 4 detail requests per row passed, now none are sent while the cursor moves. The other part is the baseline's repeated re-fetch of page N for all projects while paginating, which no longer exists.
- Time to the last row grows with latency (747 ms at 100, 2221 ms at 300) because the REST fallback is bounded to 6 requests in flight. With GraphQL, 45 projects take 3 requests in one wave (`tests/mr_list_perf.rs`: 500 rows over 50 projects load in under 2 s at 100 ms latency for both transports, and a refresh with nothing changed sends one request per batch and changes nothing).

Caveats:

- The "before" figures are from commit `1c73c76`; master has since gained the list stability and merge-rule fixes, which are included in "after" but not in "before". The difference between the two tables is therefore the whole of the work between them, not only this change.
- The mock has no GraphQL endpoint, so the GraphQL gain (3 requests for 45 projects) is shown by the wiremock tests, not by this bench. The cached first paint is likewise covered by a test (`cached_rows_are_drawn_before_any_network_response`), not by the bench.
- One run per cell, headless frame times, loopback and a single-process Python mock, as in the baseline.
- The REST path reads at most 5 pages (500 rows) per project and the GraphQL path at most 80 per project, so a project with more merge requests in the chosen state shows its most recently updated ones only. The project merge request list uses the REST path only.
