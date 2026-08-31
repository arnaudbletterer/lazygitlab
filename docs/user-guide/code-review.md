# Code Review & Diffs

`lazygitlab` turns Merge Request reviews into a lightning-fast, keyboard-driven experience directly within your terminal workspace.

![MR Code Review & Diffs](../assets/screenshots/mr_diff.svg#only-dark)
![MR Code Review & Diffs (Light)](../assets/screenshots/mr_diff-light.svg#only-light)
*Click screenshot to expand in full-resolution lightbox.*

---

## ⚡ Interactive Review Lifecycle

Review merge requests efficiently using this 4-step workflow:

=== "1. Overview & Commits"

    From the Merge Request list (<kbd>g</kbd> <kbd>m</kbd>):

    1. Highlight any MR with <kbd>j</kbd> and <kbd>k</kbd>.
    2. Inspect the summary panel: pipeline status, approval rules, target/source branches, and reviewers.
    3. Press <kbd>Tab</kbd> or <kbd>[</kbd> / <kbd>]</kbd> to switch between detail tabs (**Overview**, **Discussions**, **Commits**, **Changes**).
    4. Press <kbd>P</kbd> to inspect the latest CI/CD pipeline jobs for this MR.
    5. Press <kbd>Y</kbd> to copy the source branch name to your clipboard for local testing.

=== "2. Split & Unified Diffs"

    Press <kbd>d</kbd> or <kbd>Enter</kbd> to open the **Diff Viewer**:

    1. Switch between files using <kbd>]</kbd> (next file) and <kbd>[</kbd> (previous file).
    2. Press <kbd>v</kbd> to toggle between **side-by-side split diff** and **unified diff** mode.
    3. Press <kbd>Space</kbd> or <kbd>e</kbd> to collapse or expand large files.
    4. Scroll through lines smoothly with <kbd>j</kbd> and <kbd>k</kbd>, or jump hunks with <kbd>Ctrl+D</kbd> / <kbd>Ctrl+U</kbd>.

=== "3. Inline Comments"

    Add precise feedback directly on changed lines:

    1. Navigate to the target line in the diff.
    2. Press <kbd>Shift+j</kbd> / <kbd>Shift+k</kbd> (or <kbd>v</kbd>) to highlight a multi-line range.
    3. Press <kbd>c</kbd> to open the multiline comment editor on the selected range.
    4. Write your feedback and press <kbd>Ctrl+S</kbd> to submit (or <kbd>Esc</kbd> to discard).
    5. Navigate existing review threads with <kbd>n</kbd> (next) and <kbd>N</kbd> (previous), reply with <kbd>r</kbd>, or resolve with <kbd>s</kbd>.

=== "4. Approvals & Merging"

    Finalize and merge the MR once discussions are resolved:

    1. Press <kbd>A</kbd> to cast your official **Approval** (or <kbd>R</kbd> to revoke).
    2. Press <kbd>D</kbd> to toggle the **Draft** prefix (`Draft: ...`) when ready.
    3. Press <kbd>E</kbd> to trigger a remote fast-forward **Rebase** against target branch.
    4. Press <kbd>M</kbd> to launch the **Merge Confirmation Dialog**. Configure squash commits and branch deletion, then confirm with <kbd>Enter</kbd>.

---

## 🔍 Feature Deep Dives

### 1. High-Performance Diff Viewer

The diff viewer features full terminal syntax highlighting, intelligent hunk folding, and side-by-side diffing.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>h</kbd> / <kbd>l</kbd> / <kbd>Enter</kbd> | **Focus Switch** | Switch focus between changed files list and diff lines |
| <kbd>v</kbd> | **Split / Unified** | Toggle side-by-side split diff and unified single-column view |
| <kbd>[</kbd> / <kbd>]</kbd> | **Prev / Next File** | Jump immediately to the previous / next changed file |
| <kbd>e</kbd> / <kbd>Space</kbd> | **Toggle Fold** | Collapse or expand current file diff |
| <kbd>y</kbd> | **Copy Snippet** | Copy selected diff line(s) to system clipboard |

!!! tip "Large MR Navigation"
    For large merge requests with dozens of files, use the file list on the left (<kbd>h</kbd>) to quickly jump between modified paths without scrolling through thousands of diff lines.

---

### 2. Discussions & Inline Range Reviews

Engage in discussions without context switching to the browser.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>c</kbd> | **Inline Comment** | Add comment on current line or active range selection |
| <kbd>C</kbd> | **File Comment** | Add a top-level comment on the active file |
| <kbd>Shift+j/k</kbd> | **Range Select** | Extend selection over multiple lines for range comments |
| <kbd>r</kbd> | **Reply** | Reply to the active inline discussion thread |
| <kbd>s</kbd> | **Resolve Thread** | Toggle resolved/unresolved status of active thread |
| <kbd>n</kbd> / <kbd>N</kbd> | **Next / Prev Thread** | Jump to next or previous discussion in diff |
| <kbd>Space</kbd> | **Fold Thread** | Collapse or expand discussion thread body |

!!! note "Comment Editor Keybindings"
    Inside the multiline comment popup:
    
    * Press <kbd>Ctrl+S</kbd> or <kbd>Ctrl+Enter</kbd> to submit your comment to GitLab.
    * Press <kbd>Esc</kbd> to cancel and close the modal.

---

### 3. MR Actions & Merge Operations

Manage the complete MR lifecycle from review to landing into `main`.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>A</kbd> | **Approve MR** | Add your approval to the MR approval rules |
| <kbd>R</kbd> | **Unapprove MR** | Revoke your approval |
| <kbd>M</kbd> | **Merge MR** | Open Merge modal (Squash commits, Delete source branch) |
| <kbd>D</kbd> | **Toggle Draft** | Mark MR as Draft or Ready for Review |
| <kbd>E</kbd> | **Rebase** | Fast-forward rebase source branch on GitLab remote |
| <kbd>S</kbd> | **Toggle Squash** | Toggle default squash setting on merge |
| <kbd>c</kbd> | **Close / Reopen** | Toggle open or closed state of the MR |
| <kbd>n</kbd> | **New MR** | Open creation modal to submit a new Merge Request |

---

## 📚 Related Guides

* **[Personal Dashboard](dashboard.md)**: Review requests inbox and notification triage.
* **[CI/CD & Job Logs](ci-cd.md)**: Inspect pipeline checks before merging.
* **[Keybindings & Cheatsheet](../reference/keybindings.md)**: Full keyboard shortcut reference.
