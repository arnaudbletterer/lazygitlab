# Keybindings Reference

`lazygitlab` is fully keyboard-driven, adhering to ergonomic Vim-style navigation conventions and quick jump shortcuts.

Press <kbd>?</kbd> at any point in the application to open the **Interactive Help & Command Palette**:

![Interactive Help & Command Palette](../assets/screenshots/help_palette.svg#only-dark)
![Interactive Help & Command Palette (Light)](../assets/screenshots/help_palette-light.svg#only-light)
*Click screenshot to expand in lightbox.*

---

## 🌐 Global Navigation & Context

| Key | Action |
| --- | --- |
| <kbd>?</kbd> | Open / Close Help & Command Palette |
| <kbd>Tab</kbd> / <kbd>Shift+Tab</kbd> | Cycle through active panels forward / backward |
| <kbd>j</kbd> / <kbd>k</kbd> (or <kbd>↓</kbd>/<kbd>↑</kbd>) | Navigate cursor item down / up |
| <kbd>h</kbd> / <kbd>l</kbd> (or <kbd>←</kbd>/<kbd>→</kbd>) | Switch columns or tabbed detail sections |
| <kbd>Ctrl+D</kbd> / <kbd>Ctrl+U</kbd> | Page scroll half-screen down / up |
| <kbd>g</kbd> <kbd>t</kbd> | Jump to **To-Dos** |
| <kbd>g</kbd> <kbd>m</kbd> | Jump to **Merge Requests** |
| <kbd>g</kbd> <kbd>i</kbd> | Jump to **Issues** |
| <kbd>g</kbd> <kbd>b</kbd> | Jump to **Kanban Boards** |
| <kbd>g</kbd> <kbd>p</kbd> | Jump to **Pipelines** |
| <kbd>g</kbd> <kbd>r</kbd> | Jump to **Repository Tree** |
| <kbd>p</kbd> | Open **Project Quick Switcher** |
| <kbd>/</kbd> | Open real-time filter search bar |
| <kbd>r</kbd> | Force refresh current view data |
| <kbd>o</kbd> | Open current item in web browser |
| <kbd>q</kbd> | Quit application |

---

## 🔀 Merge Requests & Code Review

| Key | Action |
| --- | --- |
| <kbd>Enter</kbd> / <kbd>d</kbd> | Open Diff view |
| <kbd>]</kbd> / <kbd>[</kbd> | Jump to next / previous changed file |
| <kbd>Space</kbd> | Toggle collapse on current file |
| <kbd>c</kbd> | Add inline comment on line / visual range |
| <kbd>v</kbd> | Start visual line range selection |
| <kbd>a</kbd> | Approve Merge Request |
| <kbd>u</kbd> | Unapprove Merge Request |
| <kbd>m</kbd> / <kbd>M</kbd> | Open Merge dialog (squash & delete branch options) |
| <kbd>R</kbd> | Trigger remote fast-forward rebase |
| <kbd>t</kbd> | Toggle Draft (WIP) status |
| <kbd>n</kbd> | Create new Merge Request |
| <kbd>x</kbd> | Close / Reopen Merge Request |

---

## 📋 Issues & Kanban Boards

| Key | Action |
| --- | --- |
| <kbd>n</kbd> | Create new issue |
| <kbd>x</kbd> | Close / Reopen issue |
| <kbd>l</kbd> | Open Label assignment picker |
| <kbd>m</kbd> | Open Milestone assignment picker |
| <kbd>a</kbd> | Open Assignee assignment picker |
| <kbd>M</kbd> | Toggle grouping issues by Milestone |
| <kbd>h</kbd> / <kbd>l</kbd> | Switch Kanban column left / right |
| <kbd>H</kbd> / <kbd>L</kbd> | Move selected issue card to previous / next Kanban list |

---

## 🚀 CI/CD & Live Logs

| Key | Action |
| --- | --- |
| <kbd>Enter</kbd> | Inspect pipeline stages / Open job log |
| <kbd>n</kbd> | Trigger new pipeline run |
| <kbd>r</kbd> | Retry failed job / pipeline |
| <kbd>x</kbd> | Cancel running pipeline |
| <kbd>S</kbd> | Open Pipeline Schedules popup |
| <kbd>V</kbd> | Open CI Variables management popup |
| <kbd>/</kbd> | Search text in active Job Log |
| <kbd>Space</kbd> | Toggle collapsible runner section |
| <kbd>y</kbd> | Copy job log to clipboard |
