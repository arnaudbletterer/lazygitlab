# Personal Dashboard

The **Personal Dashboard** is your unified inbox and central command center across all projects in GitLab. It aggregates everything that needs your attention into four distinct sections.

![lazygitlab Personal Dashboard](../assets/screenshots/dashboard.svg#only-dark)
![lazygitlab Personal Dashboard (Light)](../assets/screenshots/dashboard-light.svg#only-light)
*Click screenshot to expand in lightbox.*

---

## Dashboard Sections

### 1. To-Dos (`g t`)

Lists all action items requiring your input: mentions, review requests, issue assignments, and pipeline failures.

* Press <kbd>d</kbd> or <kbd>e</kbd> to mark a To-Do as done.
* Press <kbd>Enter</kbd> to jump straight into the underlying MR or Issue.
* Press <kbd>s</kbd> to snooze a To-Do for later.

### 2. Merge Requests (`g m`)

Displays merge requests relevant to you across GitLab:

* **Review Requests**: MRs where you are designated as a reviewer.
* **Assigned to You**: MRs you are actively authoring or assigned to deliver.
* Use <kbd>[</kbd> / <kbd>]</kbd> to switch quickly between "Review Requests" and "Assigned".

### 3. Issues (`g i`)

Consolidates all issues assigned to you across all repositories:

* Filter by milestone, label, or search query with <kbd>/</kbd>.
* Group issues by milestone with <kbd>M</kbd>.

### 4. Cross-Project Pipelines (`g p`)

Live status of recent CI/CD pipeline runs across your GitLab projects.

* Real-time refresh with adaptive polling.
* Press <kbd>Enter</kbd> to drill into the pipeline's stages and individual jobs.

---

## Helpful Dashboard Shortcuts

| Key | Action |
| --- | --- |
| <kbd>g</kbd> <kbd>t</kbd> | Jump to To-Dos |
| <kbd>g</kbd> <kbd>m</kbd> | Jump to Merge Requests |
| <kbd>g</kbd> <kbd>i</kbd> | Jump to Issues |
| <kbd>g</kbd> <kbd>p</kbd> | Jump to Pipelines |
| <kbd>d</kbd> | Mark selected To-Do as completed |
| <kbd>[</kbd> / <kbd>]</kbd> | Cycle filter segments (e.g. Needs Review / Assigned) |
| <kbd>/</kbd> | Open real-time search filter |
| <kbd>r</kbd> | Force refresh active data |
