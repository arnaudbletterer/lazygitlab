# Getting Started

Welcome to **lazygitlab**! Get up and running with a lightning-fast, keyboard-driven GitLab terminal interface in less than two minutes.

![lazygitlab Command Palette](assets/screenshots/help_palette.svg#only-dark)
![lazygitlab Command Palette (Light)](assets/screenshots/help_palette-light.svg#only-light)
*Press <kbd>?</kbd> anywhere in lazygitlab to open the interactive Command Palette & Keymap Finder.*

---

## Installation

Choose the installation method that best fits your workflow:

=== "Cargo"

    Requires Rust 1.85 or newer:

    ```bash
    cargo install --git https://github.com/arnaudbletterer/lazygitlab
    ```

=== "Source"

    ```bash
    git clone https://github.com/arnaudbletterer/lazygitlab.git
    cd lazygitlab
    cargo build --release
    ```

    The binary is `target/release/lazygitlab`.

=== "Other"

    Not available yet: crates.io, release binaries, Homebrew (`arnaudbletterer/tap`), AUR (`lazygitlab-bin`) and Scoop. See the README for their status.

---

## Authentication & Tokens

`lazygitlab` connects securely to GitLab using a **Personal Access Token** (or Project Access Token).

!!! info "Required Token Scopes"
    When generating your token in GitLab (**User Settings → Access Tokens**), use these scopes:

    * `read_api` is the minimum for read-only browsing.
    * `api` is needed for actions such as approving, merging, commenting and editing.

### Configuration Methods

=== "Environment Variables"

    The quickest way to configure credentials is by exporting environment variables in your shell profile (`~/.bashrc` or `~/.zshrc`). `.env` files are not read:

    ```bash
    export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
    export GITLAB_URL="https://gitlab.com" # Or your organization's self-hosted URL
    ```

=== "Config File (config.toml)"

    For persistent settings across sessions, create a configuration file at `~/.config/lazygitlab/config.toml` (Linux/macOS) or `%USERPROFILE%\.config\lazygitlab\config.toml` (Windows):

    ```toml
    [gitlab]
    url = "https://gitlab.com"
    token = "glpat-xxxxxxxxxxxxxxxxxxxx"
    # insecure = true
    # project = "group/project"
    ```

=== "CLI Flags"

    You can also pass credentials dynamically via command-line arguments:

    ```bash
    lazygitlab --url "https://gitlab.example.com" --token "glpat-xxxxxx"
    ```

!!! tip "Self-Hosted Instances and Self-Signed Certificates"
    Prefer adding your CA to the system trust store. If that is not possible, pass `--insecure` or set `insecure = true` under `[gitlab]` in `config.toml`. This skips TLS certificate verification, prints a warning at every start and shows `TLS NOT VERIFIED` in the header:
    ```bash
    lazygitlab --url https://gitlab.internal.corp --insecure
    ```

    For a plain HTTP instance, use an `http://` URL; no flag is needed.

---

## Interactive Launch Journey

Follow this 4-step onboarding flow to start using `lazygitlab` immediately:

=== "1. Local Git Repo"

    Run it inside a local Git repository:

    ```bash
    cd ~/Projects/backend-api
    lazygitlab
    ```

    `lazygitlab` shows the current local branch and a dirty marker. It does not detect the project from the remote; pick it with `p` or pass `--project`.

=== "2. Target Project (--project)"

    Open any repository directly by providing its path or numeric ID without switching directories:

    ```bash
    # Open by project path
    lazygitlab --project my-organization/backend-api

    # Open by numeric project ID
    lazygitlab --project 81221057
    ```

=== "3. Personal Dashboard"

    Launch `lazygitlab` outside of any git repository to enter the unified **Personal Dashboard**:

    ```bash
    cd ~
    lazygitlab
    ```

    This consolidates all your pending To-Dos, assigned Merge Requests, review requests, and active issues across the entire GitLab organization.

=== "4. Command Palette (?)"

    Press <kbd>?</kbd> at any time to open the **Command Palette & Shortcut Finder**. Type to filter actions, or use the global navigation shortcuts below.

---

## Essential First Shortcuts

| Shortcut | Destination / Action | Description |
| :--- | :--- | :--- |
| <kbd>?</kbd> | **Command Palette** | Open interactive help and keymap finder |
| <kbd>Tab</kbd> / <kbd>Shift+Tab</kbd> | **Cycle Panels** | Switch focus between sidebar, item list, and detail view |
| <kbd>j</kbd> / <kbd>k</kbd> | **Navigate** | Move cursor down / up |
| <kbd>g</kbd> <kbd>t</kbd> | **Personal To-Dos** | Jump to unified action items inbox |
| <kbd>g</kbd> <kbd>m</kbd> | **Merge Requests** | Jump to Merge Requests view |
| <kbd>g</kbd> <kbd>i</kbd> | **Issues** | Jump to Issues triage view |
| <kbd>g</kbd> <kbd>b</kbd> | **Kanban Boards** | Jump to sprint Kanban boards |
| <kbd>g</kbd> <kbd>p</kbd> | **CI/CD Pipelines** | Jump to pipeline overview & job logs |
| <kbd>g</kbd> <kbd>r</kbd> | **Repository Explorer** | Browse file tree and switch branches |
| <kbd>p</kbd> | **Project Switcher** | Quick-switch to any project |
| <kbd>q</kbd> | **Quit** | Exit lazygitlab |

---

## Next Steps

Dive deeper into specific workflows with our dedicated user guides:

* **[Personal Dashboard](user-guide/dashboard.md)**: Manage your unified inbox, review queue, and daily action items.
* **[Code Review & Diffs](user-guide/code-review.md)**: Master split/unified diffs, range comments, approvals, and one-key merge workflows.
* **[Issues & Kanban Boards](user-guide/issues-and-boards.md)**: Triage issues and move cards seamlessly across sprint boards.
* **[CI/CD & Job Logs](user-guide/ci-cd.md)**: Monitor live pipelines and inspect streaming job output with ANSI colors.
* **[Repository Explorer](user-guide/repository.md)**: Explore directory trees, view files with syntax highlighting, and switch branches.
* **[Keybindings & Cheatsheet](reference/keybindings.md)**: Download the printable 1-page PDF cheatsheet.
