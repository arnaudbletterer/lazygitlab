#!/usr/bin/env python3
"""Measure list loading in the headless TUI against the generated mock.

For each screen (Todos, Merge Requests, Issues, Pipelines) and each latency,
waits for startup to settle (1 s + 3 x latency), marks the navigation instant,
starts tools/demo/mock_gitlab.py --big, runs the headless runner with a script
and --bench-out, and reports in ms: first frame (since process start), first rows visible on the
screen and full list shown (both since the navigation key), the requests made
until the first rows, and the request and byte totals at exit.

Request totals include the requests caused by the scroll script itself, which
keeps pressing j until the target is reached or the timeout expires; the count
to first rows is the cost of simply opening the screen.

"Full list" is 500 rows for Merge Requests and Issues, 120 for Todos and 50
for Pipelines (the app caps the cross-project pipelines list at 50). The
script presses j in bursts of 15 every 200 ms until the target is reached or the
timeout expires; "n/a" means it was not reached.

Usage: run_bench.py [--bin PATH] [--latencies 0,100,300] [--screens a,b]
                    [--timeout-ms 20000] [--markdown] [--json FILE]
"""
import argparse
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
MOCK = ROOT / "tools" / "demo" / "mock_gitlab.py"

SCREENS = {
    "todos": ("Todos", "g\nt", "todos", 120),
    "merge_requests": ("MergeRequests", "g\nm", "mrs", 500),
    "issues": ("Issues", "g\ni", "issues", 500),
    "pipelines": ("Pipelines", "g\np", "pipelines", 50),
}


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def wait_port(port):
    for _ in range(100):
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.2).close()
            return
        except OSError:
            time.sleep(0.1)
    raise SystemExit("mock server did not start")


def script_for(nav, field, full, timeout_ms, latency, settle=None):
    settle = 1000 + 3 * latency if settle is None else settle
    return (f"wait:{settle}\nmark:nav\n{nav}\ntimeout:{timeout_ms}\nuntil:@{field}>=1\n"
            f"until_key:j*15:@{field}>={full}\nq\n")


def parse(path, screen, field, full):
    first_frame = first_rows = full_at = None
    max_rows = 0
    nav = 0
    for line in open(path, encoding="utf-8"):
        r = json.loads(line)
        if r.get("event") == "mark" and r.get("detail") == "nav":
            nav = r
        if r.get("event") != "frame":
            continue
        if first_frame is None:
            first_frame = r
        if r["screen"] == screen:
            max_rows = max(max_rows, r[field])
            if first_rows is None and r[field] >= 1:
                first_rows = r
            if full_at is None and r[field] >= full:
                full_at = r
    return first_frame, first_rows, full_at, max_rows, nav


def run_one(binary, key, latency, timeout_ms, settle=None):
    screen, nav, field, full = SCREENS[key]
    port = free_port()
    mock = subprocess.Popen(
        [sys.executable, str(MOCK), str(port), "--big", "--latency-ms", str(latency)],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        wait_port(port)
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            (tmp / "home").mkdir()
            (tmp / "s.txt").write_text(script_for(nav, field, full, timeout_ms, latency, settle))
            env = {"PATH": os.environ.get("PATH", ""), "HOME": str(tmp / "home"),
                   "GITLAB_URL": f"http://127.0.0.1:{port}", "GITLAB_TOKEN": "bench-token-0000"}
            subprocess.run(
                [str(binary), "--headless", "--width", "150", "--height", "40",
                 "--script", str(tmp / "s.txt"), "--bench-out", str(tmp / "b.jsonl")],
                cwd=tmp, env=env, check=True, timeout=timeout_ms / 1000 + 60,
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            ff, fr, fa, max_rows, nav = parse(tmp / "b.jsonl", screen, field, full)
        stats = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/__stats"))
    finally:
        mock.terminate()
        mock.wait()
    return {
        "screen": key, "latency_ms": latency, "target_rows": full,
        "first_frame_ms": ff and ff["t_ms"],
        "nav_ms": nav["t_ms"],
        "first_rows_ms": fr and max(0, fr["t_ms"] - nav["t_ms"]),
        "first_rows_requests": fr and fr["requests"] - nav["requests"],
        "full_ms": fa and max(0, fa["t_ms"] - nav["t_ms"]),
        "full_requests": fa and fa["requests"] - nav["requests"],
        "full_bytes": fa and fa["bytes"] - nav["bytes"],
        "max_rows": max_rows,
        "total_requests": stats["total"],
        "total_bytes": stats["bytes"],
        "paths": {k: v["count"] for k, v in stats["paths"].items()},
    }


def fmt(v):
    return "n/a" if v is None else str(v)


def table(results, markdown):
    head = ["screen", "latency ms", "first frame", "first rows after nav", "reqs to first rows",
            "rows reached", "full list", "total reqs", "total KB"]
    rows = [head]
    for r in results:
        rows.append([r["screen"], str(r["latency_ms"]), fmt(r["first_frame_ms"]),
                     fmt(r["first_rows_ms"]), fmt(r["first_rows_requests"]),
                     f'{r["max_rows"]}/{r["target_rows"]}', fmt(r["full_ms"]),
                     str(r["total_requests"]), str(round(r["total_bytes"] / 1024))])
    if markdown:
        out = ["| " + " | ".join(rows[0]) + " |", "|" + "---|" * len(head)]
        out += ["| " + " | ".join(r) + " |" for r in rows[1:]]
        return "\n".join(out)
    widths = [max(len(r[i]) for r in rows) for i in range(len(head))]
    return "\n".join("  ".join(c.rjust(w) for c, w in zip(r, widths)) for r in rows)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--bin", default=str(ROOT / "target" / "release" / "lazygitlab"))
    ap.add_argument("--latencies", default="0,100,300")
    ap.add_argument("--screens", default=",".join(SCREENS))
    ap.add_argument("--timeout-ms", type=int, default=20000)
    ap.add_argument("--settle-ms", type=int,
                    help="wait before navigating (default 1000 + 3 x latency; 0 = cold start)")
    ap.add_argument("--markdown", action="store_true")
    ap.add_argument("--json")
    ap.add_argument("--from-json", help="print the table of an earlier --json file")
    args = ap.parse_args()
    if args.from_json:
        print(table(json.loads(Path(args.from_json).read_text()), args.markdown))
        return 0
    results = []
    for latency in [int(x) for x in args.latencies.split(",")]:
        for key in args.screens.split(","):
            print(f"running {key} at {latency} ms ...", file=sys.stderr, flush=True)
            results.append(run_one(args.bin, key, latency, args.timeout_ms, args.settle_ms))
    print(table(results, args.markdown))
    if args.json:
        Path(args.json).write_text(json.dumps(results, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
