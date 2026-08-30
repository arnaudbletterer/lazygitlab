# lazygitlab

<p align="center">
  <strong>The fast, keyboard-driven Terminal UI for GitLab.</strong><br>
  <em>Manage your merge requests, issues, Kanban boards, CI/CD pipelines, and repositories directly from the comfort of your terminal.</em>
</p>

<div class="version-banner" align="center">
  <span class="version-pill version-tag">📦 <strong id="doc-version">v0.16.0</strong></span>
  <span class="version-pill version-commit" id="doc-commit"></span>
  <span class="version-pill version-dirty" id="doc-dirty" style="display:none;">⚠️ uncommitted changes</span>
</div>

---

![lazygitlab Personal Dashboard](assets/screenshots/dashboard.svg#only-dark)
![lazygitlab Personal Dashboard (Light)](assets/screenshots/dashboard-light.svg#only-light)
*Click any screenshot to zoom in full resolution.*

---

## ⚡ Highlights

=== "Personal Dashboard"
    ![Personal Dashboard](assets/screenshots/dashboard.svg#only-dark)
    ![Personal Dashboard (Light)](assets/screenshots/dashboard-light.svg#only-light)
    Consolidated view of all your assigned To-Dos, Merge Requests awaiting your review, active Issues, and cross-project CI/CD pipelines.

=== "Code Review & Diffs"
    ![MR Code Review & Diffs](assets/screenshots/mr_diff.svg#only-dark)
    ![MR Code Review & Diffs (Light)](assets/screenshots/mr_diff-light.svg#only-light)
    Full split/unified diffs, syntax highlighting, range comments, approval status, one-key merge, squash, and rebase.

=== "Sprint Kanban Boards"
    ![Sprint Kanban Board](assets/screenshots/kanban.svg#only-dark)
    ![Sprint Kanban Board (Light)](assets/screenshots/kanban-light.svg#only-light)
    Multi-column Kanban boards with fast issue drag-and-drop, label assignment, milestone filters, and epic tracking.

=== "CI/CD & Live Job Logs"
    ![CI/CD Live Job Logs](assets/screenshots/job_log.svg#only-dark)
    ![CI/CD Live Job Logs (Light)](assets/screenshots/job_log-light.svg#only-light)
    Monitor live pipelines, inspect colorized job logs with ANSI support, section collapsing, and search.

=== "Repository Explorer"
    ![Repository Tree & File Viewer](assets/screenshots/repo_tree.svg#only-dark)
    ![Repository Tree & File Viewer (Light)](assets/screenshots/repo_tree-light.svg#only-light)
    Browse repository directory trees, switch branches on the fly, and inspect file contents with syntect syntax highlighting.

---

## 🚀 Quick Start

### 1. Installation

```bash
cargo install lazygitlab
```

### 2. Configuration

Create a GitLab Personal Access Token with `api` and `read_user` scopes.

```bash
# In your ~/.bashrc, ~/.zshrc, or .env
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
export GITLAB_URL="https://gitlab.com" # Or your self-hosted GitLab instance
```

### 3. Launch

```bash
lazygitlab
```

When run inside any local git repository whose origin is on GitLab, `lazygitlab` automatically connects and selects that project for you!

---

## 🧭 Navigation & Shortcuts

| Key | Action |
| --- | --- |
| <kbd>?</kbd> | Open Interactive Help / Command Palette |
| <kbd>Tab</kbd> / <kbd>Shift+Tab</kbd> | Cycle through active panels |
| <kbd>j</kbd> / <kbd>k</kbd> | Move cursor down / up |
| <kbd>g</kbd> <kbd>m</kbd> | Jump to Merge Requests |
| <kbd>g</kbd> <kbd>i</kbd> | Jump to Issues |
| <kbd>g</kbd> <kbd>b</kbd> | Jump to Kanban Boards |
| <kbd>g</kbd> <kbd>p</kbd> | Jump to CI/CD Pipelines |
| <kbd>p</kbd> | Open Project Quick Switcher |
| <kbd>q</kbd> | Quit application |
