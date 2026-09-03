# Issues & Kanban Boards

Organize, triage, and track project work with `lazygitlab`'s keyboard-centric Issue Inspector and multi-column Sprint Kanban Boards.

![Issue Triage & Detail](../assets/screenshots/issue_detail.svg#only-dark)
![Issue Triage & Detail (Light)](../assets/screenshots/issue_detail-light.svg#only-light)
*Click screenshot to expand in full-resolution lightbox.*

---

## Issue Triage & Management

Inspect issue details, metadata fields, discussions, and linked merge requests without opening a browser.

### Interactive Issue Triage Journey

=== "1. Filter & Search"

    Navigate to the Issues list using <kbd>g</kbd> <kbd>i</kbd>:

    1. Use <kbd>j</kbd> and <kbd>k</kbd> to move through issues in the project.
    2. Press <kbd>/</kbd> to filter issues in real-time by title, author, or labels.
    3. Press <kbd>V</kbd> to toggle grouping by upcoming milestone or target release.
    4. Press <kbd>I</kbd> to toggle the filter between all project issues and items where you are assigned or mentioned.

=== "2. Metadata Inspector"

    Inspect and edit work item details in the multi-tab inspector:

    1. Press <kbd>Tab</kbd> or <kbd>[</kbd> / <kbd>]</kbd> to switch between detail tabs (**Overview**, **Discussions**, **Linked MRs**).
    2. Press <kbd>m</kbd> to focus the **Metadata Inspector** (Assignees, Milestone, Labels, Weight, Due Date).
    3. Press <kbd>d</kbd> to focus the Markdown Description preview.
    4. Press <kbd>e</kbd> or <kbd>Enter</kbd> on a metadata field to edit values directly.

=== "3. Comments & Updates"

    Update issue lifecycle and reply to team comments:

    1. Press <kbd>n</kbd> to launch the **New Issue modal** to create a work item with title, description, and initial labels.
    2. Press <kbd>c</kbd> to close an open issue or reopen a closed issue.
    3. Switch to the **Discussions tab** and press <kbd>r</kbd> to post a new comment or reply.
    4. Press <kbd>y</kbd> to copy the issue reference (e.g. `gitlab-org/project#42`) to your clipboard.

---

### Issue Management Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>n</kbd> | **New Issue** | Create a new issue modal |
| <kbd>c</kbd> | **Close / Reopen** | Toggle open or closed state of selected issue |
| <kbd>/</kbd> | **Search Filter** | Live filter issues by text query or label |
| <kbd>V</kbd> | **Group by Milestone** | Group issue list by milestone or release |
| <kbd>I</kbd> | **Involved Only** | Filter down to issues involving you |
| <kbd>Tab</kbd> / <kbd>[</kbd> / <kbd>]</kbd> | **Cycle Detail Tabs** | Switch between Overview, Discussions, and Linked MRs |
| <kbd>m</kbd> | **Metadata Focus** | Focus Metadata Inspector |
| <kbd>d</kbd> | **Description Focus** | Focus issue description viewer |
| <kbd>e</kbd> / <kbd>Enter</kbd> | **Edit Field** | Edit selected metadata field or description |
| <kbd>r</kbd> | **Reply / Comment** | Post a new comment on the active issue discussion |
| <kbd>y</kbd> | **Copy Reference** | Copy issue markdown reference or URL |

---

## Sprint Kanban Boards

Manage project workflows visually with horizontal column navigation and instant card movement.

![Sprint Kanban Board](../assets/screenshots/kanban.svg#only-dark)
![Sprint Kanban Board (Light)](../assets/screenshots/kanban-light.svg#only-light)
*Sprint Kanban board with multi-column card triage.*

### Interactive Sprint Workflow

=== "1. Board & Columns"

    Press <kbd>g</kbd> <kbd>b</kbd> to open the **Kanban Boards** view:

    1. Use <kbd>h</kbd> and <kbd>l</kbd> (or <kbd>←</kbd> / <kbd>→</kbd>) to switch between board columns (e.g. *Backlog*, *Development*, *In Review*, *Done*).
    2. Use <kbd>j</kbd> and <kbd>k</kbd> (or <kbd>↑</kbd> / <kbd>↓</kbd>) to select cards within the active column.
    3. Notice the card summary: issue number, title, priority badges, weight, and assignees.

=== "Step 2: Move Cards Across Lists"

    Progress issues through your team's workflow stages:

    1. Highlight the card you wish to move.
    2. Press <kbd>L</kbd> to move the card to the **next column (right)**.
    3. Press <kbd>H</kbd> to move the card to the **previous column (left)**.
    4. GitLab labels and board state update remotely in real-time.

=== "Step 3: Deep Inspection from Board"

    Access full issue details directly from the board:

    1. Press <kbd>Enter</kbd> on any card to open the complete issue detail panel.
    2. Review discussion history, linked merge requests, and acceptance criteria.
    3. Press <kbd>Esc</kbd> to return smoothly to the Kanban board layout.

---

### Kanban Board Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>h</kbd> / <kbd>l</kbd> (or <kbd>←</kbd> / <kbd>→</kbd>) | **Navigate Columns** | Switch active list / column left or right |
| <kbd>j</kbd> / <kbd>k</kbd> (or <kbd>↑</kbd> / <kbd>↓</kbd>) | **Select Card** | Move selection up or down within the active column |
| <kbd>H</kbd> / <kbd>L</kbd> | **Move Card** | Move card left or right across workflow columns |
| <kbd>Enter</kbd> | **Inspect Card** | Open full issue detail modal from the board |
| <kbd>o</kbd> | **Open in Browser** | View active board or issue in web browser |
| <kbd>r</kbd> | **Refresh Board** | Refresh all column data from GitLab API |

!!! tip "Sprint Planning Tip"
    Use <kbd>g</kbd> <kbd>b</kbd> during daily standups to quickly visualize work in flight, identify bottlenecks in review columns, and move finished tasks to *Done* with a single keypress (<kbd>L</kbd>).

---

## Related Guides

* **[Personal Dashboard](dashboard.md)**: Consolidated view of assigned issues across all projects.
* **[Code Review & Diffs](code-review.md)**: Inspect merge requests linked to issues.
* **[Keybindings & Cheatsheet](../reference/keybindings.md)**: Full keyboard reference.
