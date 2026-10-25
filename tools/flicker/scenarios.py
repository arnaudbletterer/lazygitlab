"""Scripted navigation scenarios for the flicker detector.

A scenario builds a headless script (one key per line, `wait:MS`, and one
`mark:K|kind|target|desc` line before every key so the analysers know what the
user did and when) plus optional timed commands for the mock server.

Kinds: nav (target = screen reached), scroll, filter, refresh, project, drill,
back, other. `filter`, `refresh` and `project` explain a list change on
`target`; the others do not.
"""

MR, ISSUES, TODOS, PIPES = "MergeRequests", "Issues", "Todos", "Pipelines"
NAV_KEYS = {MR: "m", ISSUES: "i", TODOS: "t", PIPES: "p"}


class Script:
    def __init__(self, latency):
        self.latency = latency
        self.lines = []
        self.timeline = []

    def settle(self):
        self.wait(2000 + 6 * self.latency)
        return self

    def wait(self, ms):
        self.lines.append(f"wait:{int(ms)}")
        return self

    def key(self, kind, target, desc, *keys):
        self.lines.append(f"mark:K|{kind}|{target}|{desc}")
        self.lines.extend(keys)
        return self

    def goto(self, screen):
        return self.key("nav", screen, f"g {NAV_KEYS[screen]}", "g", NAV_KEYS[screen])

    def scroll(self, screen, n=1, gap=120):
        for _ in range(n):
            self.key("scroll", screen, "j", "j").wait(gap)
        return self

    def filter(self, screen, direction):
        return self.key("filter", screen, direction, direction)

    def at(self, ms, path, body):
        self.timeline.append({"at_ms": ms, "path": path, "body": body})
        return self

    def text(self):
        return "\n".join(self.lines + ["esc", "esc", "g", "h", "q", ""])


def s1(lat):
    s = Script(lat).settle()
    for scr in (MR, ISSUES, MR, ISSUES):
        s.goto(scr).wait(1500)
    return s.wait(1500)


def s2(lat):
    s = Script(lat).settle()
    for k in range(20):
        s.goto(MR if k % 2 == 0 else ISSUES).wait(150)
    return s.wait(3000)


def s3(lat):
    s = Script(lat).settle()
    for k in range(8):
        s.goto(MR if k % 2 == 0 else ISSUES).wait(200).scroll(MR if k % 2 == 0 else ISSUES, 6)
    return s.wait(2000)


def s4(lat):
    s = Script(lat).settle()
    gap = 1500 + 3 * lat
    s.goto(MR).wait(gap)
    for _ in range(5):
        s.filter(MR, "]").wait(gap)
    for _ in range(5):
        s.filter(MR, "[").wait(gap)
    s.goto(ISSUES).wait(gap)
    for _ in range(2):
        s.filter(ISSUES, "]").wait(gap)
    for _ in range(2):
        s.filter(ISSUES, "[").wait(gap)
    return s


def s5(lat):
    s = Script(lat).settle()
    gap = 1000 + 3 * lat
    for scr in (MR, ISSUES):
        s.goto(scr).wait(gap).scroll(scr, 2)
        for _ in range(5):
            s.key("drill", scr, "Enter", "enter").wait(gap)
            s.key("back", scr, "Esc", "esc").wait(gap)
    return s


def s6(lat):
    s = Script(lat).settle()
    gap = 1200 + 4 * lat
    s.goto(MR).wait(gap)
    s.goto(PIPES).wait(gap)
    s.goto(TODOS).wait(gap)
    s.goto(MR).wait(gap)
    s.key("nav", "Projects", "g h, Tab, 5 x j", "g", "h", "tab", "j", "j", "j", "j", "j").wait(gap)
    s.key("nav", "Groups", "j", "j").wait(gap)
    s.key("nav", "Home", "g h", "g", "h").wait(gap)
    s.goto(MR).wait(gap)
    s.goto(ISSUES).wait(gap)
    return s.goto(MR).wait(gap)


def s7(lat):
    s = Script(lat).settle().goto(MR)
    for at, body in ((12, {"merge_mr": 1}), (27, {"add_mr": 1}), (42, {"merge_mr": 2, "close_issue": 1}),
                     (57, {"add_mr": 2, "add_issue": 1}), (72, {"merge_mr": 1})):
        s.at(at * 1000, "/__mutate", body)
    return s.wait(90000)


def s8(lat):
    s = Script(lat).settle()
    gap = 1200 + 2 * lat
    s.goto(MR).wait(gap)
    s.at(6000 + 6 * lat, "/__config", {"fail_projects": [1002, 1005], "hang_projects": [1003]})
    s.at(26000 + 6 * lat, "/__config", {"fail_projects": [], "hang_projects": []})
    for k in range(24):
        s.goto(ISSUES if k % 2 == 0 else MR).wait(gap)
    return s.wait(3000)


def s9(lat):
    s = Script(lat).settle()
    gap = 1500 + 3 * lat
    s.key("nav", "Home", "g h, Tab", "g", "h", "tab").wait(300)
    s.key("nav", MR, "j j (sidebar)", "j", "j").wait(gap)
    for _ in range(4):
        s.key("nav", ISSUES, "j (sidebar)", "j").wait(gap)
        s.key("nav", MR, "k (sidebar)", "k").wait(gap)
    return s


# name: (builder, description, big_only, quick)
SCENARIOS = {
    "S1": (s1, "g m, g i, g m, g i every 1.5 s", False, True),
    "S2": (s2, "20 fast switches every 150 ms", False, True),
    "S3": (s3, "m/i switching while scrolling with j", False, False),
    "S4": (s4, "scope and state filter A to B to A", False, True),
    "S5": (s5, "Enter into an item and Esc back, repeated", False, False),
    "S6": (s6, "Pipelines, Todos, Projects, Groups and back", False, False),
    "S7": (s7, "idle 90 s while the mock merges, closes and adds", False, False),
    "S9": (s9, "switch MRs and Issues through the sidebar (Tab, j, k)", False, True),
    "S8": (s8, "mock fails or hangs some projects while switching", True, False),
}
