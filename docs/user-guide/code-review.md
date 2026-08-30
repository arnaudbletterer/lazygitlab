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

* **Approve MR**: Press <kbd>A</kbd> to cast your approval. The approvals badge (`[ ✔ APPROVED ]`) updates instantly.
* **Unapprove MR**: Press <kbd>R</kbd> to revoke your approval if more changes are required.
* **Merge MR**: Press <kbd>M</kbd> to open the Merge Confirmation modal with options for squashing commits and deleting the source branch.
* **Rebase**: Press <kbd>E</kbd> to trigger a remote fast-forward rebase against the target branch.
* **Draft / Ready**: Press <kbd>D</kbd> to toggle the Draft status (`Draft: ...`).
* **Squash Toggle**: Press <kbd>S</kbd> to toggle squash commit behavior on merge.

---

💡 *For the complete keymap list, see the [Keybindings & Cheatsheet](../reference/keybindings.md).*
