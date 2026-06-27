import argparse
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import requests
from dotenv import load_dotenv

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_PROJECT = "abletterer/lazygitlab-test"
GROUND_FILE = REPO_ROOT / "data" / "test_ground.json"
GROUND_FILE.parent.mkdir(parents=True, exist_ok=True)

LABEL_COUNT = 30
BRANCH_COUNT = 20
ISSUE_COUNT = 50
MR_COUNT = 20
DEEP_TREE_DEPTH = 10
MASSIVE_MR_FILES = 120
MASSIVE_MR_BRANCH = "massive-mr-source"
BOARD_NAME = "Test Board (auto-seeded)"
GITLAB_CI_CONTENT = (
    "stages:\n"
    "  - test\n"
    "\n"
    "smoke:\n"
    "  stage: test\n"
    "  script:\n"
    "    - echo \"auto-seeded by populate_test_ground.py\"\n"
)


def url_from_env() -> str:
    raw = os.environ.get("GITLAB_URL", "https://gitlab.com").strip()
    if raw.startswith("http://") or raw.startswith("https://"):
        return f"{raw.rstrip('/')}/api/v4"
    return f"https://{raw.rstrip('/')}/api/v4"


def headers() -> dict[str, str]:
    token = os.environ.get("GITLAB_TOKEN", "").strip()
    if not token:
        sys.exit("GITLAB_TOKEN is not set (load .env or export it).")
    return {"PRIVATE-TOKEN": token}


def paginated_get(session: requests.Session, path: str, params: dict[str, Any] | None = None) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    page = 1
    while True:
        merged = {"per_page": 100, "page": page, **(params or {})}
        r = session.get(path, params=merged, timeout=30)
        r.raise_for_status()
        chunk = r.json()
        if not isinstance(chunk, list):
            return chunk
        out.extend(chunk)
        if len(chunk) < 100:
            return out
        page += 1


def get_project(session: requests.Session, base: str, project_path: str) -> dict[str, Any]:
    encoded = requests.utils.quote(project_path, safe="")
    r = session.get(f"{base}/projects/{encoded}", timeout=30)
    r.raise_for_status()
    return r.json()


def existing_labels(session: requests.Session, base: str, project_id: int) -> set[str]:
    rows = paginated_get(session, f"{base}/projects/{project_id}/labels", {"per_page": 100})
    return {row["name"] for row in rows}


def ensure_labels(session: requests.Session, base: str, project_id: int) -> list[str]:
    have = existing_labels(session, base, project_id)
    created: list[str] = []
    for i in range(LABEL_COUNT):
        name = f"label-{i:02d}"
        if name in have:
            continue
        color = f"#{i * 5:02x}{i * 5:02x}{i * 5:02x}"
        r = session.post(f"{base}/projects/{project_id}/labels", json={"name": name, "color": color}, timeout=30)
        if r.status_code >= 400:
            print(f"  ! label {name}: {r.status_code} {r.text[:200]}")
            continue
        created.append(name)
    return sorted(set(created) | have)


def existing_branches(session: requests.Session, base: str, project_id: int) -> set[str]:
    rows = paginated_get(session, f"{base}/projects/{project_id}/repository/branches", {"per_page": 100})
    return {row["name"] for row in rows}


def ensure_branches_and_deep_files(session: requests.Session, base: str, project_id: int, ref: str) -> list[str]:
    have = existing_branches(session, base, project_id)
    created: list[str] = []
    for i in range(BRANCH_COUNT):
        branch_name = f"feature-branch-{i:02d}"
        if branch_name not in have:
            r = session.post(
                f"{base}/projects/{project_id}/repository/branches",
                params={"branch": branch_name, "ref": ref},
                timeout=30,
            )
            if r.status_code >= 400:
                print(f"  ! branch {branch_name}: {r.status_code} {r.text[:200]}")
                continue
            created.append(branch_name)
        path = "/".join(f"dir-{j}" for j in range(DEEP_TREE_DEPTH)) + f"/file-{i}.txt"
        encoded_path = requests.utils.quote(path, safe="")
        file_url = f"{base}/projects/{project_id}/repository/files/{encoded_path}"
        check = session.get(file_url, params={"ref": branch_name}, timeout=30)
        if check.status_code == 200:
            continue
        r = session.post(
            file_url,
            params={"branch": branch_name},
            json={"branch": branch_name, "content": f"Content for file {i}", "commit_message": f"Add file {i}"},
            timeout=30,
        )
        if r.status_code >= 400:
            print(f"  ! file {path} on {branch_name}: {r.status_code} {r.text[:200]}")
    return sorted(set(created) | have)


def existing_issues(session: requests.Session, base: str, project_id: int) -> set[int]:
    rows = paginated_get(session, f"{base}/projects/{project_id}/issues", {"per_page": 100, "scope": "all"})
    return {row["iid"] for row in rows}


def ensure_issues(session: requests.Session, base: str, project_id: int) -> list[int]:
    have = existing_issues(session, base, project_id)
    created: list[int] = []
    for i in range(ISSUE_COUNT):
        iid = i + 1
        if iid in have:
            continue
        r = session.post(
            f"{base}/projects/{project_id}/issues",
            json={
                "title": f"Stress Issue {iid:03d}",
                "description": f"Detailed description for issue {iid}",
                "labels": f"label-{i % LABEL_COUNT:02d}",
            },
            timeout=30,
        )
        if r.status_code >= 400:
            print(f"  ! issue #{iid}: {r.status_code} {r.text[:200]}")
            continue
        created.append(iid)
    return sorted(set(created) | have)


def existing_mrs(session: requests.Session, base: str, project_id: int) -> set[int]:
    rows = paginated_get(session, f"{base}/projects/{project_id}/merge_requests", {"per_page": 100, "scope": "all"})
    return {row["iid"] for row in rows}


def ensure_mrs(session: requests.Session, base: str, project_id: int, branches: list[str], ref: str) -> list[int]:
    have = existing_mrs(session, base, project_id)
    created: list[int] = []
    for i in range(MR_COUNT):
        iid = i + 1
        if iid in have:
            continue
        branch_name = f"feature-branch-{i:02d}"
        if branch_name not in branches:
            print(f"  ! skip MR !{iid}: missing branch {branch_name}")
            continue
        r = session.post(
            f"{base}/projects/{project_id}/merge_requests",
            json={
                "source_branch": branch_name,
                "target_branch": ref,
                "title": f"Stress MR {iid:03d}",
                "description": f"Description for MR {iid}",
            },
            timeout=30,
        )
        if r.status_code >= 400:
            print(f"  ! MR !{iid}: {r.status_code} {r.text[:200]}")
            continue
        created.append(iid)
    return sorted(set(created) | have)


def find_massive_mr(session: requests.Session, base: str, project_id: int) -> int | None:
    rows = paginated_get(
        session,
        f"{base}/projects/{project_id}/merge_requests",
        {"per_page": 100, "scope": "all", "source_branch": MASSIVE_MR_BRANCH},
    )
    for row in rows:
        if row.get("source_branch") == MASSIVE_MR_BRANCH:
            return int(row["iid"])
    return None


def seed_massive_mr(session: requests.Session, base: str, project_id: int, ref: str) -> int | None:
    existing_iid = find_massive_mr(session, base, project_id)
    if existing_iid is not None:
        return existing_iid

    branches = existing_branches(session, base, project_id)
    if MASSIVE_MR_BRANCH not in branches:
        r = session.post(
            f"{base}/projects/{project_id}/repository/branches",
            params={"branch": MASSIVE_MR_BRANCH, "ref": ref},
            timeout=30,
        )
        if r.status_code >= 400:
            print(f"  ! massive-mr branch: {r.status_code} {r.text[:200]}")
            return None

    print(f"  seeding {MASSIVE_MR_FILES} files on {MASSIVE_MR_BRANCH}...")
    for i in range(MASSIVE_MR_FILES):
        path = f"src/auto/file-{i:04d}.txt"
        encoded_path = requests.utils.quote(path, safe="")
        file_url = f"{base}/projects/{project_id}/repository/files/{encoded_path}"
        check = session.get(file_url, params={"ref": MASSIVE_MR_BRANCH}, timeout=30)
        if check.status_code == 200:
            continue
        r = session.post(
            file_url,
            params={"branch": MASSIVE_MR_BRANCH},
            json={
                "branch": MASSIVE_MR_BRANCH,
                "content": f"line for file {i}\n" + "\n".join(f"row {j}" for j in range(20)),
                "commit_message": f"Add file {i}",
            },
            timeout=30,
        )
        if r.status_code >= 400:
            print(f"  ! massive-mr file {path}: {r.status_code} {r.text[:200]}")
            return None
        if i % 20 == 0:
            print(f"    {i}/{MASSIVE_MR_FILES}")

    r = session.post(
        f"{base}/projects/{project_id}/merge_requests",
        json={
            "source_branch": MASSIVE_MR_BRANCH,
            "target_branch": ref,
            "title": "Massive MR (auto-seeded)",
            "description": f"Auto-seeded by populate_test_ground.py with {MASSIVE_MR_FILES} files.",
        },
        timeout=30,
    )
    if r.status_code >= 400:
        print(f"  ! massive MR: {r.status_code} {r.text[:200]}")
        return None
    return int(r.json()["iid"])


def ensure_gitlab_ci(session: requests.Session, base: str, project_id: int, ref: str) -> bool:
    encoded = requests.utils.quote(".gitlab-ci.yml", safe="")
    file_url = f"{base}/projects/{project_id}/repository/files/{encoded}"
    check = session.get(file_url, params={"ref": ref}, timeout=30)
    if check.status_code == 200:
        return True
    r = session.post(
        file_url,
        params={"branch": ref},
        json={"branch": ref, "content": GITLAB_CI_CONTENT, "commit_message": "Add auto-seeded CI config"},
        timeout=30,
    )
    if r.status_code >= 400:
        print(f"  ! .gitlab-ci.yml: {r.status_code} {r.text[:200]}")
        return False
    return True


def find_board(session: requests.Session, base: str, project_id: int, name: str) -> int | None:
    rows = paginated_get(session, f"{base}/projects/{project_id}/boards", {"per_page": 100})
    for row in rows:
        if row.get("name") == name:
            return int(row["id"])
    return None


def ensure_board(session: requests.Session, base: str, project_id: int) -> int | None:
    existing = find_board(session, base, project_id, BOARD_NAME)
    if existing is not None:
        return existing
    r = session.post(f"{base}/projects/{project_id}/boards", json={"name": BOARD_NAME}, timeout=30)
    if r.status_code >= 400:
        print(f"  ! board: {r.status_code} {r.text[:200]}")
        return None
    return int(r.json()["id"])


def ensure_board_list(session: requests.Session, base: str, project_id: int, board_id: int) -> bool:
    rows = paginated_get(session, f"{base}/projects/{project_id}/boards/{board_id}/lists", {"per_page": 100})
    if rows:
        return True
    labels = paginated_get(session, f"{base}/projects/{project_id}/labels", {"per_page": 100})
    target = next((l for l in labels if l.get("name") == "label-00"), None)
    if target is None and labels:
        target = labels[0]
    if target is None:
        print("  ! board list: no labels available to anchor the list")
        return False
    r = session.post(
        f"{base}/projects/{project_id}/boards/{board_id}/lists",
        json={"label_id": int(target["id"])},
        timeout=30,
    )
    if r.status_code >= 400:
        print(f"  ! board list: {r.status_code} {r.text[:200]}")
        return False
    return True


def trigger_pipeline(session: requests.Session, base: str, project_id: int, ref: str) -> int | None:
    r = session.post(f"{base}/projects/{project_id}/pipeline", params={"ref": ref}, timeout=30)
    if r.status_code >= 400:
        print(f"  ! pipeline: {r.status_code} {r.text[:200]}")
        return None
    return int(r.json()["id"])


def reset(session: requests.Session, base: str, project_id: int, project_path: str) -> None:
    print(f"!! Reset is destructive. Deleting project {project_path}...")
    r = session.delete(f"{base}/projects/{project_id}", timeout=30)
    r.raise_for_status()
    print("Project scheduled for deletion. Create it manually and re-run without --reset.")
    sys.exit(0)


def main() -> int:
    load_dotenv(REPO_ROOT / ".env")
    parser = argparse.ArgumentParser(description="Idempotent seed for lazygitlab's test ground.")
    parser.add_argument("--project", default=DEFAULT_PROJECT, help="Project path (default: %(default)s)")
    parser.add_argument("--reset", action="store_true", help="Delete the project and exit (DESTRUCTIVE).")
    parser.add_argument("--with-massive-mr", action="store_true", help="Seed a 120-file MR on a dedicated branch.")
    parser.add_argument("--dry-run", action="store_true", help="Print what would happen, make no changes.")
    args = parser.parse_args()

    base = url_from_env()
    print(f"Using GitLab: {base}")
    print(f"Project:      {args.project}")

    if args.dry_run:
        print("Dry run: not contacting GitLab.")
        return 0

    with requests.Session() as session:
        session.headers.update(headers())
        project = get_project(session, base, args.project)
        project_id = int(project["id"])
        default_ref = project.get("default_branch") or "main"
        print(f"Resolved:     id={project_id} default_branch={default_ref}")

        if args.reset:
            reset(session, base, project_id, args.project)

        print("Seeding labels...")
        labels = ensure_labels(session, base, project_id)
        print(f"  -> {len(labels)} labels present")

        print("Seeding branches + deep tree files...")
        branches = ensure_branches_and_deep_files(session, base, project_id, default_ref)
        print(f"  -> {len(branches)} branches present")

        print("Seeding issues...")
        issues = ensure_issues(session, base, project_id)
        print(f"  -> {len(issues)} issues present")

        print("Seeding merge requests...")
        mrs = ensure_mrs(session, base, project_id, branches, default_ref)
        print(f"  -> {len(mrs)} MRs present")

        print("Seeding .gitlab-ci.yml + pipeline...")
        ci_ok = ensure_gitlab_ci(session, base, project_id, default_ref)
        pipeline_id: int | None = None
        if ci_ok:
            pipeline_id = trigger_pipeline(session, base, project_id, default_ref)
        print(f"  -> .gitlab-ci.yml={'ok' if ci_ok else 'FAILED'} pipeline_id={pipeline_id}")

        print("Seeding board + list...")
        board_id = ensure_board(session, base, project_id)
        board_list_ok = False
        if board_id is not None:
            board_list_ok = ensure_board_list(session, base, project_id, board_id)
        print(f"  -> board_id={board_id} list={'ok' if board_list_ok else 'FAILED'}")

        massive_iid: int | None = None
        if args.with_massive_mr:
            print("Seeding massive MR (120 files)...")
            massive_iid = seed_massive_mr(session, base, project_id, default_ref)
            print(f"  -> massive MR iid={massive_iid}")

        ground = {
            "url": base,
            "project": {
                "id": project_id,
                "path_with_namespace": project.get("path_with_namespace", args.project),
                "web_url": project.get("web_url"),
                "default_branch": default_ref,
            },
            "counts": {
                "labels": len(labels),
                "branches": len(branches),
                "issues": len(issues),
                "merge_requests": len(mrs),
            },
            "anchors": {
                "first_label": "label-00",
                "first_branch": "feature-branch-00",
                "first_issue_iid": min(issues) if issues else None,
                "first_mr_iid": min(mrs) if mrs else None,
            },
            "extras": {
                "gitlab_ci_present": ci_ok,
                "pipeline_id": pipeline_id,
                "board_id": board_id,
                "board_list_present": board_list_ok,
                "deep_tree_root": "dir-0",
            },
            "massive_mr": {
                "iid": massive_iid,
                "branch": MASSIVE_MR_BRANCH,
                "files_count": MASSIVE_MR_FILES if massive_iid is not None else 0,
            },
            "last_seeded_at": datetime.now(timezone.utc).isoformat(),
        }
        GROUND_FILE.write_text(__import__("json").dumps(ground, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"Wrote {GROUND_FILE.relative_to(REPO_ROOT)}")
        return 0


if __name__ == "__main__":
    sys.exit(main())
