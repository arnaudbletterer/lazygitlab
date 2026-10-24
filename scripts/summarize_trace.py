#!/usr/bin/env python3
"""Summarize LAZYGITLAB_TRACE=1 request timings from a lazygitlab log.

Usage:
    scripts/summarize_trace.py [--extract] [LOGFILE ...]

Reads the given files (or stdin) and prints, per endpoint template, the number
of requests, p50 and p95 duration in ms, total time in ms, response bytes and
the number of 304 answers. Lines that are not trace records are ignored.

--extract prints only the trace records, one JSON object per line, so that a
file safe to share can be produced with:
    scripts/summarize_trace.py --extract ~/.local/state/lazygitlab/lazygitlab.log > trace.jsonl
"""
import argparse
import json
import sys

MARKER = '{"trace":1'


def records(lines):
    for line in lines:
        i = line.find(MARKER)
        if i < 0:
            continue
        try:
            rec = json.loads(line[i:])
        except ValueError:
            continue
        if isinstance(rec, dict) and rec.get("trace") == 1:
            yield rec


def percentile(sorted_values, q):
    if not sorted_values:
        return 0.0
    rank = max(1, -(-len(sorted_values) * q // 100))
    return sorted_values[int(rank) - 1]


def summarize(recs):
    groups = {}
    for r in recs:
        key = (r.get("method", "?"), r.get("endpoint", "?"))
        groups.setdefault(key, []).append(r)
    rows = []
    for (method, endpoint), items in groups.items():
        ms = sorted(float(r.get("ms", 0)) for r in items)
        rows.append({
            "endpoint": f"{method} {endpoint}",
            "count": len(items),
            "p50": percentile(ms, 50),
            "p95": percentile(ms, 95),
            "total": sum(ms),
            "bytes": sum(int(r.get("bytes", 0)) for r in items),
            "n304": sum(1 for r in items if r.get("etag_304")),
        })
    rows.sort(key=lambda r: r["total"], reverse=True)
    return rows


def render(rows):
    width = max([len(r["endpoint"]) for r in rows] + [8])
    out = [f'{"endpoint":<{width}}  {"count":>6}  {"p50 ms":>8}  {"p95 ms":>8}  '
           f'{"total ms":>10}  {"bytes":>10}  {"304":>4}']
    for r in rows:
        out.append(f'{r["endpoint"]:<{width}}  {r["count"]:>6}  {r["p50"]:>8.1f}  '
                   f'{r["p95"]:>8.1f}  {r["total"]:>10.1f}  {r["bytes"]:>10}  {r["n304"]:>4}')
    return "\n".join(out)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("files", nargs="*")
    ap.add_argument("--extract", action="store_true",
                    help="print only the trace records as JSON lines")
    args = ap.parse_args(argv)

    def lines():
        if not args.files:
            yield from sys.stdin
        for name in args.files:
            with open(name, encoding="utf-8", errors="replace") as fh:
                yield from fh

    recs = list(records(lines()))
    if args.extract:
        for r in recs:
            print(json.dumps(r, separators=(",", ":")))
        return 0
    if not recs:
        print("no trace records found (run lazygitlab with LAZYGITLAB_TRACE=1)",
              file=sys.stderr)
        return 1
    print(render(summarize(recs)))
    print(f"\n{len(recs)} requests, {sum(float(r.get('ms', 0)) for r in recs):.1f} ms "
          f"cumulative request time")
    return 0


if __name__ == "__main__":
    sys.exit(main())
