# Repository Explorer & Files

Browse directory structures, inspect source code with syntax highlighting, switch branches, and pivot across GitLab repositories with `lazygitlab`.

![Repository Tree & Syntax Highlighting](../assets/screenshots/repo_tree.svg#only-dark)
![Repository Tree & Syntax Highlighting (Light)](../assets/screenshots/repo_tree-light.svg#only-light)
*Click screenshot to expand in full-resolution lightbox.*

---

## Repository Tree & File Viewer

Explore the project's codebase directly from your terminal with fast asynchronous tree loading and rich syntax highlighting.

### Interactive Repository Browsing Journey

=== "1. Directory Tree"

    Navigate to the Repository Explorer with <kbd>g</kbd> <kbd>r</kbd>:

    1. Use <kbd>j</kbd> and <kbd>k</kbd> to move selection up and down the file hierarchy.
    2. Press <kbd>Enter</kbd> or <kbd>Space</kbd> on a directory node to expand or collapse its subfolders.
    3. Press <kbd>Enter</kbd> on any file to open it in the built-in syntax-highlighted viewer.
    4. Press <kbd>h</kbd> to jump back to parent folder.

=== "2. Syntax Viewer"

    Inspect source code with automatic language detection:

    1. Powered by `syntect`, files render with accurate syntax themes for 100+ programming languages and configuration formats.
    2. Scroll through files using <kbd>j</kbd> / <kbd>k</kbd>, <kbd>Ctrl+D</kbd> / <kbd>Ctrl+U</kbd>, or <kbd>PgDn</kbd> / <kbd>PgUp</kbd>.
    3. Press <kbd>y</kbd> to copy the file contents, selected snippet, or file path to your clipboard.
    4. Press <kbd>o</kbd> to view the file on GitLab web.

=== "3. Switch Branch"

    Pivot your repository view to any branch or tag:

    1. Press <kbd>b</kbd> to open the **Branch & Ref Picker modal**.
    2. Type to fuzzy-filter across local and remote branches (e.g. `feat/auth`, `staging`, `v1.2.0`).
    3. Press <kbd>Enter</kbd> to switch the repository tree to the selected ref instantly.

---

### Repository Explorer Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>Enter</kbd> / <kbd>Space</kbd> | **Expand / Open** | Expand or collapse directory / open file |
| <kbd>b</kbd> | **Switch Branch** | Open branch and tag selector modal |
| <kbd>j</kbd> / <kbd>k</kbd> | **Navigate Tree** | Move up or down in file tree |
| <kbd>Ctrl+D</kbd> / <kbd>Ctrl+U</kbd> | **Page Scroll** | Scroll file preview half page down / up |
| <kbd>y</kbd> | **Copy Path / Content** | Copy file path or content to clipboard |
| <kbd>o</kbd> | **Open in Browser** | Open file on GitLab web |

---

## Project Quick Switcher

Switch seamlessly between any accessible GitLab project in seconds without exiting `lazygitlab`.

![Project Quick Switcher](../assets/screenshots/project_switch.svg#only-dark)
![Project Quick Switcher (Light)](../assets/screenshots/project_switch-light.svg#only-light)
*Instant project quick switcher with real-time fuzzy search.*

### Interactive Project Switching Workflow

=== "1. Summon Switcher (p)"

    Press <kbd>p</kbd> from any view or screen in `lazygitlab`:

    1. The **Project Switcher popup** appears centered on screen.
    2. Focus is placed directly on the search input for instant typing.

=== "Step 2: Fuzzy Filter Across Projects"

    Type any part of the project name or group path:

    1. Substring and fuzzy matching filters your accessible repositories instantly.
    2. Inspect project metadata: group namespace, project name, star count, and visibility (Public / Internal / Private).
    3. Use <kbd>j</kbd> and <kbd>k</kbd> (or <kbd>↑</kbd> / <kbd>↓</kbd>) to highlight target project.

=== "Step 3: Instant Context Pivot"

    Press <kbd>Enter</kbd> on the highlighted project:

    1. `lazygitlab` pivots its active context immediately.
    2. Merge requests, issues, pipelines, branches, and boards refresh in the background without needing a restart.
    3. Press <kbd>Esc</kbd> if you wish to cancel and keep the current project.

---

### Project Switcher Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>p</kbd> | **Open Switcher** | Open Project Quick Switcher modal from anywhere |
| <kbd>j</kbd> / <kbd>k</kbd> (or <kbd>↑</kbd> / <kbd>↓</kbd>) | **Select Project** | Move up or down the project results list |
| <kbd>Enter</kbd> | **Switch Context** | Load selected project into lazygitlab |
| <kbd>Esc</kbd> | **Cancel** | Close switcher and keep current project |

!!! tip "Multi-Project Productivity"
    Working across multiple microservices or libraries? Press <kbd>p</kbd>, type the first few letters of the service name, and hit <kbd>Enter</kbd> to switch tasks in under a second.

---

## Related Guides

* **[Personal Dashboard](dashboard.md)**: Cross-project task overview and unified inbox.
* **[Code Review & Diffs](code-review.md)**: Review merge request diffs across branches.
* **[CI/CD & Job Logs](ci-cd.md)**: Track automated builds for your branches.
* **[Keybindings & Cheatsheet](../reference/keybindings.md)**: Full keyboard reference.
