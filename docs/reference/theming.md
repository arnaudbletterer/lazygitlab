# Theming

`lazygitlab` supports fully customizable color schemes. You can choose from built-in classical presets or create a custom palette in `theme.toml`.

---

## 🎨 Built-in Presets

To activate a built-in theme, specify its name in `~/.config/lazygitlab/theme.toml` (or `~/.config/lazygitlab/config.toml`):

```toml
theme = "tokyo-night"
```

=== "Dark Themes"

    | Preset | Description | Key Characteristic |
    | :--- | :--- | :--- |
    | `catppuccin-mocha` | Soothing pastel dark palette *(Default)* | Soft mauve & blue accents |
    | `tokyo-night` | Tokyo Night aesthetic | Clean dark blue & cyan |
    | `nord` | Arctic, north-bluish palette | Frost teal & calm slate |
    | `dracula` | High-contrast dark theme | Vibrant purple & pink |
    | `gruvbox-dark` | Retro groove dark palette | Warm amber, forest green & orange |
    | `github-dark` | Official GitHub Dark UI palette | High-contrast gray & blue |

=== "Light Themes"

    | Preset | Description | Key Characteristic |
    | :--- | :--- | :--- |
    | `catppuccin-latte` | Soft warm pastel light theme | High-contrast purple on cream |
    | `gruvbox-light` | Retro groove light palette | Warm parchment with dark text |
    | `github-light` | Official GitHub Light UI palette | Crisp white & cobalt blue |

---

## 🛠️ Custom Themes

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

## 📋 Core Color Reference

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

## 🎨 Supported Color Formats

Colors can be specified in three flexible formats:

* **Hex strings**: `"#7aa2f7"`, `"#3d5"`
* **Standard named colors**: `"Blue"`, `"Green"`, `"Cyan"`, `"Red"`, `"Yellow"`, `"White"`, `"Black"`
* **RGB integer triplets**: `[122, 162, 247]`
