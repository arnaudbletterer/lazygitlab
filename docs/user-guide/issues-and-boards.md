# Issues & Kanban Boards

Organize, track, and triage your work efficiently using `lazygitlab`'s Issue management and multi-column Kanban board views.

---

## 📋 Issue Triage & Management (`g i`)

Inspect issue metadata, discussion threads, linked merge requests, milestones, and labels.

![Issue Triage & Detail](../assets/screenshots/issue_detail.svg#only-dark)
![Issue Triage & Detail (Light)](../assets/screenshots/issue_detail-light.svg#only-light)
*Click screenshot to expand in lightbox.*

### Key Actions on Issues:

* **Create Issue**: Press <kbd>n</kbd> to open the New Issue modal (title, description, labels, milestone).
* **Close / Reopen**: Press <kbd>c</kbd> to close an open issue or reopen a closed issue.
* **Assignees**: Press <kbd>a</kbd> or navigate to the Assignees field in the Metadata Inspector to assign team members.
* **Labels**: Press <kbd>l</kbd> to open the label selector popup.
* **Milestones**: Press <kbd>m</kbd> to assign the issue to a milestone.
* **Discussions & Comments**: Switch to the Discussions tab (`Tab` / `]`) to read threads or reply with <kbd>r</kbd>.

---

## 📊 Sprint Kanban Boards (`g b`)

Visual Kanban board view with horizontal column scrolling and instant card drag-and-drop.

![Sprint Kanban Board](../assets/screenshots/kanban.svg#only-dark)
![Sprint Kanban Board (Light)](../assets/screenshots/kanban-light.svg#only-light)
*Click screenshot to expand in lightbox.*

### Kanban Navigation:

* **Switch Lists (Columns)**: Use <kbd>h</kbd> and <kbd>l</kbd> (or <kbd>←</kbd> / <kbd>→</kbd>) to navigate between columns (e.g. Backlog, Doing, In Review, Done).
* **Select Cards**: Use <kbd>j</kbd> and <kbd>k</kbd> (or <kbd>↑</kbd> / <kbd>↓</kbd>) to navigate through issues in the active list.
* **Move Issue Across Lists**: Press <kbd>H</kbd> to move an issue left, or <kbd>L</kbd> to move an issue right. The label assignment on GitLab updates immediately.
* **Inspect Details**: Press <kbd>Enter</kbd> on any card to view its full description and metadata panel.

---

💡 *For the complete keymap list, see the [Keybindings & Cheatsheet](../reference/keybindings.md).*
