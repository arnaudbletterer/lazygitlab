# Personal Dashboard

The **Personal Dashboard** is your unified inbox and central command center across all projects in GitLab. It aggregates everything requiring your attention into four distinct, high-signal views.

![lazygitlab Personal Dashboard](../assets/screenshots/dashboard.svg#only-dark)
![lazygitlab Personal Dashboard (Light)](../assets/screenshots/dashboard-light.svg#only-light)
*Click screenshot to expand in full-resolution lightbox.*

---

## Daily Triage Workflow

Process your daily GitLab notifications and tasks in seconds without leaving your terminal:

=== "1. To-Dos"

    Jump directly to your Personal To-Dos with <kbd>g</kbd> <kbd>t</kbd>:

    1. Use <kbd>j</kbd> and <kbd>k</kbd> to move through your incoming mentions, review requests, and notifications.
    2. Press <kbd>Enter</kbd> on any To-Do to jump straight into the underlying Merge Request, Issue, or comment thread.
    3. Press <kbd>d</kbd> to mark the To-Do as done (completed).
    4. Press <kbd>s</kbd> to snooze items you want to handle later.

=== "2. Merge Requests"

    Jump to your global Merge Requests queue with <kbd>g</kbd> <kbd>m</kbd>:

    1. Switch between **"Review Requests"** (MRs awaiting your feedback) and **"Assigned to You"** (MRs you are authoring) using <kbd>[</kbd> and <kbd>]</kbd>.
    2. Inspect status indicators: pipeline health, merge conflict flags, approval counts, and draft status.
    3. Press <kbd>d</kbd> or <kbd>Enter</kbd> to jump straight into the full Diff Viewer.

=== "3. Issues"

    Jump to your cross-project Issues view with <kbd>g</kbd> <kbd>i</kbd>:

    1. Press <kbd>/</kbd> to filter issues in real-time by title, label, or assignee.
    2. Press <kbd>V</kbd> to toggle grouping by milestone or repository.
    3. Press <kbd>I</kbd> to restrict the list strictly to involved items.
    4. Press <kbd>c</kbd> to close or reopen an issue directly from the list.

=== "4. CI/CD"

    Jump to the cross-project Pipelines monitor with <kbd>g</kbd> <kbd>p</kbd>:

    1. View real-time status pills across all recent builds (`[ PASSED ]`, `[ FAILED ]`, `[ ⏳ RUNNING ]`).
    2. Press <kbd>Enter</kbd> to drill into the pipeline's stages and individual jobs.
    3. Press <kbd>R</kbd> to immediately retry a failed pipeline, or <kbd>C</kbd> to cancel a running build.

---

## Deep Dive: Dashboard Sections

### 1. Unified To-Dos Inbox

The To-Do inbox consolidates notifications from all GitLab projects into one clean action queue.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>Enter</kbd> | **Inspect / Open** | Drill directly into the linked MR, issue, or discussion |
| <kbd>d</kbd> | **Mark Done** | Remove action item from your To-Do queue on GitLab |
| <kbd>s</kbd> | **Snooze** | Temporarily dismiss item from active list |
| <kbd>o</kbd> | **Open in Browser** | Open the notification URL in your default web browser |
| <kbd>y</kbd> | **Copy URL** | Copy the item URL to system clipboard |

!!! tip "Zero-Inbox Productivity"
    Use <kbd>j</kbd> / <kbd>k</kbd> + <kbd>Enter</kbd> to review items, and press <kbd>d</kbd> as soon as you have taken action. Your To-Do list updates remotely in real-time.

---

### 2. Merge Requests Queue

Never lose track of code reviews waiting on you or branches you need to merge.

* **Review Requests**: Pulls MRs across all repositories where team members have assigned you as a reviewer.
* **Assigned to You**: MRs where you are the primary assignee responsible for delivery.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>[</kbd> / <kbd>]</kbd> | **Switch View** | Toggle between "Review Requests" and "Assigned" |
| <kbd>d</kbd> / <kbd>Enter</kbd> | **Open Diff** | Enter side-by-side or unified diff reviewer |
| <kbd>A</kbd> / <kbd>R</kbd> | **Approve / Unapprove** | Cast or revoke approval directly from list |
| <kbd>P</kbd> | **Head Pipeline** | View CI/CD jobs for the MR's latest commit |
| <kbd>Y</kbd> | **Copy Branch** | Copy source branch name (e.g. `feat/auth-service`) |

---

### 3. Cross-Project Issues

Aggregates all open issues assigned to you across all repositories in your organization.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>/</kbd> | **Search Filter** | Live substring filter across titles, descriptions, and labels |
| <kbd>V</kbd> | **Group by Milestone** | Organize issues by upcoming release milestones |
| <kbd>I</kbd> | **Involved Filter** | Toggle between all accessible issues vs. items where you are directly involved |
| <kbd>n</kbd> | **New Issue** | Create a new issue modal in the selected repository |
| <kbd>c</kbd> | **Close / Reopen** | Toggle open/closed state of the selected issue |

---

### 4. CI/CD Pipelines Monitor

Monitors live pipeline execution across projects with adaptive polling that backs off automatically when builds complete.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>Enter</kbd> | **View Jobs** | Expand pipeline stages and drill into live job logs |
| <kbd>R</kbd> | **Retry Pipeline** | Re-run failed pipeline stages |
| <kbd>C</kbd> | **Cancel Pipeline** | Abort a running pipeline |
| <kbd>S</kbd> | **Pipeline Schedules** | View scheduled runs; press <kbd>Enter</kbd> to execute immediately |
| <kbd>v</kbd> | **CI Variables** | Inspect and manage project CI environment variables |

---

## Global Navigation Shortcuts

| Shortcut | Destination / Action |
| :--- | :--- |
| <kbd>?</kbd> | Show interactive Help & Command Palette |
| <kbd>p</kbd> | Open Project Quick Switcher modal |
| <kbd>f</kbd> | Trigger Global Search across GitLab |
| <kbd>Tab</kbd> / <kbd>Shift+Tab</kbd> | Cycle focus between sidebar menu, list, and detail view |
| <kbd>r</kbd> | Force instant refresh of current view |
| <kbd>q</kbd> | Quit lazygitlab |

---

## Related Guides

* **[Code Review & Diffs](code-review.md)**: Deep dive into keyboard-driven MR reviews, split diffs, and range comments.
* **[Issues & Kanban Boards](issues-and-boards.md)**: Sprint planning, column drag-and-drop, and issue triage.
* **[CI/CD & Job Logs](ci-cd.md)**: Real-time ANSI log streaming and runner section collapsing.
