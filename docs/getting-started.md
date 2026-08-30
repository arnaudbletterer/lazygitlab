# Getting Started

Get up and running with `lazygitlab` in less than two minutes.

---

## 📦 Installation

### Using Cargo (Recommended)

The easiest and most common way to install `lazygitlab` is via `cargo`:

```bash
cargo install lazygitlab
```

To upgrade to the latest version:
```bash
cargo install lazygitlab --force
```

### From Source

Ensure you have Rust 1.75+ installed:

```bash
git clone https://github.com/abletterer/lazygitlab.git
cd lazygitlab
cargo build --release
sudo cp target/release/lazygitlab /usr/local/bin/
```

---

## 🔑 Authentication & Configuration

`lazygitlab` requires a GitLab Personal Access Token (or Project Access Token) with **`api`** and **`read_user`** scopes.

### Method 1: Environment Variables (Fastest)

Set your token and URL in your shell configuration (`~/.bashrc`, `~/.zshrc`, or `.env`):

```bash
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
export GITLAB_URL="https://gitlab.com" # Or your self-hosted GitLab instance URL
```

### Method 2: Configuration File (`config.toml`)

Alternatively, save your credentials in `~/.config/lazygitlab/config.toml` (or `%APPDATA%\lazygitlab\config.toml` on Windows):

```toml
gitlab_url = "https://gitlab.com"
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

# Optional: Default theme (see Theming guide for options)
# theme = "catppuccin-mocha"
```

### Self-Hosted & Self-Signed Certificates

If your organization uses a self-hosted GitLab instance with custom or self-signed SSL certificates, pass the `-k` / `--insecure` flag or set it in your config:

```bash
lazygitlab --url https://gitlab.internal.corp --token $TOKEN --insecure
```

---

## 🚀 Launching

### Automatic Git Remote Detection

When run inside any local git repository whose remote origin points to GitLab:

```bash
cd ~/Projects/my-awesome-project
lazygitlab
```

`lazygitlab` automatically extracts the repository URL from your `.git/config`, resolves the project on GitLab, and opens it immediately!

### Launching Directly into a Specific Project

You can also target any project by its GitLab ID or path:

```bash
# By project path
lazygitlab -p my-group/backend-service

# By numeric project ID
lazygitlab -p 81221057
```

### Personal Dashboard Mode

When launched outside of a git repository, `lazygitlab` opens your **Personal Dashboard**, displaying your cross-project To-Dos, assigned Merge Requests, active Issues, and CI/CD pipelines.

---

## 🎨 Next Steps

* **[Personal Dashboard](user-guide/dashboard.md)**: Master your unified To-Do inbox and review queue.
* **[Code Review & Diffs](user-guide/code-review.md)**: Perform fast, keyboard-driven MR reviews with split/unified diffs and comments.
* **[Keybindings & Cheatsheet](reference/keybindings.md)**: Explore the interactive command palette and printable 1-page PDF cheatsheet.
* **[Theming & Color Schemes](reference/theming.md)**: Try our interactive theme visualizer and customize your colors.
