# lazygitlab Enhancement Roadmap

## Objective
To expand `lazygitlab` from a triage and monitoring tool into a comprehensive TUI replacement for the GitLab web interface, providing deep collaboration, repository management, and project tracking capabilities.

## Design Philosophy (TUI Best Practices)
To ensure the application remains "easy to use and coherent with the spirit of the best TUIs," all new features must adhere to the following principles:
*   **Vim-inspired Navigation:** Consistent use of `h/j/k/l` for movement, `/` for search, and modal interactions where appropriate.
*   **Keyboard-Centric:** Every action must be accessible without a mouse. Complex actions should use mnemonic key chords or a command palette (e.g., `?` for help, `:` for commands).
*   **Speed and Responsiveness:** The UI must never block the main thread. Data fetching and processing must remain asynchronous.
*   **Information Density:** Maximize the use of terminal real estate without clutter. Use collapsible sections and focused views (like Zen mode) for deep work.
*   **Contextual Help:** Always display relevant keybindings at the bottom of the screen based on the active view.

---

## Phased Implementation Plan

### Phase 1: Core Developer Workflows (High ROI)
Focus on the most frequent tasks developers perform after pushing code.

*   **Multi-line Text Editor:**
    *   *Implementation:* Integrate a multi-line editing component (e.g., `tui-textarea` or similar) to replace the single-line input.
    *   *UX:* Support basic editor commands (arrows, backspace, enter) and an easy way to save (`Ctrl+s` or `Esc` to exit insert mode then `w`).
*   **Advanced MR Actions:**
    *   *Implementation:* Add API calls and UI bindings for "Approve," "Revoke Approval," and toggling "Draft/Ready."
    *   *UX:* Add to the MR Detail view shortcuts (e.g., `A` to approve, `D` to toggle draft).
*   **Inline Code Review:**
    *   *Implementation:* Map discussion threads to specific lines in the `MRDiff` model.
    *   *UX:* In the diff view, allow users to select a line and press `c` to open the multi-line editor to leave a targeted comment.

### Phase 2: Repository Exploration
Provide visibility into the code itself without leaving the terminal.

*   **File Tree Browser:**
    *   *Implementation:* Fetch the repository tree via GitLab API. Build a tree-view widget.
    *   *UX:* Left panel for the directory tree, right panel for file content rendering (with syntax highlighting via `syntect` if possible).
*   **History & Branches:**
    *   *Implementation:* Fetch commit logs and branch lists.
    *   *UX:* Dedicated `Branches` and `Commits` menu items under the Project context. Support for creating/deleting branches via popup prompts.

### Phase 3: Project Management & Visibility
Elevate the tool for leads and maintainers.

*   **Kanban Boards (TUI Style):**
    *   *Implementation:* Fetch board configurations and issue lists per board list.
    *   *UX:* Render horizontal lists. Use `h/l` to move between columns, `j/k` to select issues. Use `<Shift+h/l>` to move issues between columns (change labels).
*   **Extended Entity Support:**
    *   *Implementation:* Fetch and display Epics, Related Issues, and Linked MRs in the Detail view metadata section.

### Phase 4: Architecture & UX Polish
Refine the foundation for scale.

*   **Global Search:**
    *   *Implementation:* Implement the GitLab global search API.
    *   *UX:* A persistent shortcut (e.g., `/` from the dashboard) to open a global search palette, displaying mixed results (MRs, Issues, Projects).
*   **Performance & Real-time Data:**
    *   *Implementation:* Investigate transitioning from polling to GraphQL subscriptions or Server-Sent Events (SSE) if supported by the GitLab instance, or optimize polling to back off when idle.
*   **Richer Markdown Rendering:**
    *   *Implementation:* Improve the `markdown.rs` parser to handle GFM tables, checklists, and blockquotes more gracefully within terminal constraints.

---

## Technical Considerations
*   **Rate Limiting:** As we add more features (like file trees and inline comments), API calls will increase. We must implement robust caching and respect rate limits.
*   **Crate Dependencies:** Evaluate new dependencies carefully (e.g., for multi-line editing or tree views) to avoid bloating the binary or complicating cross-platform builds.

## Verification
*   Each phase must include unit tests for new models/API calls.
*   UI testing via `ratatui::backend::TestBackend` must be updated to cover new interactive components (especially the multi-line editor and board views).
