<p align="center">
  <img src="assets/logo.svg" alt="lazygitlab logo" width="160" />
</p>

# lazygitlab

<p align="center">
  <strong>A keyboard-driven terminal UI for GitLab.</strong><br>
  <em>Triage merge requests, review diffs, watch pipelines and manage issues without leaving your terminal.</em>
</p>

![lazygitlab Personal Dashboard](assets/screenshots/dashboard.svg#only-dark)
![lazygitlab Personal Dashboard (Light)](assets/screenshots/dashboard-light.svg#only-light)
*Click any screenshot to zoom in full resolution.*

---

## Highlights

=== "Dashboard"
    ![Personal Dashboard](assets/screenshots/dashboard.svg#only-dark)
    ![Personal Dashboard (Light)](assets/screenshots/dashboard-light.svg#only-light)
    Consolidated view of all your assigned To-Dos, Merge Requests awaiting your review, active Issues, and cross-project CI/CD pipelines.

=== "Code Review"
    ![MR Code Review & Diffs](assets/screenshots/mr_diff.svg#only-dark)
    ![MR Code Review & Diffs (Light)](assets/screenshots/mr_diff-light.svg#only-light)
    Full split/unified diffs, syntax highlighting, range comments, approval status, one-key merge, squash, and rebase.

=== "Kanban Boards"
    ![Sprint Kanban Board](assets/screenshots/kanban.svg#only-dark)
    ![Sprint Kanban Board (Light)](assets/screenshots/kanban-light.svg#only-light)
    Multi-column Kanban boards with fast issue drag-and-drop, label assignment, milestone filters, and epic tracking.

=== "CI/CD Pipelines"
    ![CI/CD Live Job Logs](assets/screenshots/job_log.svg#only-dark)
    ![CI/CD Live Job Logs (Light)](assets/screenshots/job_log-light.svg#only-light)
    Monitor live pipelines, inspect colorized job logs with ANSI support, section collapsing, and search.

=== "Repository"
    ![Repository Tree & File Viewer](assets/screenshots/repo_tree.svg#only-dark)
    ![Repository Tree & File Viewer (Light)](assets/screenshots/repo_tree-light.svg#only-light)
    Browse repository directory trees, switch branches on the fly, and inspect file contents with syntect syntax highlighting.

---

## Quick Start

```bash
# 1. Install from source (Rust 1.85 or newer)
cargo install --git https://github.com/arnaudbletterer/lazygitlab

# 2. Configure credentials
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
export GITLAB_URL="https://gitlab.com"

# 3. Launch (auto-connects to the current git repository)
lazygitlab
```

See the complete **[Getting Started Guide](getting-started.md)** for configuration options and self-hosted instances.

---

## Navigation & Shortcuts

| Key | Action |
| --- | --- |
| <kbd>?</kbd> | Open Interactive Help / Command Palette |
| <kbd>Tab</kbd> / <kbd>Shift+Tab</kbd> | Cycle through active panels |
| <kbd>j</kbd> / <kbd>k</kbd> | Move cursor down / up |
| <kbd>g</kbd> <kbd>t</kbd> | Jump to To-Dos |
| <kbd>g</kbd> <kbd>m</kbd> | Jump to Merge Requests |
| <kbd>g</kbd> <kbd>i</kbd> | Jump to Issues |
| <kbd>g</kbd> <kbd>b</kbd> | Jump to Kanban Boards |
| <kbd>g</kbd> <kbd>p</kbd> | Jump to CI/CD Pipelines |
| <kbd>g</kbd> <kbd>r</kbd> | Jump to Repository Tree |
| <kbd>p</kbd> | Open Project Quick Switcher |
| <kbd>q</kbd> | Quit application |

See the full **[Keybindings Reference & Cheatsheet](reference/keybindings.md)** for all shortcut tables and printable PDF cheatsheet.
