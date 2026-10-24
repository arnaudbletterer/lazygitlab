# Theming

`lazygitlab` supports fully customizable color schemes. You can choose from built-in classical presets or create a custom palette in `theme.toml` in your config directory (`~/.config/lazygitlab/`). It is not read from the working directory.

---

## Built-in Presets Gallery

Select any preset tab below to preview how it looks directly in `lazygitlab`:

=== "Catppuccin Mocha"

    ![Catppuccin Mocha Theme](../assets/screenshots/theme_catppuccin_mocha.svg)

    *Soothing pastel dark palette (Default)*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "catppuccin-mocha"
    ```

=== "Tokyo Night"

    ![Tokyo Night Theme](../assets/screenshots/theme_tokyo_night.svg)

    *Clean dark blue & cyan aesthetic*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "tokyo-night"
    ```

=== "Nord"

    ![Nord Theme](../assets/screenshots/theme_nord.svg)

    *Arctic, north-bluish calm slate palette*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "nord"
    ```

=== "Dracula"

    ![Dracula Theme](../assets/screenshots/theme_dracula.svg)

    *High-contrast vibrant purple & pink dark theme*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "dracula"
    ```

=== "Gruvbox Dark"

    ![Gruvbox Dark Theme](../assets/screenshots/theme_gruvbox_dark.svg)

    *Retro groove warm dark palette*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "gruvbox-dark"
    ```

=== "GitHub Dark"

    ![GitHub Dark Theme](../assets/screenshots/theme_github_dark.svg)

    *Official GitHub Dark UI palette*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "github-dark"
    ```

=== "Catppuccin Latte"

    ![Catppuccin Latte Theme](../assets/screenshots/theme_catppuccin_latte.svg)

    *Soft, warm pastel light theme*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "catppuccin-latte"
    ```

=== "Gruvbox Light"

    ![Gruvbox Light Theme](../assets/screenshots/theme_gruvbox_light.svg)

    *Warm parchment retro groove light theme*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "gruvbox-light"
    ```

=== "GitHub Light"

    ![GitHub Light Theme](../assets/screenshots/theme_github_light.svg)

    *Official crisp GitHub Light UI palette*

    ```toml
    # ~/.config/lazygitlab/theme.toml
    theme = "github-light"
    ```

---

## Custom Themes

You can easily define your own theme by creating `~/.config/lazygitlab/theme.toml`.

`lazygitlab` uses an **intelligent base-color system**: you only need to define **8 core colors**, and all UI elements (borders, status pills, diff lines, badges, and headers) automatically derive from them with optimal contrast.

### Minimal Example

```toml
# ~/.config/lazygitlab/theme.toml

# 1. Base Typography & Structure
title_fg     = "#c0caf5"   # Primary text and titles
dim          = "#565f89"   # Secondary / muted text & inactive borders
border       = "#73daca"   # Active panel borders and focus indicators

# 2. Surfaces & Accents
accent       = "#7aa2f7"   # Primary accent (active tabs, open MRs, running jobs)
highlight_bg = "#292e42"   # Selection surface & highlighted rows
code_bg      = "#16161e"   # Panel & code block background

# 3. Statuses
success      = "#9ece6a"   # Green: passing CI/CD, merged items, additions
failed       = "#f7768e"   # Red: failed CI/CD, closed items, deletions
```

---

## Core Color Reference

| Color Key | Role | What It Colors |
| :--- | :--- | :--- |
| `title_fg` | **Primary Text** | Item titles, descriptions, table headers |
| `dim` | **Muted Text** | Author usernames, line numbers, inactive borders |
| `border` | **Focus Border** | Active panel borders, popup frames |
| `accent` | **Primary Accent** | Active tabs, open MRs, running pipeline spinners, keybinding shortcuts |
| `highlight_bg` | **Selection Row** | Selected cursor rows, badge backgrounds |
| `code_bg` | **Base Background** | Popup dialog background, diff view background |
| `success` | **Positive Status** | Passing pipelines (`PASSED`), approvals, diff additions |
| `failed` | **Negative Status** | Failed pipelines (`FAILED`), closed MRs, diff deletions |

> [!TIP]
> **Extending a Preset**: You can also inherit a preset and override only specific colors:
> ```toml
> theme = "tokyo-night"
> accent = "#ff007f"        # Override just the accent to hot pink
> ```

---

## Supported Color Formats

Colors can be specified in three flexible formats:

* **Hex strings**: `"#7aa2f7"`, `"#3d5"`
* **Standard named colors**: `"Blue"`, `"Green"`, `"Cyan"`, `"Red"`, `"Yellow"`, `"White"`, `"Black"`
* **RGB integer triplets**: `[122, 162, 247]`
