#!/usr/bin/env python3
"""Fictional GitLab REST v4 server for demo screenshots. Stdlib only.

Serves the endpoints lazygitlab calls from the JSON fixtures in ./fixtures.
Set MOCK_DEBUG=1 to log unhandled routes on stderr.

Usage: mock_gitlab.py [PORT] [--latency-ms N] [--big] [--projects N] [--mrs N]
                      [--issues N] [--todos N] [--pipelines N]

--latency-ms N  sleep N ms before answering every request (not /__stats).
--big           replace the fictional world by a generated one (default 40
                projects, 500 merge requests, 500 issues, skewed so that a few
                projects hold most of them). Nothing is stored on disk.
GET /__stats    JSON request counters per method and path template.
POST /__reset   zero the counters.

List endpoints that receive page or per_page (or any list in --big mode) are
paginated like GitLab: X-Page, X-Per-Page, X-Next-Page, X-Prev-Page, X-Total,
X-Total-Pages and Link headers.
"""
import argparse
import hashlib
import http.server
import json
import os
import re
import sys
import threading
import time
import zlib
import urllib.parse
from datetime import datetime, timedelta, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fixtures")
HOST = "https://gitlab.example.com"
ME = "ada"
PID = 42
DEBUG = bool(os.environ.get("MOCK_DEBUG"))
LATENCY_MS = 0.0
BIG = False
PROJECT_PATHS = {}


def load(name):
    with open(os.path.join(FIX, name), encoding="utf-8") as fh:
        return json.load(fh)


def text(path):
    with open(os.path.join(FIX, path), encoding="utf-8") as fh:
        return fh.read()


USERS = load("users.json")
LABELS = {lb["name"]: lb for lb in load("labels.json")}
PROJ = load("projects.json")
MRS = load("mrs.json")
ISSUES = load("issues.json")
TODOS = load("todos.json")
PIPES = load("pipelines.json")
DISCUSSIONS = load("discussions.json")
DIFFS = load("diffs.json")
BOARD = load("board.json")

PROJECT_IDS = set()

UNITS = {"m": 60, "h": 3600, "d": 86400, "w": 604800}


def ago(spec):
    n, unit = int(spec[:-1]), spec[-1]
    t = datetime.now(timezone.utc) - timedelta(seconds=n * UNITS[unit])
    return t.strftime("%Y-%m-%dT%H:%M:%S.000Z")


def user(name):
    u = dict(USERS[name])
    u.update(state="active", avatar_url="", web_url=f"{HOST}/{name}")
    return u


def label(name):
    return dict(LABELS[name])


def ppath(pid):
    return PROJECT_PATHS.get(pid, "acme/widgets")


def project(p):
    return {
        "id": p["id"], "name": p["name"], "path_with_namespace": p["path"],
        "web_url": f"{HOST}/{p['path']}", "default_branch": p["default_branch"],
        "last_activity_at": ago(p["age"]),
        "namespace": {"id": 7, "name": "Acme", "path": "acme", "kind": "group",
                      "full_path": "acme"},
    }


def mr(m):
    iid = m["iid"]
    pid = m.get("project", PID)
    out = {
        "id": 1000 + m.get("gid", iid), "iid": iid, "project_id": pid, "title": m["title"],
        "description": m.get("description", ""), "state": m["state"],
        "draft": bool(m.get("draft")), "author": user(m["author"]),
        "web_url": f"{HOST}/{ppath(pid)}/-/merge_requests/{iid}",
        "references": {"full": f"{ppath(pid)}!{iid}"},
        "upvotes": m.get("upvotes", 0), "downvotes": 0,
        "labels": [label(n) for n in m["labels"]],
        "assignees": [user(n) for n in m["assignees"]],
        "reviewers": [user(n) for n in m["reviewers"]],
        "participants": [user(n) for n in dict.fromkeys(
            [m["author"]] + m["reviewers"])],
        "squash": False, "created_at": ago(m["age"]),
        "merge_status": "can_be_merged",
        "detailed_merge_status": "mergeable" if m["state"] == "opened" else "not_open",
        "has_conflicts": False,
        "source_branch": m["source_branch"], "target_branch": m["target_branch"],
        "diff_refs": {"base_sha": "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678", "head_sha": "e4f5a6b7c8d90a1b2c3d4e5f60718293a4b5c6d7", "start_sha": "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678"},
        "head_pipeline": m.get("pipeline"),
    }
    if m.get("milestone"):
        out["milestone"] = {"id": 1, "title": m["milestone"]}
    return out


def issue(i):
    iid = i["iid"]
    pid = i.get("project", PID)
    due = i.get("due_date")
    if due and due.startswith("@date:"):
        days = int(due[6:].rstrip("d"))
        due = (datetime.now(timezone.utc) + timedelta(days=days)).strftime("%Y-%m-%d")
    out = {
        "id": 2000 + i.get("gid", iid), "iid": iid, "project_id": pid, "title": i["title"],
        "description": i.get("description", ""), "state": i["state"],
        "author": user(i["author"]), "assignees": [user(n) for n in i["assignees"]],
        "labels": [label(n) for n in i["labels"]],
        "web_url": f"{HOST}/{ppath(pid)}/-/issues/{iid}",
        "references": {"full": f"{ppath(pid)}#{iid}"}, "issue_type": "issue",
        "due_date": due, "weight": i.get("weight"), "confidential": False,
        "created_at": ago(i["age"]), "upvotes": 0, "downvotes": 0,
    }
    if i.get("milestone"):
        out["milestone"] = {"id": 1, "title": i["milestone"]}
    return out


def todo(t):
    kind, num = t["target"].split(":")
    slug = "merge_requests" if kind == "mr" else "issues"
    tp = next((x for x in PROJ["projects"] if x["id"] == t.get("project")),
              PROJ["projects"][0])
    return {
        "id": t["id"], "project": project(tp),
        "action_name": t["action_name"], "target_type": t["target_type"],
        "target_url": f"{HOST}/{tp['path']}/-/{slug}/{num}", "body": t["body"],
        "state": "pending", "created_at": ago(t["age"]),
        "target": {"iid": int(num), "title": t["body"]},
    }


def pipeline(p):
    pid = p.get("project", PID)
    return {
        "id": p["id"], "project_id": pid, "status": p["status"],
        "source": p["source"], "ref": p["ref"],
        "web_url": f"{HOST}/{ppath(pid)}/-/pipelines/{p['id']}",
        "created_at": ago(p["age"]), "started_at": ago(p["age"]),
        "duration": p["duration"],
    }


def job(j, pipe_id):
    status = j["status"]
    started = status not in ("created", "pending", "skipped")
    finished = status in ("success", "failed", "canceled")
    return {
        "id": j["id"], "name": j["name"], "stage": j["stage"], "status": status,
        "created_at": ago("1h"), "started_at": ago("1h") if started else None,
        "finished_at": ago("50m") if finished else None, "duration": j["duration"],
        "user": user(ME), "runner": {"description": "shared-runner-01"},
        "allow_failure": False, "tag_list": [],
        "web_url": f"{HOST}/acme/widgets/-/jobs/{j['id']}",
    }


def diffs_for(iid):
    out = []
    for d in DIFFS.get(str(iid), []):
        raw = text("diffs/" + d["file"]).split("\n")
        if raw and raw[-1] == "":
            raw.pop()
        body = "\n".join(line or " " for line in raw) + "\n"
        out.append({
            "old_path": d["old_path"], "new_path": d["new_path"], "diff": body,
            "new_file": bool(d.get("new_file")), "renamed_file": False,
            "deleted_file": False, "collapsed": False, "too_large": False,
        })
    return out


def discussions_for(iid):
    out = []
    for d in DISCUSSIONS.get(str(iid), []):
        notes = []
        for n in d["notes"]:
            note = {"id": zlib.crc32((d["id"] + n["body"]).encode()) % 10**6, "body": n["body"],
                    "author": user(n["author"]), "system": bool(n.get("system")),
                    "created_at": ago(n["age"]), "position": None}
            if n.get("path"):
                note["type"] = "DiffNote"
                note["position"] = {
                    "base_sha": "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678", "head_sha": "e4f5a6b7c8d90a1b2c3d4e5f60718293a4b5c6d7", "start_sha": "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
                    "position_type": "text", "old_path": n["path"],
                    "new_path": n["path"], "old_line": None, "new_line": n["new_line"],
                }
            notes.append(note)
        out.append({"id": d["id"], "individual_note": bool(d.get("individual_note")),
                    "resolved": d.get("resolved"), "notes": notes})
    return out


def find(items, key, value):
    return next((x for x in items if x[key] == value), None)


def board_issue_list(list_id):
    open_issues = [i for i in ISSUES if i["state"] == "opened"]
    if list_id == BOARD["backlog_list_id"]:
        names = {l["label"] for l in BOARD["lists"]}
        sel = [i for i in open_issues if not names & set(i["labels"])]
    elif list_id == BOARD["closed_list_id"]:
        sel = [i for i in ISSUES if i["state"] == "closed"]
    else:
        lab = next(l["label"] for l in BOARD["lists"] if l["id"] == list_id)
        sel = [i for i in open_issues if lab in i["labels"]]
    return [issue(i) for i in sel]


def board_list(l):
    return {"id": l["id"], "label": label(l["label"]), "position": l["position"],
            "list_type": "label"}


def board_details():
    return {
        "id": BOARD["id"], "name": BOARD["name"],
        "backlog_list": {"id": BOARD["backlog_list_id"], "label": None,
                         "position": None, "list_type": "backlog"},
        "closed_list": {"id": BOARD["closed_list_id"], "label": None,
                        "position": None, "list_type": "closed"},
    }


def trace_for(job_id):
    raw = text("traces/failed.txt" if job_id == 23 else "traces/default.txt")
    return re.sub(r"(?<!\x1b)\[(\d+(?:;\d*)*[mK])", "\x1b[\\1", raw)


def route(path, query):
    """Return (status, body) for a GET; body is JSON-able or a str for plain text."""
    p = urllib.parse.unquote(path)
    if p == "/user":
        return user(ME)
    if p == "/groups":
        return PROJ["groups"]
    if p == "/projects":
        return [project(x) for x in PROJ["projects"]]
    if p == "/todos":
        return [todo(t) for t in TODOS] if query.get("state", ["pending"])[0] == "pending" else []
    if p == "/merge_requests":
        return mr_list(MRS, query)
    if p == "/issues":
        st = query.get("state", ["opened"])[0]
        return [issue(i) for i in ISSUES if st in ("all", i["state"])
                and ME in i["assignees"]]
    m = re.match(r"^/projects/(?:(\d+)|acme/widgets)(/.*)?$", p)
    if m:
        pid = int(m.group(1)) if m.group(1) else PID
        if m.group(1) and pid not in PROJECT_IDS and pid != PID:
            return None
        return project_route(pid, m.group(2) or "", query)
    m = re.match(r"^/groups/[^/]+/projects$", p)
    if m:
        return [project(x) for x in PROJ["projects"]]
    return None


def in_project(items, pid):
    return [x for x in items if x.get("project", PID) == pid]


def mr_list(items, query):
    state = query.get("state", ["opened"])[0]
    keep = [m for m in items if state in ("all", m["state"])]
    if query.get("scope", [""])[0] == "assigned_to_me":
        keep = [m for m in keep if ME in m["assignees"]]
    if query.get("reviewer_username", [""])[0] == ME:
        keep = [m for m in keep if ME in m["reviewers"]]
    return [mr(m) for m in keep]


def project_route(pid, sub, query):
    if sub == "":
        return project(next((x for x in PROJ["projects"] if x["id"] == pid),
                            PROJ["projects"][0]))
    if sub == "/merge_requests":
        return mr_list(in_project(MRS, pid), query)
    if sub == "/issues":
        st = query.get("state", ["opened"])[0]
        return [issue(i) for i in in_project(ISSUES, pid)
                if st in ("all", i["state"])]
    m = re.match(r"^/issues/(\d+)$", sub)
    if m:
        return issue(find(in_project(ISSUES, pid), "iid", int(m.group(1))))
    if re.match(r"^/issues/\d+/(discussions|participants|links|closed_by)$", sub):
        return []
    m = re.match(r"^/merge_requests/(\d+)$", sub)
    if m:
        return mr(find(in_project(MRS, pid), "iid", int(m.group(1))))
    m = re.match(r"^/merge_requests/(\d+)/(\w+)$", sub)
    if m:
        iid, what = int(m.group(1)), m.group(2)
        if what == "diffs":
            return diffs_for(iid)
        if what == "discussions":
            return discussions_for(iid)
        if what == "approvals":
            mm = find(in_project(MRS, pid), "iid", iid)
            return {"approvals_required": 1,
                    "approvals_left": 0 if mm.get("approved_by") else 1,
                    "approved_by": [{"user": user(n)} for n in mm.get("approved_by", [])]}
        if what == "participants":
            return mr(find(in_project(MRS, pid), "iid", iid))["participants"]
        if what in ("award_emoji", "related_merge_requests", "closes_issues", "notes"):
            return []
    if sub == "/labels":
        return list(LABELS.values())
    if sub in ("/members/all", "/members", "/users"):
        return [user(n) for n in USERS]
    if sub == "/milestones":
        return [{"id": 1, "title": "v1.4"}]
    if sub == "/pipelines":
        st = query.get("status", [""])[0]
        return [pipeline(x) for x in in_project(PIPES["pipelines"], pid)
                if not st or x["status"] == st]
    m = re.match(r"^/pipelines/(\d+)$", sub)
    if m:
        return pipeline(find(PIPES["pipelines"], "id", int(m.group(1))))
    m = re.match(r"^/pipelines/(\d+)/jobs$", sub)
    if m:
        pid = m.group(1)
        jobs = PIPES["jobs"].get(pid, PIPES["jobs"]["default"])
        return [job(j, pid) for j in jobs]
    if re.match(r"^/pipelines/\d+/variables$", sub):
        return []
    m = re.match(r"^/jobs/(\d+)/trace$", sub)
    if m:
        return trace_for(int(m.group(1)))
    if sub in ("/pipeline_schedules", "/variables"):
        return []
    if sub == "/repository/branches":
        return [{"name": n, "merged": False, "protected": n == "main",
                 "default": n == "main"} for n in
                ("main", "feat/retry-backoff", "fix/pagination", "refactor/serde-config")]
    if sub == "/boards":
        return [{"id": BOARD["id"], "name": BOARD["name"]}]
    m = re.match(r"^/boards/(\d+)$", sub)
    if m:
        return board_details()
    m = re.match(r"^/boards/(\d+)/lists$", sub)
    if m:
        return [board_list(l) for l in BOARD["lists"]]
    m = re.match(r"^/boards/(\d+)/lists/(\d+)/issues$", sub)
    if m:
        return board_issue_list(int(m.group(2)))
    return None


def build_big(n_projects, n_mrs, n_issues, n_todos, n_pipelines):
    """Replace the fictional world by a generated one. Deterministic."""
    global PROJ, MRS, ISSUES, TODOS, PIPES, PROJECT_PATHS
    users = list(USERS)
    label_names = list(LABELS)
    projects = [{"id": 1001 + k, "name": f"proj-{k:02d}", "path": f"acme/proj-{k:02d}",
                 "default_branch": "main", "age": f"{k * 3 + 1}h"}
                for k in range(n_projects)]
    pids = [p["id"] for p in projects]
    weights = [1.0 / (k + 1) for k in range(n_projects)]
    total = sum(weights)

    def spread(n):
        counts = [int(n * w / total) for w in weights]
        for k in range(n - sum(counts)):
            counts[k % n_projects] += 1
        return counts

    def pick(seq, i, n):
        return [seq[(i + j) % len(seq)] for j in range(n)]

    mrs, issues, gid = [], [], 0
    for pid, count in zip(pids, spread(n_mrs)):
        for iid in range(1, count + 1):
            gid += 1
            mrs.append({
                "iid": iid, "gid": gid, "project": pid, "title": f"Merge request {gid}",
                "description": "", "state": "opened",
                "draft": gid % 7 == 0, "author": users[gid % len(users)],
                "labels": pick(label_names, gid, 2),
                "assignees": pick(users, gid, 1 + gid % 2),
                "reviewers": pick(users, gid + 1, 1),
                "age": f"{gid * 9 + 5}m",
                "source_branch": f"feat/b{gid}", "target_branch": "main",
            })
    gid = 0
    for pid, count in zip(pids, spread(n_issues)):
        for iid in range(1, count + 1):
            gid += 1
            issues.append({
                "iid": iid, "gid": gid, "project": pid, "title": f"Issue {gid}",
                "description": "", "state": "opened",
                "author": users[gid % len(users)],
                "assignees": pick(users, gid, 1 + gid % 2),
                "labels": pick(label_names, gid, 2), "age": f"{gid * 11 + 3}m",
            })
    todos = []
    for k in range(n_todos):
        src = mrs[k * 3 % len(mrs)] if k % 2 == 0 else issues[k * 3 % len(issues)]
        kind = "mr" if k % 2 == 0 else "issue"
        todos.append({
            "id": 5000 + k, "project": src["project"],
            "action_name": "review_requested" if kind == "mr" else "assigned",
            "target_type": "MergeRequest" if kind == "mr" else "Issue",
            "target": f"{kind}:{src['iid']}", "body": src["title"],
            "age": f"{k * 17 + 2}m",
        })
    statuses = ["success", "failed", "running", "success", "canceled"]
    pipes, pid_seq = [], 9000
    for pid, count in zip(pids, spread(n_pipelines)):
        for j in range(count):
            pid_seq += 1
            pipes.append({"id": pid_seq, "project": pid, "status": statuses[pid_seq % 5],
                          "source": "push", "ref": "main", "age": f"{pid_seq % 97 * 11 + 1}m",
                          "duration": 60 + pid_seq % 300})
    PROJ = {"groups": PROJ["groups"], "projects": projects}
    PROJECT_PATHS = {p["id"]: p["path"] for p in projects}
    PROJECT_IDS.update(pids)
    MRS, ISSUES, TODOS = mrs, issues, todos
    PIPES = {"pipelines": pipes, "jobs": PIPES["jobs"]}


STATS_LOCK = threading.Lock()
STATS = {"total": 0, "bytes": 0, "paths": {}}


def template(path):
    out = []
    for seg in path.split("/"):
        out.append(":id" if seg.isdigit() else seg)
    return "/".join(out)


def stats_record(method, path, nbytes):
    key = f"{method} {template(path)}"
    with STATS_LOCK:
        STATS["total"] += 1
        STATS["bytes"] += nbytes
        entry = STATS["paths"].setdefault(key, {"count": 0, "bytes": 0})
        entry["count"] += 1
        entry["bytes"] += nbytes


def stats_reset():
    with STATS_LOCK:
        STATS["total"] = 0
        STATS["bytes"] = 0
        STATS["paths"] = {}


def paginate(items, query, host, path):
    """Slice a list like GitLab and return (slice, headers)."""
    def num(name, default):
        try:
            return int(query.get(name, [str(default)])[0])
        except ValueError:
            return default

    per_page = max(1, min(100, num("per_page", 20)))
    page = max(1, num("page", 1))
    total = len(items)
    pages = max(1, -(-total // per_page))
    chunk = items[(page - 1) * per_page: page * per_page]

    def link(n, rel):
        q = {k: v[0] for k, v in query.items()}
        q.update(page=n, per_page=per_page)
        return f'<http://{host}/api/v4{path}?{urllib.parse.urlencode(q)}>; rel="{rel}"'

    links = []
    if page > 1:
        links.append(link(page - 1, "prev"))
    if page < pages:
        links.append(link(page + 1, "next"))
    links += [link(1, "first"), link(pages, "last")]
    headers = {
        "X-Page": str(page), "X-Per-Page": str(per_page),
        "X-Next-Page": str(page + 1) if page < pages else "",
        "X-Prev-Page": str(page - 1) if page > 1 else "",
        "X-Total": str(total), "X-Total-Pages": str(pages),
        "Link": ", ".join(links),
    }
    return chunk, headers


class Server(http.server.ThreadingHTTPServer):
    request_queue_size = 256
    daemon_threads = True


class Handler(http.server.BaseHTTPRequestHandler):
    def _send(self, code, body, ctype="application/json", headers=None):
        data = body.encode() if isinstance(body, str) else body
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        for k, v in (headers or {}).items():
            self.send_header(k, v)
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(data)
        return len(data)

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        if path == "/__stats":
            with STATS_LOCK:
                self._send(200, json.dumps(STATS))
            return
        if LATENCY_MS:
            time.sleep(LATENCY_MS / 1000.0)
        if path.startswith("/api/v4"):
            path = path[len("/api/v4"):]
        query = urllib.parse.parse_qs(parsed.query)
        result = route(path, query)
        headers = {}
        if result is None:
            if DEBUG:
                print("UNHANDLED GET", self.path, file=sys.stderr, flush=True)
            n = self._send(200, "[]")
        elif isinstance(result, str):
            n = self._send(200, result, "text/plain")
        else:
            if isinstance(result, list) and (BIG or "per_page" in query
                                             or "page" in query):
                result, headers = paginate(result, query,
                                           self.headers.get("Host", "localhost"), path)
            body = json.dumps(result)
            if path == "/todos":
                etag = '"' + hashlib.md5(body.encode()).hexdigest() + '"'
                headers["ETag"] = etag
                if self.headers.get("If-None-Match") == etag:
                    n = self._send(304, b"", headers=headers)
                    stats_record("GET", path, n)
                    return
            n = self._send(200, body, headers=headers)
        stats_record("GET", path, n)

    def do_POST(self):
        if self.path == "/__reset":
            stats_reset()
            self._send(200, "{}")
            return
        self.send_response(404)
        self.send_header("Content-Length", "0")
        self.end_headers()

    do_PUT = do_DELETE = do_POST

    def log_message(self, *args):
        pass


def main():
    global LATENCY_MS, BIG
    ap = argparse.ArgumentParser(description="Fictional GitLab REST v4 server.")
    ap.add_argument("port", nargs="?", type=int, default=8765)
    ap.add_argument("--latency-ms", type=float, default=0.0)
    ap.add_argument("--big", action="store_true")
    ap.add_argument("--projects", type=int, default=40)
    ap.add_argument("--mrs", type=int, default=500)
    ap.add_argument("--issues", type=int, default=500)
    ap.add_argument("--todos", type=int, default=120)
    ap.add_argument("--pipelines", type=int, default=400)
    args = ap.parse_args()
    LATENCY_MS = args.latency_ms
    if args.big:
        BIG = True
        build_big(args.projects, args.mrs, args.issues, args.todos, args.pipelines)
    srv = Server(("127.0.0.1", args.port), Handler)
    srv.serve_forever()


if __name__ == "__main__":
    main()
