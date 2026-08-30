# Code Review & Diffs

`lazygitlab` turns merge request code reviews into a fluid, keyboard-driven experience without leaving your terminal editor workflow.

![MR Code Review & Diffs](../assets/screenshots/mr_diff.svg#only-dark)
![MR Code Review & Diffs (Light)](../assets/screenshots/mr_diff-light.svg#only-light)
*Click screenshot to expand in lightbox.*

---

## Code Review Workflow

### 1. Inspecting Changes (`Enter` or `d`)

From the MR list or detail panel, press <kbd>Enter</kbd> (or <kbd>d</kbd>) to open the **Diff Viewer**:

* Jump between changed files using <kbd>]</kbd> and <kbd>[</kbd>.
* Collapse / expand individual file diffs with <kbd>Space</kbd>.
* Toggle between side-by-side split view and unified view with <kbd>v</kbd>.

### 2. Adding Inline Comments & Range Reviews (`c` / `v`)

* Navigate cursor to the desired line and press <kbd>c</kbd> to open the multiline comment editor.
* Press <kbd>v</kbd> to initiate a multi-line selection range, move with <kbd>j</kbd>/<kbd>k</kbd>, and press <kbd>c</kbd> to comment on the selected block.
* Submit with <kbd>Ctrl+S</kbd> or discard with <kbd>Esc</kbd>.

### 3. Approvals & Merge Actions

* **Approve MR**: Press <kbd>a</kbd> to cast your approval. The approvals badge (`[ ✔ APPROVED ]`) updates instantly.
* **Unapprove MR**: Press <kbd>u</kbd> to revoke your approval if more changes are required.
* **Merge MR**: Press <kbd>M</kbd> (or <kbd>m</kbd>) to open the Merge Confirmation modal with options for squashing commits and deleting the source branch.
* **Rebase**: Press <kbd>R</kbd> to trigger a remote fast-forward rebase against the target branch.
* **Draft / Ready**: Press <kbd>t</kbd> to toggle the Draft status (`Draft: ...`).

---

## Code Review Shortcuts Cheatsheet

| Key | Action |
| --- | --- |
| <kbd>Enter</kbd> / <kbd>d</kbd> | Open Diff View for selected MR |
| <kbd>j</kbd> / <kbd>k</kbd> | Scroll line down / up |
| <kbd>]</kbd> / <kbd>[</kbd> | Jump to next / previous changed file |
| <kbd>Space</kbd> | Toggle collapse on current file |
| <kbd>c</kbd> | Post inline comment on current line or range |
| <kbd>v</kbd> | Start visual line range selection |
| <kbd>a</kbd> | Approve Merge Request |
| <kbd>u</kbd> | Unapprove Merge Request |
| <kbd>m</kbd> / <kbd>M</kbd> | Open Merge dialog (with squash / delete branch options) |
| <kbd>R</kbd> | Trigger remote Rebase |
| <kbd>t</kbd> | Toggle Draft (WIP) status |
| <kbd>o</kbd> | Open MR in web browser |
| <kbd>Esc</kbd> | Return to MR List view |
