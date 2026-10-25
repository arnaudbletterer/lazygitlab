"""Turn a headless text frame into the facts the analysers compare.

A frame is the 150 x 40 cell dump written by `--bench-out`. The left 30 columns
hold the sidebar, the rest the main pane. Only merge request and issue rows
carry stable ids (`!12 title`, `#7 title`); other screens are judged by row
counts and by the empty-state message.
"""
import re

PANE_X = 30

LIST_KINDS = (
    ("MergeRequests", re.compile(r"Merge Requests")),
    ("Issues", re.compile(r"Issues")),
    ("Todos", re.compile(r"To-?[Dd]o")),
    ("Pipelines", re.compile(r"Pipelines")),
)
ID_SCREENS = {"MergeRequests": "!", "Issues": "#"}

ROW_RE = re.compile(r"([!#])(\d+) (.+?)(?:\s{2,}|$)")
GROUP_RE = re.compile(r"^[▼▶] (.+?)\s*$")
EMPTY_RE = re.compile(r"^No .* match")
TAB_WORDS = ("Opened", "Closed", "All", "Pending", "Done", "Snoozed", "Review")
FOOTER_COUNT_RE = re.compile(r"\[(\d+)\]")
SIDEBAR_COUNT_RE = re.compile(r"›\s+(To-Dos|Merge Requests|Issues)\s+\((\d+)\)")
SIDEBAR_KEYS = {"To-Dos": "todos", "Merge Requests": "mrs", "Issues": "issues"}


def parse_frame(text):
    """Return a dict describing the frame, see the keys below."""
    lines = text.split("\n")
    while lines and lines[-1] == "":
        lines.pop()
    header = lines[0] if lines else ""
    footer = lines[-1] if lines else ""
    pane = [ln[PANE_X:] for ln in lines[1:-1]]
    title = ""
    inside = []
    started = False
    for ln in pane:
        if not started:
            if ln.startswith("╭"):
                title = re.sub(r"[─╮╭]", "", ln).strip()
                started = True
            continue
        if ln.startswith("╰"):
            break
        if ln.startswith("│"):
            inside.append(ln[1:].rstrip("│").rstrip())
    kind = next((k for k, rx in LIST_KINDS if rx.search(title)), None)
    rows, groups, selected, empty, sel_line = [], [], None, False, None
    if kind:
        for content in inside:
            sel = content.startswith("▎")
            body = content.lstrip("▎ ").rstrip()
            if not body:
                continue
            if EMPTY_RE.match(body):
                empty = True
                continue
            g = GROUP_RE.match(body)
            if g:
                groups.append(g.group(1))
                if sel:
                    selected, sel_line = "group:" + g.group(1), body
                continue
            m = ROW_RE.search(body)
            if m:
                rid = f"{m.group(1)}{m.group(2)} {m.group(3)}"
                rows.append(rid)
                if sel:
                    selected, sel_line = rid, body
            elif kind not in ID_SCREENS and not _is_tabs(body):
                rows.append(re.sub(r"\s+", " ", body))
                if sel:
                    selected, sel_line = rows[-1], body
    if empty and kind not in ID_SCREENS:
        rows, selected, sel_line = [], None, None
    fc = FOOTER_COUNT_RE.search(footer)
    sidebar = {}
    for ln in lines[1:-1]:
        m = SIDEBAR_COUNT_RE.search(ln[:PANE_X])
        if m:
            sidebar[SIDEBAR_KEYS[m.group(1)]] = int(m.group(2))
    return {
        "header": header.strip(),
        "footer": footer.strip(),
        "title": title,
        "kind": kind,
        "rows": rows,
        "groups": groups,
        "selected": selected,
        "selected_line": sel_line,
        "empty": empty,
        "capacity": len(inside),
        "footer_count": int(fc.group(1)) if fc else None,
        "sidebar": sidebar,
        "loading_badge": "LOADING" in header,
    }


def _is_tabs(body):
    return sum(w in body for w in TAB_WORDS) >= 2 and "│" in body


def pane_state(parsed):
    """rows, empty, blank (list pane without rows or message) or none."""
    if not parsed["kind"]:
        return "none"
    if parsed["rows"]:
        return "rows"
    if parsed["empty"]:
        return "empty"
    return "blank"


def evidence_lines(text, parsed, limit_rows=3):
    """The few lines of a frame worth showing next to a defect."""
    out = [parsed["header"], parsed["title"] and f"[pane] {parsed['title']}"]
    out += [f"[row] {r}" for r in parsed["rows"][:limit_rows]]
    if parsed["rows"]:
        out.append(f"[rows visible] {len(parsed['rows'])}")
    if parsed["empty"]:
        out.append("[pane] empty-state message shown")
    if parsed["selected"]:
        out.append(f"[selected] {parsed['selected']}")
    out.append(parsed["footer"])
    return [ln for ln in out if ln]
