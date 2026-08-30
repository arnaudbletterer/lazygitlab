# Configuration

lazygitlab reads configuration from `~/.config/lazygitlab/config.toml`.

## Authentication

Set your GitLab URL and Personal Access Token:

```toml
gitlab_url = "https://gitlab.com"
token = "glpat-xxxxxxxxxxxxxxxxxxxx"
```

## Auto-detection

lazygitlab automatically detects the GitLab project based on the current directory's git remotes.

## Theming

You can customize the theme by editing `~/.config/lazygitlab/theme.toml`. See the [Theming](../reference/theming.md) guide for details.
