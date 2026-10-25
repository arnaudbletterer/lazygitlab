"""Unit tests for the flicker analysers, on synthetic frame sequences.

Run: python3 -I -m unittest discover -s tools/flicker
"""
import json
import unittest

import analysers
import frames
import report

TITLES = {"MergeRequests": "Merge Requests", "Issues": "Issues", "Todos": "To-Do List",
          "Pipelines": "Pipelines", "Home": "Home"}
SIGIL = {"MergeRequests": "!", "Issues": "#"}


def render(screen, rows=(), sel=None, empty=False, title=None, footer=None, sidebar="", group="acme/p"):
    """Draw a 150 x 40 frame like the headless runner does."""
    inner = []
    if empty:
        inner.append("No items match this view or filter.")
    elif rows:
        inner.append(f"▎ ▼ {group}" if sel == "group" else f"  ▼ {group}")
        for r in rows:
            mark = "▎ " if r == sel else "  "
            inner.append(f"{mark}✦ {SIGIL.get(screen, '!')}{r} Title {r}  label")
    lines = ["🦊  GitLab  ❯ Personal Dashboard  ● LIVE".ljust(150)]
    pane = ["╭ " + (title or TITLES[screen]) + " " + "─" * 100 + "╮"]
    pane += ["│" + c.ljust(116)[:116] + "│" for c in inner]
    pane = pane[:37] + ["│" + " " * 116 + "│"] * max(0, 37 - len(pane)) + ["╰" + "─" * 116 + "╯"]
    side = [sidebar.ljust(30)] * 38
    for i in range(38):
        lines.append(side[i] + (pane[i] if i < len(pane) else ""))
    lines.append((footer or f" LIST   [{len(rows) + 1 if rows else 0}]  ? help").ljust(150))
    return "\n".join(lines) + "\n"


def frame(t, screen, rows=(), loading=False, mrs=0, issues=0, todos=0, **kw):
    return {"event": "frame", "t_ms": t, "screen": screen, "loading": loading, "mrs": mrs,
            "issues": issues, "todos": todos, "pipelines": 0, "requests": 0, "bytes": 0,
            "frame": render(screen, rows, **kw)}


def key(t, kind, target, desc):
    return {"event": "mark", "t_ms": t, "detail": f"K|{kind}|{target}|{desc}", "ok": True}


def run(events, **ctx):
    lines = [json.dumps(e) for e in events] + [json.dumps({"event": "end", "t_ms": events[-1]["t_ms"] + 5000})]
    return analysers.analyse(analysers.load_events(lines), analysers.Context(**ctx))


def types(defects):
    return [d["type"] for d in defects]


class ParseTest(unittest.TestCase):
    def test_rows_selection_and_footer(self):
        p = frames.parse_frame(render("MergeRequests", [1, 2, 3], sel=2))
        self.assertEqual(p["kind"], "MergeRequests")
        self.assertEqual(p["rows"], ["!1 Title 1", "!2 Title 2", "!3 Title 3"])
        self.assertEqual(p["selected"], "!2 Title 2")
        self.assertEqual(p["footer_count"], 4)
        self.assertEqual(p["groups"], ["acme/p"])

    def test_empty_state(self):
        p = frames.parse_frame(render("Issues", empty=True))
        self.assertTrue(p["empty"])
        self.assertEqual(frames.pane_state(p), "empty")

    def test_home_is_not_a_list(self):
        self.assertEqual(frames.pane_state(frames.parse_frame(render("Home"))), "none")

    def test_sidebar_counts(self):
        p = frames.parse_frame(render("Issues", [1], sidebar="│    › Merge Requests (12)  │"))
        self.assertEqual(p["sidebar"], {"mrs": 12})


class ShrinkTest(unittest.TestCase):
    def test_count_drop_without_action(self):
        ev = [frame(0, "MergeRequests", [1, 2], mrs=2), frame(500, "MergeRequests", [1, 2], mrs=0)]
        self.assertIn("ROWS_SHRINK", types(run(ev)))

    def test_count_drop_explained_by_filter(self):
        ev = [frame(0, "MergeRequests", [1, 2], mrs=2), key(400, "filter", "MergeRequests", "]"),
              frame(500, "MergeRequests", [1, 2], mrs=0)]
        self.assertEqual(types(run(ev)), [])

    def test_count_drop_of_another_list_is_not_explained(self):
        ev = [frame(0, "MergeRequests", [1], mrs=1, todos=50), key(400, "filter", "MergeRequests", "]"),
              frame(500, "MergeRequests", [1], mrs=1, todos=0)]
        self.assertEqual(types(run(ev)), ["ROWS_SHRINK"])

    def test_visible_rows_fall(self):
        ev = [frame(0, "Issues", [1, 2, 3], issues=3), frame(300, "Issues", [1, 2], issues=3)]
        self.assertEqual(types(run(ev)), ["ROWS_SHRINK"])

    def test_merge_by_the_mock_explains_the_drop(self):
        ev = [frame(0, "MergeRequests", [1, 2, 3], mrs=3), frame(8000, "MergeRequests", [1, 2], mrs=2)]
        self.assertEqual(types(run(ev, mutations=[(7000, {"merge_mr": 1})])), [])
        self.assertEqual(types(run(ev, mutations=[])), ["ROWS_SHRINK"])

    def test_scrolling_is_not_a_shrink(self):
        ev = [frame(0, "Issues", [1, 2, 3], issues=3), key(100, "scroll", "Issues", "j"),
              frame(150, "Issues", [2, 3], issues=3)]
        self.assertEqual(types(run(ev)), [])


class PopTest(unittest.TestCase):
    def test_row_appears_and_vanishes(self):
        ev = [frame(0, "MergeRequests", [1, 2], mrs=2), frame(100, "MergeRequests", [1, 2, 3], mrs=2),
              frame(200, "MergeRequests", [1, 2], mrs=2)]
        self.assertEqual(types(run(ev)), ["POP_IN_OUT"])

    def test_row_that_stays_is_fine(self):
        ev = [frame(0, "MergeRequests", [1, 2], mrs=2), frame(100, "MergeRequests", [1, 2, 3], mrs=3),
              frame(200, "MergeRequests", [1, 2, 3], mrs=3)]
        self.assertEqual(types(run(ev)), [])

    def test_old_row_vanishing_is_a_shrink_not_a_pop(self):
        ev = [frame(0, "MergeRequests", [1, 2], mrs=2), frame(5000, "MergeRequests", [1, 2], mrs=2, loading=True),
              frame(9000, "MergeRequests", [1], mrs=1)]
        self.assertNotIn("POP_IN_OUT", types(run(ev)))

    def test_displacement_by_a_growing_list_is_not_a_pop(self):
        ev = [frame(0, "MergeRequests", [5, 6], mrs=2), frame(100, "MergeRequests", [1, 2, 3, 5], mrs=4),
              frame(200, "MergeRequests", [1, 2, 3], mrs=6)]
        self.assertNotIn("POP_IN_OUT", types(run(ev)))


class EmptyTest(unittest.TestCase):
    def test_flash_then_rows_return(self):
        ev = [frame(0, "Issues", [1, 2], issues=2), frame(300, "Issues", empty=True, issues=2),
              frame(600, "Issues", [1, 2], issues=2)]
        out = run(ev)
        flashes = [d for d in out if d["type"] == "EMPTY_FLASH"]
        self.assertEqual(len(flashes), 1)
        self.assertTrue(flashes[0]["rows_returned"])
        self.assertEqual(flashes[0]["empty_ms"], 300)

    def test_empty_after_filter_is_expected(self):
        ev = [frame(0, "Issues", [1, 2], issues=2), key(100, "filter", "Issues", "]"),
              frame(300, "Issues", empty=True, issues=0)]
        self.assertEqual(types(run(ev)), [])

    def test_empty_before_first_rows_while_loading(self):
        ev = [frame(0, "Issues", empty=True, loading=True), frame(300, "Issues", [1], issues=1)]
        self.assertEqual(types(run(ev)), ["EMPTY_WHILE_LOADING"])

    def test_flash_when_coming_back_to_a_screen(self):
        ev = [frame(0, "Issues", [1, 2], issues=2), key(100, "nav", "MergeRequests", "g m"),
              frame(101, "MergeRequests", [1], mrs=1), key(200, "nav", "Issues", "g i"),
              frame(201, "Issues", empty=True, issues=2)]
        self.assertIn("EMPTY_FLASH", types(run(ev)))


class SpinnerTest(unittest.TestCase):
    EV = [frame(0, "Issues", [1], loading=True, issues=1), frame(15000, "Issues", [1], loading=False, issues=1)]

    def test_stuck_without_requests(self):
        samples = [(t, 0, t) for t in range(0, 15000, 200)]
        self.assertEqual(types(run(self.EV, samples=samples)), ["SPINNER_STUCK"])

    def test_slow_but_requests_outstanding(self):
        samples = [(t, 1, t) for t in range(0, 15000, 200)]
        self.assertEqual(types(run(self.EV, samples=samples)), [])

    def test_short_loading_is_fine(self):
        ev = [frame(0, "Issues", [1], loading=True, issues=1), frame(3000, "Issues", [1], issues=1)]
        self.assertEqual(types(run(ev, samples=[(t, 0, 0) for t in range(0, 3000, 200)])), [])


class SelectionTest(unittest.TestCase):
    def test_jump_without_key(self):
        ev = [frame(0, "Issues", [1, 2, 3], issues=3, sel=1), frame(300, "Issues", [1, 2, 3], issues=3, sel=3)]
        self.assertEqual(types(run(ev)), ["SELECTION_JUMP"])

    def test_move_with_key(self):
        ev = [frame(0, "Issues", [1, 2, 3], issues=3, sel=1), key(100, "scroll", "Issues", "j"),
              frame(150, "Issues", [1, 2, 3], issues=3, sel=2)]
        self.assertEqual(types(run(ev)), [])


class BleedCountTest(unittest.TestCase):
    def test_issue_rows_on_the_mr_screen(self):
        bad = frame(0, "MergeRequests", [1, 2], mrs=2)
        bad["frame"] = bad["frame"].replace("!1 ", "#1 ")
        self.assertEqual(types(run([bad])), ["SCREEN_BLEED"])

    def test_wrong_pane_title(self):
        ev = [frame(0, "Issues", [1], issues=1, title="Merge Requests")]
        self.assertEqual(types(run(ev)), ["SCREEN_BLEED"])

    def test_sidebar_count_differs_from_app(self):
        ev = [frame(0, "Issues", [1], issues=1, mrs=7, sidebar="│    › Merge Requests (9)  │")]
        self.assertEqual(types(run(ev)), ["COUNT_MISMATCH"])

    def test_footer_below_held_rows(self):
        ev = [frame(0, "MergeRequests", [1, 2], mrs=50, footer=" LIST   [3]  ? help")]
        self.assertEqual(types(run(ev)), ["COUNT_MISMATCH"])

    def test_consistent_counts(self):
        ev = [frame(0, "MergeRequests", [1, 2], mrs=2, sidebar="│    › Merge Requests (2)  │")]
        self.assertEqual(types(run(ev)), [])


class StaleTest(unittest.TestCase):
    def away_and_back(self, back_rows_at):
        ev = [frame(0, "Issues", [1, 2], issues=2), key(100, "nav", "MergeRequests", "g m"),
              frame(101, "MergeRequests", [1], mrs=1, issues=2), key(1000, "nav", "Issues", "g i"),
              frame(1001, "Issues", empty=True, issues=2, mrs=1)]
        if back_rows_at:
            ev.append(frame(back_rows_at, "Issues", [1, 2], issues=2, mrs=1))
        return ev

    def test_slow_return(self):
        out = [d for d in run(self.away_and_back(1800)) if d["type"] == "STALE_AFTER_SWITCH"]
        self.assertEqual(len(out), 1)
        self.assertEqual(out[0]["delay_ms"], 800)

    def test_fast_return(self):
        ev = [frame(0, "Issues", [1, 2], issues=2), key(100, "nav", "MergeRequests", "g m"),
              frame(101, "MergeRequests", [1], mrs=1, issues=2), key(1000, "nav", "Issues", "g i"),
              frame(1001, "Issues", [1, 2], issues=2, mrs=1)]
        self.assertEqual(types(run(ev)), [])

    def test_never_returns(self):
        out = [d for d in run(self.away_and_back(0)) if d["type"] == "STALE_AFTER_SWITCH"]
        self.assertEqual(len(out), 1)

    def test_first_visit_is_not_stale(self):
        ev = [key(0, "nav", "Issues", "g i"), frame(1, "Issues", empty=True), frame(900, "Issues", [1], issues=1)]
        self.assertNotIn("STALE_AFTER_SWITCH", types(run(ev)))


class ReportTest(unittest.TestCase):
    def test_markdown_lists_unreproduced_defects(self):
        res = [{"scenario": "S1", "fixture": "big", "latency_ms": 0, "frames": 3,
                "defects": [], "counts": {t: 0 for t in analysers.DEFECT_TYPES}}]
        md = report.markdown(res)
        self.assertIn("did not reproduce", md)
        self.assertIn("Not reproduced in any run.", md)


if __name__ == "__main__":
    unittest.main()
