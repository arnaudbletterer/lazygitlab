#!/usr/bin/env python3
"""Fictional GitLab REST v4 server for demo screenshots. Stdlib only.

Serves the endpoints lazygitlab calls from the JSON fixtures in ./fixtures.
Set MOCK_DEBUG=1 to log unhandled routes on stderr.
"""
import http.server
import json
import os
import re
import sys
import zlib
import urllib.parse
from datetime import datetime, timedelta, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fixtures")
HOST = "https://gitlab.example.com"
ME = "ada"
PID = 42
DEBUG = bool(os.environ.get("MOCK_DEBUG"))


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
    out = {
        "id": 1000 + iid, "iid": iid, "project_id": PID, "title": m["title"],
        "description": m.get("description", ""), "state": m["state"],
        "draft": bool(m.get("draft")), "author": user(m["author"]),
        "web_url": f"{HOST}/acme/widgets/-/merge_requests/{iid}",
        "references": {"full": f"acme/widgets!{iid}"},
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
    due = i.get("due_date")
    if due and due.startswith("@date:"):
        days = int(due[6:].rstrip("d"))
        due = (datetime.now(timezone.utc) + timedelta(days=days)).strftime("%Y-%m-%d")
    out = {
        "id": 2000 + iid, "iid": iid, "project_id": PID, "title": i["title"],
        "description": i.get("description", ""), "state": i["state"],
        "author": user(i["author"]), "assignees": [user(n) for n in i["assignees"]],
        "labels": [label(n) for n in i["labels"]],
        "web_url": f"{HOST}/acme/widgets/-/issues/{iid}",
        "references": {"full": f"acme/widgets#{iid}"}, "issue_type": "issue",
        "due_date": due, "weight": i.get("weight"), "confidential": False,
        "created_at": ago(i["age"]), "upvotes": 0, "downvotes": 0,
    }
    if i.get("milestone"):
        out["milestone"] = {"id": 1, "title": i["milestone"]}
    return out


def todo(t):
    kind, num = t["target"].split(":")
    slug = "merge_requests" if kind == "mr" else "issues"
    return {
        "id": t["id"], "project": project(PROJ["projects"][0]),
        "action_name": t["action_name"], "target_type": t["target_type"],
        "target_url": f"{HOST}/acme/widgets/-/{slug}/{num}", "body": t["body"],
        "state": "pending", "created_at": ago(t["age"]),
        "target": {"iid": int(num), "title": t["body"]},
    }


def pipeline(p):
    return {
        "id": p["id"], "project_id": PID, "status": p["status"],
        "source": p["source"], "ref": p["ref"],
        "web_url": f"{HOST}/acme/widgets/-/pipelines/{p['id']}",
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
    if p in ("/merge_requests", f"/projects/{PID}/merge_requests",
             "/projects/acme/widgets/merge_requests"):
        state = query.get("state", ["opened"])[0]
        keep = [m for m in MRS if state in ("all", m["state"])]
        scope = query.get("scope", [""])[0]
        if scope == "assigned_to_me":
            keep = [m for m in keep if ME in m["assignees"]]
        if query.get("reviewer_username", [""])[0] == ME:
            keep = [m for m in keep if ME in m["reviewers"]]
        return [mr(m) for m in keep]
    if p == "/issues":
        st = query.get("state", ["opened"])[0]
        return [issue(i) for i in ISSUES if st in ("all", i["state"])
                and ME in i["assignees"]]
    m = re.match(r"^/projects/(?:42|acme/widgets)(/.*)?$", p)
    if m:
        return project_route(m.group(1) or "", query)
    m = re.match(r"^/groups/[^/]+/projects$", p)
    if m:
        return [project(x) for x in PROJ["projects"]]
    return None


def project_route(sub, query):
    if sub == "":
        return project(PROJ["projects"][0])
    if sub == "/issues":
        st = query.get("state", ["opened"])[0]
        return [issue(i) for i in ISSUES if st in ("all", i["state"])]
    m = re.match(r"^/issues/(\d+)$", sub)
    if m:
        return issue(find(ISSUES, "iid", int(m.group(1))))
    if re.match(r"^/issues/\d+/(discussions|participants|links|closed_by)$", sub):
        return []
    m = re.match(r"^/merge_requests/(\d+)$", sub)
    if m:
        return mr(find(MRS, "iid", int(m.group(1))))
    m = re.match(r"^/merge_requests/(\d+)/(\w+)$", sub)
    if m:
        iid, what = int(m.group(1)), m.group(2)
        if what == "diffs":
            return diffs_for(iid)
        if what == "discussions":
            return discussions_for(iid)
        if what == "approvals":
            mm = find(MRS, "iid", iid)
            return {"approvals_required": 1,
                    "approvals_left": 0 if mm.get("approved_by") else 1,
                    "approved_by": [{"user": user(n)} for n in mm.get("approved_by", [])]}
        if what == "participants":
            return mr(find(MRS, "iid", iid))["participants"]
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
        return [pipeline(x) for x in PIPES["pipelines"] if not st or x["status"] == st]
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


class Handler(http.server.BaseHTTPRequestHandler):
    def _send(self, code, body, ctype="application/json"):
        data = body.encode() if isinstance(body, str) else body
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        if path.startswith("/api/v4"):
            path = path[len("/api/v4"):]
        result = route(path, urllib.parse.parse_qs(parsed.query))
        if result is None:
            if DEBUG:
                print("UNHANDLED GET", self.path, file=sys.stderr, flush=True)
            self._send(200, "[]")
        elif isinstance(result, str):
            self._send(200, result, "text/plain")
        else:
            self._send(200, json.dumps(result))

    def do_POST(self):
        self.send_response(404)
        self.send_header("Content-Length", "0")
        self.end_headers()

    do_PUT = do_DELETE = do_POST

    def log_message(self, *args):
        pass


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler)
    srv.serve_forever()


if __name__ == "__main__":
    main()
