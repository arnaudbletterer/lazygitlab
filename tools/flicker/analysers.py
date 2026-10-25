"""Find flicker defects in a recorded sequence of headless frames.

Input is the `--bench-out` JSONL (frame events, plus `mark:K|kind|target|desc`
marks written by scenarios.py before every key). `analyse` walks the events once
and returns a list of defects, each with the frame numbers, the recent keys and
the relevant lines of the frames involved.
"""
import json
from collections import deque
from dataclasses import dataclass, field

import frames

DEFECT_TYPES = ["ROWS_SHRINK", "POP_IN_OUT", "EMPTY_FLASH", "SPINNER_STUCK",
                "SELECTION_JUMP", "SCREEN_BLEED", "COUNT_MISMATCH", "STALE_AFTER_SWITCH",
                "EMPTY_WHILE_LOADING"]

LIST_SCREENS = {"MergeRequests": "mrs", "Issues": "issues", "Todos": "todos", "Pipelines": "pipelines"}
COUNT_KEYS = ("mrs", "issues", "todos")
EXPLAINING = ("filter", "refresh", "project")
MAX_EVIDENCE = 5


@dataclass
class Context:
    latency: int = 0
    samples: list = field(default_factory=list)
    mutations: list = field(default_factory=list)
    pop_frames: int = 5
    pop_ms: int = 2000
    stale_ms: int = 300
    spinner_ms: int = 10000

    def window_ms(self):
        return 1500 + 8 * self.latency

    def mutation_ms(self):
        return 40000 + 4 * self.latency


def load_events(lines):
    events, n = [], 0
    for line in lines:
        if not line.strip():
            continue
        e = json.loads(line)
        if e["event"] == "frame":
            e["n"] = n
            n += 1
            e["parsed"] = frames.parse_frame(e["frame"])
            e["state"] = frames.pane_state(e["parsed"])
        elif e["event"] == "mark" and e.get("detail", "").startswith("K|"):
            _, kind, target, desc = (e["detail"].split("|", 3) + ["", "", ""])[:4]
            e["key"] = {"kind": kind, "target": target, "desc": desc}
        events.append(e)
    return events


class Analyser:
    def __init__(self, ctx):
        self.ctx = ctx
        self.defects = []
        self.keys = deque(maxlen=12)
        self.keys_since_frame = []
        self.window_until = {}
        self.project_until = -1
        self.mem = {}
        self.prev_any = None
        self.prev_same = None
        self.appeared = {}
        self.pending_nav = None
        self.removals = [dict(t=t, mrs=b.get("merge_mr", 0), issues=b.get("close_issue", 0), used=0, any=True)
                         for t, b in ctx.mutations]
        self.last_screen = None
        self.loading_since = None
        self.loading_frame = None

    def add(self, typ, cur, detail, prev=None, **extra):
        count = sum(1 for d in self.defects if d["type"] == typ)
        d = {"type": typ, "t_ms": cur["t_ms"], "frame": cur["n"] if "n" in cur else None,
             "prev_frame": prev["n"] if prev else None, "screen": cur.get("screen"),
             "detail": detail,
             "keys": [f"{k['desc']} @{k['t']}ms" for k in list(self.keys)[-6:]]}
        d.update(extra)
        if count < MAX_EVIDENCE and "frame" in cur and "parsed" in cur:
            d["lines"] = {"cur": frames.evidence_lines(cur["frame"], cur["parsed"])}
            if prev:
                d["lines"]["prev"] = frames.evidence_lines(prev["frame"], prev["parsed"])
        self.defects.append(d)
        return d

    def explained(self, screen, t):
        return t <= self.window_until.get(screen, -1) or t <= self.project_until

    def mutation_recent(self, t):
        return any(m["t"] <= t <= m["t"] + self.ctx.mutation_ms() for m in self.removals)

    def consume_removal(self, lst, amount, t):
        for m in self.removals:
            if amount <= 0:
                break
            free = m[lst] - m["used"]
            if free > 0 and m["t"] <= t <= m["t"] + self.ctx.mutation_ms():
                take = min(free, amount)
                m["used"] += take
                amount -= take
        return amount

    def on_key(self, e):
        k = dict(e["key"], t=e["t_ms"])
        self.keys.append(k)
        self.keys_since_frame.append(k)
        t = e["t_ms"]
        if k["kind"] in EXPLAINING:
            if k["kind"] == "project":
                self.project_until = t + self.ctx.window_ms()
            else:
                self.window_until[k["target"]] = t + self.ctx.window_ms()
        if k["kind"] == "nav":
            self.finish_nav(t)
            screen = k["target"]
            if screen in LIST_SCREENS and screen != self.last_screen and self.mem.get(screen, {}).get("had_rows"):
                self.pending_nav = {"screen": screen, "t": t, "key": k, "frames": []}

    def finish_nav(self, t):
        p = self.pending_nav
        self.pending_nav = None
        if p and t - p["t"] >= self.ctx.stale_ms:
            self.defect_stale(p, t - p["t"], None)

    def defect_stale(self, p, delay, rows_frame):
        first = p["frames"][0] if p["frames"] else None
        cur = rows_frame or first or {"t_ms": p["t"], "screen": p["screen"]}
        self.add("STALE_AFTER_SWITCH", cur,
                 f"{p['screen']} had rows before; {'rows came back' if rows_frame else 'no rows'} "
                 f"{delay} ms after the switch (limit {self.ctx.stale_ms} ms)",
                 prev=None, delay_ms=delay, empty_frames=len(p["frames"]),
                 switch_frame=first["n"] if first else None)

    def on_frame(self, cur):
        t, screen, st = cur["t_ms"], cur["screen"], cur["state"]
        p = cur["parsed"]
        prev = self.prev_same if self.prev_same and self.prev_same["screen"] == screen else None
        keyed = bool(self.keys_since_frame)
        moved = any(k["kind"] in ("scroll", "nav", "drill", "back", "filter", "refresh", "project")
                    for k in self.keys_since_frame)
        explained = self.explained(screen, t)
        mem = self.mem.setdefault(screen, {"had_rows": False, "in_empty": False})
        self.check_bleed(cur, p)
        self.check_counts(cur, p)
        self.check_shrink(cur, prev, moved, explained, screen)
        if screen in LIST_SCREENS and p["kind"] == screen_kind(screen):
            self.check_empty_and_stale(cur, mem, st, explained, screen)
            if st != "none":
                self.check_pop(cur, prev, moved, explained, screen)
                self.check_selection(cur, prev, keyed, explained, screen)
        if screen in LIST_SCREENS and p["kind"] == screen:
            if explained:
                mem["had_rows"] = st == "rows"
            elif st == "rows":
                mem["had_rows"] = True

        self.prev_any = cur
        self.prev_same = cur
        self.last_screen = screen
        self.keys_since_frame = []
        self.last_loading(cur)

    def check_bleed(self, cur, p):
        screen = cur["screen"]
        want = screen_kind(screen)
        if not want or not p["kind"]:
            return
        wrong_title = p["kind"] != want
        sigil = frames.ID_SCREENS.get(screen)
        stray = [r for r in p["rows"] if sigil and r[0] in "!#" and r[0] != sigil]
        if wrong_title or stray:
            self.add("SCREEN_BLEED", cur,
                     f"screen {screen} draws a {p['kind']} pane" if wrong_title else
                     f"{len(stray)} row(s) with the wrong marker on {screen}: {stray[0]}",
                     rows=len(stray))

    def check_counts(self, cur, p):
        bad = []
        for key, shown in p["sidebar"].items():
            if shown != cur[key]:
                bad.append(f"sidebar {key} shows {shown}, app holds {cur[key]}")
        screen = cur["screen"]
        key = LIST_SCREENS.get(screen)
        n = p["footer_count"]
        if (key in ("mrs", "issues") and p["kind"] == screen_kind(screen) and n is not None
                and not self.explained(screen, cur["t_ms"])):
            held = cur[key]
            visible = len(p["rows"]) + len(p["groups"])
            if n < held:
                bad.append(f"footer counts {n} but app holds {held} {key}")
            elif n > held + 200:
                bad.append(f"footer counts {n} for {held} {key}")
            if n <= p["capacity"] and visible != n:
                bad.append(f"footer counts {n} but {visible} lines are visible")
        if bad:
            self.add("COUNT_MISMATCH", cur, "; ".join(bad))

    def check_shrink(self, cur, prev, moved, explained, screen):
        t = cur["t_ms"]
        before = self.prev_any
        counted = False
        if before:
            for lst in COUNT_KEYS:
                drop = before[lst] - cur[lst]
                owner = next((s for s, k in LIST_SCREENS.items() if k == lst), None)
                if drop > 0 and not self.explained(owner, t):
                    left = self.consume_removal(lst, drop, t)
                    if left > 0:
                        self.add("ROWS_SHRINK", cur, f"{lst} held by the app fell from {before[lst]} to {cur[lst]}",
                                 prev=before, list=lst, before=before[lst], after=cur[lst])
                        if owner == screen:
                            counted = True
        if (not counted and prev and prev["state"] == "rows" and cur["state"] in ("rows", "empty", "blank")
                and not moved and not explained and not self.mutation_recent(t)):
            gone = [r for r in prev["parsed"]["rows"] if r not in cur["parsed"]["rows"]]
            shrank = len(cur["parsed"]["rows"]) < len(prev["parsed"]["rows"])
            if shrank and gone and not all(r in self.appeared for r in gone):
                self.add("ROWS_SHRINK", cur,
                         f"visible rows fell from {len(prev['parsed']['rows'])} to {len(cur['parsed']['rows'])} "
                         f"on {screen}, {len(gone)} gone",
                         prev=prev, before=len(prev["parsed"]["rows"]), after=len(cur["parsed"]["rows"]),
                         gone=gone[:5])

    def check_empty_and_stale(self, cur, mem, st, explained, screen):
        if st in ("empty", "blank"):
            if self.pending_nav and self.pending_nav["screen"] == screen:
                self.pending_nav["frames"].append(cur)
            if mem["had_rows"] and not mem["in_empty"] and not explained:
                d = self.add("EMPTY_FLASH", cur, f"{screen} had rows and now shows its {st} state",
                             prev=self.prev_any, state=st, rows_returned=False)
                mem["flash"] = d
            elif st == "empty" and cur["loading"] and not mem["had_rows"] and not mem["in_empty"] and not explained:
                self.add("EMPTY_WHILE_LOADING", cur,
                         f"{screen} shows its no-match message while loading, before any rows",
                         prev=self.prev_any)
            mem["in_empty"] = True
        elif st == "rows":
            if mem["in_empty"] and mem.get("flash") is not None:
                mem["flash"]["rows_returned"] = True
                mem["flash"]["empty_ms"] = cur["t_ms"] - mem["flash"]["t_ms"]
            mem["in_empty"] = False
            mem["flash"] = None
            p = self.pending_nav
            if p and p["screen"] == screen:
                self.pending_nav = None
                delay = cur["t_ms"] - p["t"]
                if delay > self.ctx.stale_ms:
                    self.defect_stale(p, delay, cur)

    def check_pop(self, cur, prev, moved, explained, screen):
        if screen not in frames.ID_SCREENS:
            return
        t = cur["t_ms"]
        now = set(cur["parsed"]["rows"])
        before = list(prev["parsed"]["rows"]) if prev and prev["state"] == "rows" else []
        grew = prev and cur[LIST_SCREENS[screen]] > prev[LIST_SCREENS[screen]]
        if prev and not moved and not explained and not grew and not self.mutation_recent(t):
            vanished = []
            for rid in before:
                if rid in now:
                    continue
                a = self.appeared.get(rid)
                if not a:
                    continue
                if cur["n"] - a["n"] <= self.ctx.pop_frames or t - a["t_ms"] <= self.ctx.pop_ms:
                    pushed = (before.index(rid) >= len(before) - 3 and len(now) >= len(before))
                    if not pushed:
                        vanished.append((rid, a))
            if vanished:
                a0 = vanished[0][1]
                self.add("POP_IN_OUT", cur,
                         f"{len(vanished)} row(s) appeared and vanished again within "
                         f"{cur['n'] - a0['n']} frames / {t - a0['t_ms']} ms: {vanished[0][0]}",
                         prev=prev, appeared_frame=a0["n"], rows=[v[0] for v in vanished[:5]],
                         lifetime_ms=t - a0["t_ms"])
        for rid in list(self.appeared):
            if rid not in now:
                self.appeared.pop(rid)
        if prev:
            for rid in now - set(before):
                self.appeared[rid] = {"n": cur["n"], "t_ms": t}

    def check_selection(self, cur, prev, keyed, explained, screen):
        if screen not in frames.ID_SCREENS:
            return
        if not prev or keyed or explained or self.mutation_recent(cur["t_ms"]):
            return
        a, b = prev["parsed"]["selected"], cur["parsed"]["selected"]
        if a and b and a != b:
            self.add("SELECTION_JUMP", cur, f"selection moved from {a!r} to {b!r} with no key", prev=prev,
                     before=a, after=b)

    def last_loading(self, cur):
        if cur["loading"]:
            if self.loading_since is None:
                self.loading_since = cur["t_ms"]
                self.loading_frame = cur
        else:
            self.close_loading(cur["t_ms"])

    def close_loading(self, end):
        start, self.loading_since = self.loading_since, None
        if start is None or end - start < self.ctx.spinner_ms:
            return
        for lo, hi in idle_runs(self.ctx.samples):
            lo, hi = max(lo, start), min(hi, end)
            if hi - lo >= self.ctx.spinner_ms:
                self.add("SPINNER_STUCK", self.loading_frame,
                         f"loading indicator on from {start} to {end} ms with no request outstanding "
                         f"from {lo} to {hi} ms", loading_ms=end - start, idle_ms=hi - lo)
                return

    def finish(self, end_t):
        self.finish_nav(end_t)
        self.close_loading(end_t)


def idle_runs(samples):
    """Spans (start, end) of consecutive samples with no request in flight."""
    runs, lo, hi = [], None, None
    for t, inflight, _ in samples:
        if inflight == 0:
            lo = t if lo is None else lo
            hi = t
        elif lo is not None:
            runs.append((lo, hi))
            lo = None
    if lo is not None:
        runs.append((lo, hi))
    return runs


def screen_kind(screen):
    return screen if screen in LIST_SCREENS else None


def analyse(events, ctx):
    a = Analyser(ctx)
    end_t = 0
    for e in events:
        end_t = max(end_t, e.get("t_ms", 0))
        if e["event"] == "mark" and "key" in e:
            a.on_key(e)
        elif e["event"] == "frame":
            a.on_frame(e)
    a.finish(end_t)
    return a.defects
