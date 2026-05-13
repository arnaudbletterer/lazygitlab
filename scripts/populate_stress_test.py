import requests
import os
import time

TOKEN = os.environ.get("GITLAB_TOKEN")
URL = "https://gitlab.com/api/v4"
PROJECT_ID = "82156545"

headers = {"PRIVATE-TOKEN": TOKEN}

def check_res(res):
    if res.status_code >= 400:
        print(f"Error: {res.status_code} - {res.text}")

print(f"Populating project {PROJECT_ID}...")

# 1. Create many labels
print("Creating 30 labels...")
for i in range(30):
    requests.post(f"{URL}/projects/{PROJECT_ID}/labels", headers=headers, json={
        "name": f"label-{i:02}", "color": f"#{i*5:02x}{i*5:02x}{i*5:02x}"
    })

# 2. Create many branches and files (Deep Tree)
print("Creating 20 branches and deep files...")
for i in range(20):
    branch_name = f"feature-branch-{i:02}"
    requests.post(f"{URL}/projects/{PROJECT_ID}/repository/branches", headers=headers, params={
        "branch": branch_name, "ref": "main"
    })
    # Add a deeply nested file
    path = "/".join([f"dir-{j}" for j in range(10)]) + f"/file-{i}.txt"
    requests.post(f"{URL}/projects/{PROJECT_ID}/repository/files/{path.replace('/', '%2F')}", headers=headers, json={
        "branch": branch_name, "content": f"Content for file {i}", "commit_message": f"Add file {i}"
    })

# 3. Create many issues
print("Creating 50 issues...")
for i in range(50):
    requests.post(f"{URL}/projects/{PROJECT_ID}/issues", headers=headers, json={
        "title": f"Stress Issue {i:03}",
        "description": f"Detailed description for issue {i}",
        "labels": f"label-{i%30:02}"
    })

# 4. Create many MRs
print("Creating 20 MRs...")
for i in range(20):
    requests.post(f"{URL}/projects/{PROJECT_ID}/merge_requests", headers=headers, json={
        "source_branch": f"feature-branch-{i:02}",
        "target_branch": "main",
        "title": f"Stress MR {i:03}",
        "description": f"Description for MR {i}"
    })

print("Done populating!")
