#!/usr/bin/env python3
"""Detect list instability ("flicker") in the headless lazygitlab TUI.

Runs the release binary headless against tools/demo/mock_gitlab.py through
scripted navigation scenarios (scenarios.py), records every frame written by
`--bench-out`, and looks for defects between consecutive frames (analysers.py).
Exits 1 when any defect is found, so it can gate CI.

Usage: detect.py [--quick] [--scenarios S1,S2] [--latencies 0,100,300]
                 [--fixtures small,big] [--out DIR] [--bin PATH] [--jobs N]
                 [--report-only] [--probe SCRIPT --latency N --big]

--quick        S1-S5 at 100 ms on the big fixtures (about a minute and a half).
--out DIR      write DIR/report.json and DIR/summary.md.
--report-only  always exit 0.
--probe        run one raw script file and print a line per frame (debugging).
--keep-raw DIR keep each run's bench JSONL in DIR; --show FILE prints one.
"""
import argparse
import concurrent.futures
import json
import os
import socket
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
MOCK = ROOT / "tools" / "demo" / "mock_gitlab.py"
sys.path.insert(0, str(HERE))

import analysers  # noqa: E402
import report  # noqa: E402
import scenarios  # noqa: E402


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


def http(port, path, body=None):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(f"http://127.0.0.1:{port}{path}", data=data,
                                 method="GET" if body is None else "POST")
    return json.load(urllib.request.urlopen(req, timeout=5))


class Background(threading.Thread):
    """Sample the mock's inflight counter and fire timed mock commands."""

    def __init__(self, port, t0, timeline):
        super().__init__(daemon=True)
        self.port, self.t0, self.timeline = port, t0, sorted(timeline, key=lambda x: x["at_ms"])
        self.samples, self.mutations = [], []
        self.stop = threading.Event()

    def run(self):
        pending = list(self.timeline)
        while not self.stop.wait(0.15):
            now = int((time.monotonic() - self.t0) * 1000)
            try:
                st = http(self.port, "/__stats")
                self.samples.append((now, st["inflight"], st["total"]))
                while pending and pending[0]["at_ms"] <= now:
                    cmd = pending.pop(0)
                    http(self.port, cmd["path"], cmd["body"])
                    if cmd["path"] == "/__mutate":
                        self.mutations.append((now, cmd["body"]))
            except OSError:
                pass


def run_one(binary, name, latency, big, keep_raw=None, script_text=None, timeline=(), mock_extra=(), timeout=240):
    port = free_port()
    cmd = [sys.executable, str(MOCK), str(port), "--latency-ms", str(latency), *mock_extra]
    if big:
        cmd.append("--big")
    mock = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        wait_port(port)
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            (tmp / "home").mkdir()
            (tmp / "s.txt").write_text(script_text)
            env = {"PATH": os.environ.get("PATH", ""), "HOME": str(tmp / "home"),
                   "GITLAB_URL": f"http://127.0.0.1:{port}", "GITLAB_TOKEN": "flicker-token-0000"}
            t0 = time.monotonic()
            bg = Background(port, t0, timeline)
            bg.start()
            try:
                subprocess.run(
                    [str(binary), "--headless", "--width", "150", "--height", "40",
                     "--script", str(tmp / "s.txt"), "--bench-out", str(tmp / "b.jsonl")],
                    cwd=tmp, env=env, check=True, timeout=timeout,
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            finally:
                bg.stop.set()
                bg.join()
            raw = (tmp / "b.jsonl").read_text(encoding="utf-8")
            if keep_raw:
                Path(keep_raw).mkdir(parents=True, exist_ok=True)
                (Path(keep_raw) / f"{name}.jsonl").write_text(raw, encoding="utf-8")
        stats = http(port, "/__stats")
    finally:
        mock.terminate()
        mock.wait()
    return raw, bg, stats


def run_scenario(binary, key, latency, big, keep_raw=None):
    builder, desc, _big_only, _quick = scenarios.SCENARIOS[key]
    script = builder(latency)
    fixture = "big" if big else "small"
    name = f"{key}-{fixture}-{latency}ms"
    started = time.time()
    raw, bg, stats = run_one(binary, name, latency, big, keep_raw, script.text(), script.timeline)
    ctx = analysers.Context(latency=latency, samples=bg.samples, mutations=bg.mutations)
    events = analysers.load_events(raw.splitlines())
    defects = analysers.analyse(events, ctx)
    frames = sum(1 for e in events if e["event"] == "frame")
    return {
        "scenario": key, "description": desc, "fixture": fixture, "latency_ms": latency,
        "frames": frames, "duration_ms": events[-1]["t_ms"] if events else 0,
        "wall_s": round(time.time() - started, 1), "requests": stats["total"],
        "defects": defects,
        "counts": {t: sum(1 for d in defects if d["type"] == t) for t in analysers.DEFECT_TYPES},
    }


def probe(binary, script_file, latency, big):
    raw, bg, _ = run_one(binary, "probe", latency, big, script_text=Path(script_file).read_text(), timeout=60)
    show(raw)


def show(raw):
    events = analysers.load_events(raw.splitlines())
    n = 0
    for e in events:
        if e["event"] == "frame":
            p = e["parsed"]
            print(f"#{n} t={e['t_ms']} {e['screen']} load={int(e['loading'])} counts="
                  f"{e['mrs']}/{e['issues']}/{e['todos']}/{e['pipelines']} kind={p['kind']} "
                  f"title={p['title']!r} rows={len(p['rows'])} empty={p['empty']} sel={p['selected']!r} "
                  f"foot#={p['footer_count']} side={p['sidebar']}")
            n += 1
        elif e["event"] == "mark":
            print(f"   t={e['t_ms']} mark {e['detail']}")


def selected_runs(args):
    runs = []
    scen = args.scenarios.split(",") if args.scenarios else (
        [k for k, v in scenarios.SCENARIOS.items() if v[3]] if args.quick else list(scenarios.SCENARIOS))
    lats = [int(x) for x in (args.latencies or ("100" if args.quick else "0,100,300")).split(",")]
    fixtures = (args.fixtures or ("big" if args.quick else "small,big")).split(",")
    for key in scen:
        big_only = scenarios.SCENARIOS[key][2]
        for fixture in fixtures:
            if big_only and fixture != "big":
                continue
            for lat in lats:
                runs.append((key, lat, fixture == "big"))
    return runs


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--bin", default=str(ROOT / "target" / "release" / "lazygitlab"))
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--scenarios")
    ap.add_argument("--latencies")
    ap.add_argument("--fixtures")
    ap.add_argument("--out")
    ap.add_argument("--keep-raw")
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--report-only", action="store_true")
    ap.add_argument("--probe")
    ap.add_argument("--show", help="print a line per frame of a kept raw JSONL file")
    ap.add_argument("--latency", type=int, default=100)
    ap.add_argument("--big", action="store_true")
    args = ap.parse_args()
    if args.show:
        show(Path(args.show).read_text(encoding="utf-8"))
        return 0
    if args.probe:
        probe(args.bin, args.probe, args.latency, args.big)
        return 0
    if not Path(args.bin).exists():
        raise SystemExit(f"{args.bin} not found: run cargo build --release")
    runs = selected_runs(args)
    results = []

    def one(run):
        key, lat, big = run
        print(f"running {key} {'big' if big else 'small'} {lat} ms ...", file=sys.stderr, flush=True)
        return run_scenario(args.bin, key, lat, big, args.keep_raw)

    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        results = list(pool.map(one, runs))
    print(report.console(results))
    if args.out:
        out = Path(args.out)
        out.mkdir(parents=True, exist_ok=True)
        (out / "report.json").write_text(json.dumps(results, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
        (out / "summary.md").write_text(report.markdown(results), encoding="utf-8")
    total = sum(sum(r["counts"].values()) for r in results)
    return 0 if args.report_only or total == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
