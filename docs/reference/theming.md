# Theming & Color Schemes

`lazygitlab` comes with a rich set of built-in classical TUI themes and supports complete, granular color customization via `theme.toml`.

---

## 🎨 Interactive Theme Visualizer

Select a theme preset below to preview how it looks live in `lazygitlab`, inspect its 5-color core palette swatch bar, and observe UI components including side lists, status badges, diff view syntax highlighting, and keybinding hints.

<div class="theme-visualizer-toolbar">
  <div class="theme-select-wrapper">
    <label for="theme-select" class="theme-select-label">
      <span>🎨</span> Theme Preset:
    </label>
    <select id="theme-select" class="theme-select" onchange="applyTheme(this.value)">
      <optgroup label="Dark Themes">
        <option value="catppuccin-mocha" selected>Catppuccin Mocha (Default)</option>
        <option value="tokyo-night">Tokyo Night</option>
        <option value="nord">Nord</option>
        <option value="dracula">Dracula</option>
        <option value="gruvbox-dark">Gruvbox Dark</option>
        <option value="github-dark">GitHub Dark</option>
      </optgroup>
      <optgroup label="Light Themes">
        <option value="catppuccin-latte">Catppuccin Latte</option>
        <option value="gruvbox-light">Gruvbox Light</option>
        <option value="github-light">GitHub Light</option>
      </optgroup>
    </select>
  </div>
</div>

<!-- 5-Color Core Palette Swatch Bar -->
<div class="palette-swatches-bar" id="palette-bar">
  <div class="palette-swatch" title="Background: Main canvas and base container fill">
    <div class="swatch-color-circle" id="swatch-bg"></div>
    <div class="swatch-details">
      <span class="swatch-name">Background</span>
      <span class="swatch-hex" id="hex-bg">#1e1e2e</span>
    </div>
  </div>
  <div class="palette-swatch" title="Foreground: Primary typography, titles, and text">
    <div class="swatch-color-circle" id="swatch-fg"></div>
    <div class="swatch-details">
      <span class="swatch-name">Foreground</span>
      <span class="swatch-hex" id="hex-fg">#cdd6f4</span>
    </div>
  </div>
  <div class="palette-swatch" title="Accent: Focal UI points, active tab text, and primary badges">
    <div class="swatch-color-circle" id="swatch-accent"></div>
    <div class="swatch-details">
      <span class="swatch-name">Accent</span>
      <span class="swatch-hex" id="hex-accent">#89b4fa</span>
    </div>
  </div>
  <div class="palette-swatch" title="Border: Window panels, block borders, and separators">
    <div class="swatch-color-circle" id="swatch-border"></div>
    <div class="swatch-details">
      <span class="swatch-name">Border</span>
      <span class="swatch-hex" id="hex-border">#a6e3a1</span>
    </div>
  </div>
  <div class="palette-swatch" title="Highlight: Selection rows, active list focus, and cursor backgrounds">
    <div class="swatch-color-circle" id="swatch-highlight"></div>
    <div class="swatch-details">
      <span class="swatch-name">Highlight</span>
      <span class="swatch-hex" id="hex-highlight">#89b4fa</span>
    </div>
  </div>
</div>

<!-- Mock TUI Window -->
<div id="mock-tui" style="border-radius: 12px; padding: 16px; font-family: 'JetBrains Mono', 'SF Mono', monospace; font-size: 12.5px; line-height: 1.4; transition: all 0.3s ease; box-shadow: 0 16px 36px rgba(0,0,0,0.35); border: 1px solid var(--tui-border); margin-bottom: 2rem;">
  <!-- Window Header -->
  <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--tui-border);">
    <div style="display: flex; gap: 6px;">
      <span style="width: 11px; height: 11px; border-radius: 50%; background: #ff5f57; display: inline-block;"></span>
      <span style="width: 11px; height: 11px; border-radius: 50%; background: #febc2e; display: inline-block;"></span>
      <span style="width: 11px; height: 11px; border-radius: 50%; background: #28c840; display: inline-block;"></span>
    </div>
    <span style="color: var(--tui-header); font-weight: bold; margin-left: 10px;">🦊 lazygitlab v0.16.0</span>
    <span style="color: var(--tui-dim); margin-left: auto;">abletterer/lazygitlab (main)</span>
  </div>

  <!-- Mock Split View -->
  <div style="display: grid; grid-template-columns: 230px 1fr; gap: 14px;">
    <!-- Side Panel -->
    <div style="border: 1px solid var(--tui-border); border-radius: 6px; padding: 8px;">
      <div style="color: var(--tui-header1); font-weight: bold; margin-bottom: 6px;">📋 MERGE REQUESTS</div>
      <div style="background: var(--tui-highlight); color: #ffffff; padding: 2px 6px; border-radius: 4px; font-weight: 500;">!124 feat(diff): split review</div>
      <div style="color: var(--tui-dim); padding: 2px 6px;">!123 fix(pipeline): cancel</div>
      <div style="color: var(--tui-dim); padding: 2px 6px;">!122 perf(tree): fast load</div>
      <div style="margin-top: 10px; color: var(--tui-header2); font-weight: bold;">⚡ STATUS</div>
      <div style="color: var(--tui-success);">✔ 14 Passed</div>
      <div style="color: var(--tui-running);">⟳ 1 Running</div>
      <div style="color: var(--tui-pending);">⏸ 2 Pending</div>
    </div>

    <!-- Detail Panel -->
    <div style="border: 1px solid var(--tui-border); border-radius: 6px; padding: 10px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
        <span style="color: var(--tui-mr); font-weight: bold;">!124 feat(diff): add syntax-highlighted diff review</span>
        <span style="background: var(--tui-success); color: #000000; font-size: 11px; font-weight: bold; padding: 1px 6px; border-radius: 3px;">✔ APPROVED</span>
      </div>
      <div style="color: var(--tui-dim); margin-bottom: 8px;">
        Author: <span style="color: var(--tui-author);">@abletterer</span> • Branch: <span style="color: var(--tui-project);">master</span>
      </div>
      <!-- Code Snippet -->
      <div style="background: var(--tui-code-bg); padding: 8px; border-radius: 4px; font-size: 12px;">
        <div><span style="color: var(--tui-dim);">42</span> <span style="color: var(--tui-keyword);">pub async fn</span> <span style="color: var(--tui-code);">post_comment</span>(&amp;<span style="color: var(--tui-keyword);">mut self</span>) -&gt; Result&lt;()&gt; {</div>
        <div style="background: var(--tui-range-hl);"><span style="color: var(--tui-success);">+43</span>     <span style="color: var(--tui-keyword);">let</span> discussion = CreateMRDiscussion { project, body };</div>
        <div><span style="color: var(--tui-dim);">44</span>     discussion.query_async(&amp;client).<span style="color: var(--tui-keyword);">await</span>?;</div>
        <div><span style="color: var(--tui-dim);">45</span> }</div>
      </div>
    </div>
  </div>

  <!-- Footer Keybindings -->
  <div style="margin-top: 12px; padding-top: 8px; border-top: 1px solid var(--tui-border); display: flex; flex-wrap: wrap; gap: 12px; font-size: 11.5px;">
    <span><strong style="color: var(--tui-key);">Enter</strong> <span style="color: var(--tui-desc);">Diff</span></span>
    <span><strong style="color: var(--tui-key);">a</strong> <span style="color: var(--tui-desc);">Approve</span></span>
    <span><strong style="color: var(--tui-key);">M</strong> <span style="color: var(--tui-desc);">Merge</span></span>
    <span><strong style="color: var(--tui-key);">r</strong> <span style="color: var(--tui-desc);">Refresh</span></span>
    <span><strong style="color: var(--tui-key);">?</strong> <span style="color: var(--tui-desc);">Help</span></span>
  </div>
</div>

<script>
const themes = {
  'catppuccin-mocha': {
    bg: '#1e1e2e', fg: '#cdd6f4', accent: '#89b4fa', border: '#a6e3a1', highlight: '#89b4fa',
    header: '#89b4fa', header1: '#89dceb', header2: '#cba6f7', dim: '#6c7086',
    author: '#f5c2e7', project: '#89dceb', success: '#a6e3a1', failed: '#f38ba8',
    running: '#89b4fa', pending: '#f9e2af', mr: '#cba6f7', issue: '#a6e3a1',
    key: '#89dceb', desc: '#cdd6f4', keyword: '#f38ba8', code: '#a6e3a1',
    codeBg: '#181825', rangeHl: '#313244'
  },
  'tokyo-night': {
    bg: '#1a1b26', fg: '#c0caf5', accent: '#7aa2f7', border: '#73daca', highlight: '#3d59a1',
    header: '#7aa2f7', header1: '#7dcfff', header2: '#bb9af7', dim: '#565f89',
    author: '#bb9af7', project: '#7dcfff', success: '#9ece6a', failed: '#f7768e',
    running: '#7aa2f7', pending: '#e0af68', mr: '#bb9af7', issue: '#9ece6a',
    key: '#7dcfff', desc: '#c0caf5', keyword: '#f7768e', code: '#9ece6a',
    codeBg: '#16161e', rangeHl: '#292e42'
  },
  'nord': {
    bg: '#2e3440', fg: '#e5e9f0', accent: '#81a1c1', border: '#8fbcbb', highlight: '#5e81ac',
    header: '#88c0d0', header1: '#8fbcbb', header2: '#b48ead', dim: '#616e88',
    author: '#b48ead', project: '#88c0d0', success: '#a3be8c', failed: '#bf616a',
    running: '#81a1c1', pending: '#ebcb8b', mr: '#b48ead', issue: '#a3be8c',
    key: '#88c0d0', desc: '#e5e9f0', keyword: '#bf616a', code: '#a3be8c',
    codeBg: '#242933', rangeHl: '#434c5e'
  },
  'dracula': {
    bg: '#282a36', fg: '#f8f8f2', accent: '#8be9fd', border: '#bd93f9', highlight: '#6272a4',
    header: '#bd93f9', header1: '#8be9fd', header2: '#ff79c6', dim: '#6272a4',
    author: '#ff79c6', project: '#8be9fd', success: '#50fa7b', failed: '#ff5555',
    running: '#8be9fd', pending: '#f1fa8c', mr: '#ff79c6', issue: '#50fa7b',
    key: '#8be9fd', desc: '#f8f8f2', keyword: '#ff5555', code: '#50fa7b',
    codeBg: '#1e1f29', rangeHl: '#44475a'
  },
  'gruvbox-dark': {
    bg: '#282828', fg: '#ebdbb2', accent: '#83a598', border: '#b8bb26', highlight: '#458588',
    header: '#fe8019', header1: '#8ec07c', header2: '#d3869b', dim: '#928374',
    author: '#d3869b', project: '#8ec07c', success: '#b8bb26', failed: '#fb4934',
    running: '#83a598', pending: '#fabd2f', mr: '#d3869b', issue: '#b8bb26',
    key: '#fe8019', desc: '#ebdbb2', keyword: '#fb4934', code: '#b8bb26',
    codeBg: '#1d2021', rangeHl: '#504945'
  },
  'github-dark': {
    bg: '#0d1117', fg: '#f0f6fc', accent: '#58a6ff', border: '#2ea043', highlight: '#1f6feb',
    header: '#58a6ff', header1: '#39c5bb', header2: '#a371f7', dim: '#6e7681',
    author: '#a371f7', project: '#58a6ff', success: '#3fb950', failed: '#f85149',
    running: '#58a6ff', pending: '#d29922', mr: '#a371f7', issue: '#3fb950',
    key: '#58a6ff', desc: '#f0f6fc', keyword: '#f85149', code: '#3fb950',
    codeBg: '#161b22', rangeHl: '#30363d'
  },
  'catppuccin-latte': {
    bg: '#eff1f5', fg: '#4c4f69', accent: '#1e66f5', border: '#40a02b', highlight: '#1e66f5',
    header: '#1e66f5', header1: '#209fb5', header2: '#8839ef', dim: '#9ca0b0',
    author: '#8839ef', project: '#209fb5', success: '#40a02b', failed: '#d20f39',
    running: '#1e66f5', pending: '#df8e1d', mr: '#8839ef', issue: '#40a02b',
    key: '#1e66f5', desc: '#4c4f69', keyword: '#d20f39', code: '#40a02b',
    codeBg: '#e6e9ef', rangeHl: '#ccd0da'
  },
  'github-light': {
    bg: '#ffffff', fg: '#1f2328', accent: '#0969da', border: '#1a7f37', highlight: '#0969da',
    header: '#0969da', header1: '#0550ae', header2: '#8250df', dim: '#6e7781',
    author: '#8250df', project: '#0969da', success: '#1a7f37', failed: '#cf222e',
    running: '#0969da', pending: '#9a6700', mr: '#8250df', issue: '#1a7f37',
    key: '#0969da', desc: '#1f2328', keyword: '#cf222e', code: '#1a7f37',
    codeBg: '#f6f8fa', rangeHl: '#ddf4ff'
  }
};

function applyTheme(name) {
  const t = themes[name];
  if (!t) return;

  // Update Swatch Bar
  document.getElementById('swatch-bg').style.backgroundColor = t.bg;
  document.getElementById('hex-bg').textContent = t.bg;

  document.getElementById('swatch-fg').style.backgroundColor = t.fg;
  document.getElementById('hex-fg').textContent = t.fg;

  document.getElementById('swatch-accent').style.backgroundColor = t.accent;
  document.getElementById('hex-accent').textContent = t.accent;

  document.getElementById('swatch-border').style.backgroundColor = t.border;
  document.getElementById('hex-border').textContent = t.border;

  document.getElementById('swatch-highlight').style.backgroundColor = t.highlight;
  document.getElementById('hex-highlight').textContent = t.highlight;

  // Update Mock TUI Properties
  const box = document.getElementById('mock-tui');
  box.style.background = t.bg;
  box.style.setProperty('--tui-border', t.border);
  box.style.setProperty('--tui-header', t.header);
  box.style.setProperty('--tui-header1', t.header1);
  box.style.setProperty('--tui-header2', t.header2);
  box.style.setProperty('--tui-dim', t.dim);
  box.style.setProperty('--tui-highlight', t.highlight);
  box.style.setProperty('--tui-mr', t.mr);
  box.style.setProperty('--tui-author', t.author);
  box.style.setProperty('--tui-project', t.project);
  box.style.setProperty('--tui-success', t.success);
  box.style.setProperty('--tui-running', t.running);
  box.style.setProperty('--tui-pending', t.pending);
  box.style.setProperty('--tui-code-bg', t.codeBg);
  box.style.setProperty('--tui-keyword', t.keyword);
  box.style.setProperty('--tui-code', t.code);
  box.style.setProperty('--tui-range-hl', t.rangeHl);
  box.style.setProperty('--tui-key', t.key);
  box.style.setProperty('--tui-desc', t.desc);

  // Sync Dropdown Value
  const select = document.getElementById('theme-select');
  if (select && select.value !== name) {
    select.value = name;
  }
}

// Initialize on page load
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => applyTheme('catppuccin-mocha'));
} else {
  applyTheme('catppuccin-mocha');
}
</script>

---

## ⚙️ Selecting a Preset Theme

To activate any built-in theme, create a `theme.toml` file in your working directory (or next to the `lazygitlab` binary) and set the `theme` property:

```toml
theme = "tokyo-night"
```

Preset names are case-insensitive and ignore hyphens, underscores, and spaces.

| Preset Key | Mode | Description | Aliases |
| :--- | :--- | :--- | :--- |
| `"catppuccin-mocha"` | **Dark** | Soothing pastel theme (Default) | `"mocha"`, `"default"` |
| `"tokyo-night"` | **Dark** | Modern dark theme inspired by Tokyo neon lights | `"tokyo"` |
| `"nord"` | **Dark** | Arctic, north-bluish clean color palette | `"nord"` |
| `"dracula"` | **Dark** | High-contrast dark theme with vibrant accents | `"dracula"` |
| `"gruvbox-dark"` | **Dark** | Retro groove warm dark palette | `"gruvbox"` |
| `"github-dark"` | **Dark** | GitHub Dark default syntax & interface colors | `"githubdark"` |
| `"catppuccin-latte"` | **Light** | Crisp light pastel theme | `"latte"` |
| `"github-light"` | **Light** | GitHub Light clean interface styling | `"githublight"` |

---

## 📝 Custom Theme Configuration (`theme.toml`)

`lazygitlab` supports comprehensive color customizations. You can override any or all color keys by specifying them in `theme.toml`.

### Supported Color Formats

Colors in `theme.toml` can be declared in three convenient formats:

1. **Hex Strings**: `"#RRGGBB"` (e.g. `"#89b4fa"`, `"#1e1e2e"`)
2. **Standard Color Names**: `"Green"`, `"Red"`, `"Blue"`, `"Black"`, `"White"`, `"Cyan"`, `"Magenta"`, `"Yellow"`, `"DarkGray"`, `"LightRed"`, `"LightGreen"`, `"LightYellow"`, `"LightBlue"`, `"LightMagenta"`, `"LightCyan"`, `"Gray"`
3. **RGB Integer Arrays**: `[r, g, b]` (e.g. `[100, 100, 200]`)

### Complete `theme.toml` Example

Below is a fully populated `theme.toml` configuration covering all available UI keys:

```toml
# ==============================================================================
# lazygitlab Theme Configuration
# Place this file in your working directory or next to the lazygitlab executable.
# ==============================================================================

[theme]
# --- Window Panels & Borders ---
border = "#a6e3a1"            # Default window/panel border color
active_border = "#a6e3a1"     # Focused panel border
inactive_border = "#6c7086"   # Unfocused/inactive panel border

# --- Highlights & Accents ---
highlight_bg = "#89b4fa"      # Selected list row / active tab background
accent = "#89b4fa"            # Primary accent for focus headers and active tabs
title_fg = "#a6e3a1"          # Panel title text and breadcrumbs

# --- Application Headers ---
header = "#89b4fa"            # Top banner title and primary header
header1 = "#89dceb"           # Section subheaders & category titles
header2 = "#cba6f7"           # Auxiliary subheaders & secondary category titles

# --- Entity Identification ---
author = "#f5c2e7"            # Author username (@name)
dim = "#6c7086"               # Dimmed metadata, line numbers, and separators (❯)
project = "#89dceb"           # Project names, namespaces, and branch references
mr = "#cba6f7"                # Merge request references (!123) and titles
issue = "#a6e3a1"             # Issue references (#456) and titles

# --- Keybinding Badges & Hints ---
shortcut_key = "#89dceb"      # Shortcut key text (e.g. [Enter], [a], [M])
shortcut_desc = "#cdd6f4"     # Shortcut description label
badge_bg = "#89b4fa"          # Keybinding shortcut badge background
badge_fg = "#a6e3a1"          # Keybinding shortcut badge text

# --- Chips, Pills & Labels ---
chip_bg = "#a6e3a1"           # Milestone chip and status pill background
chip_fg = "#181825"           # Milestone chip and status pill text
label_bg = "#6c7086"          # Default issue / MR label background
label_fg = "#181825"          # Default issue / MR label text

# --- Diff Viewer & Syntax Highlighting ---
line_number_fg = "#6c7086"    # Gutter line numbers in diffs and file viewers
hunk_header_bg = "#89b4fa"    # Diff hunk header background (@@ -1,4 +1,5 @@)
hunk_header_fg = "#a6e3a1"    # Diff hunk header text
diff_add_bg = "#a6e3a1"       # Background fill for added diff lines
diff_add_fg = "#181825"       # Text for added diff lines
diff_del_bg = "#f38ba8"       # Background fill for deleted diff lines
diff_del_fg = "#181825"       # Text for deleted diff lines
keyword = "#f38ba8"           # Code syntax keyword token (fn, let, pub, etc.)
code = "#a6e3a1"              # Code body text and identifiers
code_bg = "#181825"           # Code viewer and textarea background fill
range_highlight = "#313244"   # Background for selected/active diff line span

# --- CI/CD & Pipeline Statuses ---
success = "#a6e3a1"           # Success status text and approved badges
failed = "#f38ba8"            # Failure / error status text
running = "#89b4fa"           # In-progress / running job indicator
pending = "#f9e2af"           # Pending / queued job indicator
status_success = "#a6e3a1"    # Dedicated status badge for passed jobs (✔)
status_failed = "#f38ba8"     # Dedicated status badge for failed jobs (✖)
status_running = "#89b4fa"    # Dedicated status badge for running jobs (⟳)
status_pending = "#f9e2af"    # Dedicated status badge for queued jobs (⏸)
status_draft = "#6c7086"      # Status badge for draft MRs (⭘)
status_merged = "#cba6f7"     # Status badge for merged MRs (✦)

# --- Merge Request / Issue States ---
open_mr = "#89b4fa"           # Color for open MRs and open issues
merged_mr = "#cba6f7"         # Color for merged MRs
closed_mr = "#f38ba8"         # Color for closed MRs and closed issues
```

---

## 📊 Complete Color Keys Reference

| Key Name | Category | Description |
| :--- | :--- | :--- |
| `border` | **Borders** | Default border color for windows, popup dialogs, and panels. |
| `active_border` | **Borders** | Border color of the currently focused UI pane. |
| `inactive_border` | **Borders** | Border color of unfocused/inactive panes. |
| `highlight_bg` | **Accents** | Background highlight for selected list items, rows, and active menu tabs. |
| `accent` | **Accents** | Primary focus accent color for active section headers and indicators. |
| `title_fg` | **Accents** | Foreground text color for pane title banners and navigation breadcrumbs. |
| `header` | **Headers** | Primary application top header bar and branding text. |
| `header1` | **Headers** | Section headers, category titles, and primary panel headings. |
| `header2` | **Headers** | Secondary category headers and auxiliary section titles. |
| `author` | **Entities** | Usernames and author mentions (`@username`). |
| `dim` | **Entities** | Subdued text, line number gutters, timestamps, and separator glyphs (`❯`). |
| `project` | **Entities** | Project namespace paths and Git branch identifiers. |
| `mr` | **Entities** | Merge Request reference numbers (`!124`) and MR titles. |
| `issue` | **Entities** | Issue reference numbers (`#42`) and issue titles. |
| `shortcut_key` | **Shortcuts** | Keybinding trigger keys in footer bars and popups (e.g. `[Enter]`, `[r]`). |
| `shortcut_desc` | **Shortcuts** | Descriptive label text accompanying shortcut keys. |
| `badge_bg` | **Shortcuts** | Background fill for shortcut key badges. |
| `badge_fg` | **Shortcuts** | Text color inside shortcut key badges. |
| `chip_bg` | **Pills** | Milestone chip and auxiliary pill background. |
| `chip_fg` | **Pills** | Milestone chip and auxiliary pill text. |
| `label_bg` | **Pills** | Default background for issue and MR labels when hex is absent. |
| `label_fg` | **Pills** | Default text color for issue and MR labels. |
| `line_number_fg` | **Diffs** | Line number gutter color in diff review and file viewing modes. |
| `hunk_header_bg` | **Diffs** | Background banner fill for unified diff hunk headers (`@@ ... @@`). |
| `hunk_header_fg` | **Diffs** | Foreground text for diff hunk headers. |
| `diff_add_bg` | **Diffs** | Background fill color for added lines (`+`) in diffs. |
| `diff_add_fg` | **Diffs** | Foreground text color for added lines in diffs. |
| `diff_del_bg` | **Diffs** | Background fill color for deleted lines (`-`) in diffs. |
| `diff_del_fg` | **Diffs** | Foreground text color for deleted lines in diffs. |
| `keyword` | **Diffs** | Code syntax highlighting color for keywords (`pub`, `fn`, `async`, etc.). |
| `code` | **Diffs** | Code body text, functions, and identifier tokens. |
| `code_bg` | **Diffs** | Background fill for code snippets, markdown blocks, and text areas. |
| `range_highlight`| **Diffs** | Highlight background for selected line spans and diff selections. |
| `success` | **Status** | General success indicator, passed CI pipeline badges, and approvals. |
| `failed` | **Status** | General error indicator, failed pipelines, and rejection banners. |
| `running` | **Status** | Running CI/CD pipeline jobs and spinner activity. |
| `pending` | **Status** | Queued, pending, or scheduled pipeline stages. |
| `status_success` | **Status** | Dedicated badge color for passed jobs and resolved discussions (`✔`). |
| `status_failed` | **Status** | Dedicated badge color for failed jobs and unresolved blockers (`✖`). |
| `status_running` | **Status** | Dedicated badge color for active running jobs (`⟳`). |
| `status_pending` | **Status** | Dedicated badge color for pending/queued jobs (`⏸`). |
| `status_draft` | **Status** | Dedicated badge color for draft / WIP merge requests (`⭘`). |
| `status_merged` | **Status** | Dedicated badge color for merged merge requests (`✦`). |
| `open_mr` | **Lifecycle** | State color for open merge requests and open issues. |
| `merged_mr` | **Lifecycle** | State color for merged merge requests. |
| `closed_mr` | **Lifecycle** | State color for closed merge requests and closed issues. |

---

## 🛠️ Complete Preset Configurations

Copy and customize the exact definitions of any built-in preset below into your `theme.toml`:

=== "Catppuccin Mocha"
    ```toml
    [theme]
    border = "#a6e3a1"
    active_border = "#a6e3a1"
    inactive_border = "#6c7086"
    highlight_bg = "#89b4fa"
    accent = "#89b4fa"
    title_fg = "#a6e3a1"
    header = "#89b4fa"
    header1 = "#89dceb"
    header2 = "#cba6f7"
    author = "#f5c2e7"
    dim = "#6c7086"
    project = "#89dceb"
    mr = "#cba6f7"
    issue = "#a6e3a1"
    shortcut_key = "#89dceb"
    shortcut_desc = "#cdd6f4"
    badge_bg = "#89b4fa"
    badge_fg = "#a6e3a1"
    chip_bg = "#a6e3a1"
    chip_fg = "#181825"
    label_bg = "#6c7086"
    label_fg = "#181825"
    line_number_fg = "#6c7086"
    hunk_header_bg = "#89b4fa"
    hunk_header_fg = "#a6e3a1"
    diff_add_bg = "#a6e3a1"
    diff_add_fg = "#181825"
    diff_del_bg = "#f38ba8"
    diff_del_fg = "#181825"
    success = "#a6e3a1"
    failed = "#f38ba8"
    running = "#89b4fa"
    pending = "#f9e2af"
    status_success = "#a6e3a1"
    status_failed = "#f38ba8"
    status_running = "#89b4fa"
    status_pending = "#f9e2af"
    status_draft = "#6c7086"
    status_merged = "#cba6f7"
    open_mr = "#89b4fa"
    merged_mr = "#cba6f7"
    closed_mr = "#f38ba8"
    keyword = "#f38ba8"
    code = "#a6e3a1"
    code_bg = "#181825"
    range_highlight = "#313244"
    ```

=== "Tokyo Night"
    ```toml
    [theme]
    border = "#73daca"
    active_border = "#73daca"
    inactive_border = "#565f89"
    highlight_bg = "#3d59a1"
    accent = "#7aa2f7"
    title_fg = "#9ece6a"
    header = "#7aa2f7"
    header1 = "#7dcfff"
    header2 = "#bb9af7"
    author = "#bb9af7"
    dim = "#565f89"
    project = "#7dcfff"
    mr = "#bb9af7"
    issue = "#9ece6a"
    shortcut_key = "#7dcfff"
    shortcut_desc = "#c0caf5"
    badge_bg = "#3d59a1"
    badge_fg = "#9ece6a"
    chip_bg = "#73daca"
    chip_fg = "#1a1b26"
    label_bg = "#565f89"
    label_fg = "#1a1b26"
    line_number_fg = "#565f89"
    hunk_header_bg = "#3d59a1"
    hunk_header_fg = "#9ece6a"
    diff_add_bg = "#9ece6a"
    diff_add_fg = "#1a1b26"
    diff_del_bg = "#f7768e"
    diff_del_fg = "#1a1b26"
    success = "#9ece6a"
    failed = "#f7768e"
    running = "#7aa2f7"
    pending = "#e0af68"
    status_success = "#9ece6a"
    status_failed = "#f7768e"
    status_running = "#7aa2f7"
    status_pending = "#e0af68"
    status_draft = "#565f89"
    status_merged = "#bb9af7"
    open_mr = "#7aa2f7"
    merged_mr = "#bb9af7"
    closed_mr = "#f7768e"
    keyword = "#f7768e"
    code = "#9ece6a"
    code_bg = "#1a1b26"
    range_highlight = "#292e42"
    ```

=== "Nord"
    ```toml
    [theme]
    border = "#8fbcbb"
    active_border = "#8fbcbb"
    inactive_border = "#616e88"
    highlight_bg = "#5e81ac"
    accent = "#81a1c1"
    title_fg = "#a3be8c"
    header = "#88c0d0"
    header1 = "#8fbcbb"
    header2 = "#b48ead"
    author = "#b48ead"
    dim = "#616e88"
    project = "#88c0d0"
    mr = "#b48ead"
    issue = "#a3be8c"
    shortcut_key = "#88c0d0"
    shortcut_desc = "#e5e9f0"
    badge_bg = "#5e81ac"
    badge_fg = "#a3be8c"
    chip_bg = "#8fbcbb"
    chip_fg = "#2e3440"
    label_bg = "#616e88"
    label_fg = "#2e3440"
    line_number_fg = "#616e88"
    hunk_header_bg = "#5e81ac"
    hunk_header_fg = "#a3be8c"
    diff_add_bg = "#a3be8c"
    diff_add_fg = "#2e3440"
    diff_del_bg = "#bf616a"
    diff_del_fg = "#2e3440"
    success = "#a3be8c"
    failed = "#bf616a"
    running = "#81a1c1"
    pending = "#ebcb8b"
    status_success = "#a3be8c"
    status_failed = "#bf616a"
    status_running = "#81a1c1"
    status_pending = "#ebcb8b"
    status_draft = "#616e88"
    status_merged = "#b48ead"
    open_mr = "#81a1c1"
    merged_mr = "#b48ead"
    closed_mr = "#bf616a"
    keyword = "#bf616a"
    code = "#a3be8c"
    code_bg = "#2e3440"
    range_highlight = "#434c5e"
    ```

=== "Dracula"
    ```toml
    [theme]
    border = "#bd93f9"
    active_border = "#bd93f9"
    inactive_border = "#6272a4"
    highlight_bg = "#6272a4"
    accent = "#8be9fd"
    title_fg = "#50fa7b"
    header = "#bd93f9"
    header1 = "#8be9fd"
    header2 = "#ff79c6"
    author = "#ff79c6"
    dim = "#6272a4"
    project = "#8be9fd"
    mr = "#ff79c6"
    issue = "#50fa7b"
    shortcut_key = "#8be9fd"
    shortcut_desc = "#f8f8f2"
    badge_bg = "#6272a4"
    badge_fg = "#50fa7b"
    chip_bg = "#bd93f9"
    chip_fg = "#282a36"
    label_bg = "#6272a4"
    label_fg = "#282a36"
    line_number_fg = "#6272a4"
    hunk_header_bg = "#6272a4"
    hunk_header_fg = "#50fa7b"
    diff_add_bg = "#50fa7b"
    diff_add_fg = "#282a36"
    diff_del_bg = "#ff5555"
    diff_del_fg = "#282a36"
    success = "#50fa7b"
    failed = "#ff5555"
    running = "#8be9fd"
    pending = "#f1fa8c"
    status_success = "#50fa7b"
    status_failed = "#ff5555"
    status_running = "#8be9fd"
    status_pending = "#f1fa8c"
    status_draft = "#6272a4"
    status_merged = "#bd93f9"
    open_mr = "#8be9fd"
    merged_mr = "#bd93f9"
    closed_mr = "#ff5555"
    keyword = "#ff5555"
    code = "#50fa7b"
    code_bg = "#1e1f29"
    range_highlight = "#44475a"
    ```

=== "Gruvbox Dark"
    ```toml
    [theme]
    border = "#b8bb26"
    active_border = "#b8bb26"
    inactive_border = "#928374"
    highlight_bg = "#458588"
    accent = "#83a598"
    title_fg = "#b8bb26"
    header = "#fe8019"
    header1 = "#8ec07c"
    header2 = "#d3869b"
    author = "#d3869b"
    dim = "#928374"
    project = "#8ec07c"
    mr = "#d3869b"
    issue = "#b8bb26"
    shortcut_key = "#fe8019"
    shortcut_desc = "#ebdbb2"
    badge_bg = "#458588"
    badge_fg = "#b8bb26"
    chip_bg = "#b8bb26"
    chip_fg = "#282828"
    label_bg = "#928374"
    label_fg = "#282828"
    line_number_fg = "#928374"
    hunk_header_bg = "#458588"
    hunk_header_fg = "#b8bb26"
    diff_add_bg = "#b8bb26"
    diff_add_fg = "#282828"
    diff_del_bg = "#fb4934"
    diff_del_fg = "#282828"
    success = "#b8bb26"
    failed = "#fb4934"
    running = "#83a598"
    pending = "#fabd2f"
    status_success = "#b8bb26"
    status_failed = "#fb4934"
    status_running = "#83a598"
    status_pending = "#fabd2f"
    status_draft = "#928374"
    status_merged = "#d3869b"
    open_mr = "#83a598"
    merged_mr = "#d3869b"
    closed_mr = "#fb4934"
    keyword = "#fb4934"
    code = "#b8bb26"
    code_bg = "#282828"
    range_highlight = "#504945"
    ```

=== "GitHub Dark"
    ```toml
    [theme]
    border = "#2ea043"
    active_border = "#2ea043"
    inactive_border = "#6e7681"
    highlight_bg = "#1f6feb"
    accent = "#58a6ff"
    title_fg = "#3fb950"
    header = "#58a6ff"
    header1 = "#39c5bb"
    header2 = "#a371f7"
    author = "#a371f7"
    dim = "#6e7681"
    project = "#58a6ff"
    mr = "#a371f7"
    issue = "#3fb950"
    shortcut_key = "#58a6ff"
    shortcut_desc = "#f0f6fc"
    badge_bg = "#1f6feb"
    badge_fg = "#3fb950"
    chip_bg = "#2ea043"
    chip_fg = "#161b22"
    label_bg = "#6e7681"
    label_fg = "#161b22"
    line_number_fg = "#6e7681"
    hunk_header_bg = "#1f6feb"
    hunk_header_fg = "#3fb950"
    diff_add_bg = "#3fb950"
    diff_add_fg = "#161b22"
    diff_del_bg = "#f85149"
    diff_del_fg = "#161b22"
    success = "#3fb950"
    failed = "#f85149"
    running = "#58a6ff"
    pending = "#d29922"
    status_success = "#3fb950"
    status_failed = "#f85149"
    status_running = "#58a6ff"
    status_pending = "#d29922"
    status_draft = "#6e7681"
    status_merged = "#a371f7"
    open_mr = "#58a6ff"
    merged_mr = "#a371f7"
    closed_mr = "#f85149"
    keyword = "#f85149"
    code = "#3fb950"
    code_bg = "#161b22"
    range_highlight = "#30363d"
    ```

=== "Catppuccin Latte"
    ```toml
    [theme]
    border = "#40a02b"
    active_border = "#40a02b"
    inactive_border = "#9ca0b0"
    highlight_bg = "#1e66f5"
    accent = "#1e66f5"
    title_fg = "#40a02b"
    header = "#1e66f5"
    header1 = "#209fb5"
    header2 = "#8839ef"
    author = "#8839ef"
    dim = "#9ca0b0"
    project = "#209fb5"
    mr = "#8839ef"
    issue = "#40a02b"
    shortcut_key = "#1e66f5"
    shortcut_desc = "#4c4f69"
    badge_bg = "#1e66f5"
    badge_fg = "#40a02b"
    chip_bg = "#40a02b"
    chip_fg = "#e6e9ef"
    label_bg = "#9ca0b0"
    label_fg = "#e6e9ef"
    line_number_fg = "#9ca0b0"
    hunk_header_bg = "#1e66f5"
    hunk_header_fg = "#40a02b"
    diff_add_bg = "#40a02b"
    diff_add_fg = "#e6e9ef"
    diff_del_bg = "#d20f39"
    diff_del_fg = "#e6e9ef"
    success = "#40a02b"
    failed = "#d20f39"
    running = "#1e66f5"
    pending = "#df8e1d"
    status_success = "#40a02b"
    status_failed = "#d20f39"
    status_running = "#1e66f5"
    status_pending = "#df8e1d"
    status_draft = "#9ca0b0"
    status_merged = "#7287fd"
    open_mr = "#1e66f5"
    merged_mr = "#7287fd"
    closed_mr = "#d20f39"
    keyword = "#d20f39"
    code = "#40a02b"
    code_bg = "#e6e9ef"
=== "Gruvbox Light"
    ```toml
    [theme]
    border = "#79740e"
    active_border = "#79740e"
    inactive_border = "#928374"
    highlight_bg = "#d5c4a1"
    accent = "#076678"
    title_fg = "#282828"
    header = "#af3a03"
    header1 = "#427b58"
    header2 = "#8f3f71"
    author = "#8f3f71"
    dim = "#7c6f64"
    project = "#076678"
    mr = "#8f3f71"
    issue = "#79740e"
    shortcut_key = "#af3a03"
    shortcut_desc = "#282828"
    badge_bg = "#d5c4a1"
    badge_fg = "#076678"
    chip_bg = "#d5c4a1"
    chip_fg = "#79740e"
    label_bg = "#d5c4a1"
    label_fg = "#282828"
    line_number_fg = "#7c6f64"
    hunk_header_bg = "#d5c4a1"
    hunk_header_fg = "#076678"
    diff_add_bg = "#d5c4a1"
    diff_add_fg = "#79740e"
    diff_del_bg = "#ebdbb2"
    diff_del_fg = "#cc241d"
    success = "#79740e"
    failed = "#cc241d"
    running = "#076678"
    pending = "#b57614"
    status_success = "#79740e"
    status_failed = "#cc241d"
    status_running = "#076678"
    status_pending = "#b57614"
    status_draft = "#7c6f64"
    status_merged = "#8f3f71"
    open_mr = "#076678"
    merged_mr = "#8f3f71"
    closed_mr = "#cc241d"
    keyword = "#cc241d"
    code = "#79740e"
    code_bg = "#ebdbb2"
    range_highlight = "#d5c4a1"
    ```

=== "GitHub Light"
    ```toml
    [theme]
    border = "#1a7f37"
    active_border = "#1a7f37"
    inactive_border = "#6e7781"
    highlight_bg = "#0969da"
    accent = "#0969da"
    title_fg = "#1a7f37"
    header = "#0969da"
    header1 = "#0550ae"
    header2 = "#8250df"
    author = "#8250df"
    dim = "#6e7781"
    project = "#0969da"
    mr = "#8250df"
    issue = "#1a7f37"
    shortcut_key = "#0969da"
    shortcut_desc = "#1f2328"
    badge_bg = "#0969da"
    badge_fg = "#1a7f37"
    chip_bg = "#1a7f37"
    chip_fg = "#f6f8fa"
    label_bg = "#6e7781"
    label_fg = "#f6f8fa"
    line_number_fg = "#6e7781"
    hunk_header_bg = "#0969da"
    hunk_header_fg = "#1a7f37"
    diff_add_bg = "#1a7f37"
    diff_add_fg = "#f6f8fa"
    diff_del_bg = "#cf222e"
    diff_del_fg = "#f6f8fa"
    success = "#1a7f37"
    failed = "#cf222e"
    running = "#0969da"
    pending = "#9a6700"
    status_success = "#1a7f37"
    status_failed = "#cf222e"
    status_running = "#0969da"
    status_pending = "#9a6700"
    status_draft = "#6e7681"
    status_merged = "#8250df"
    open_mr = "#0969da"
    merged_mr = "#8250df"
    closed_mr = "#cf222e"
    keyword = "#cf222e"
    code = "#1a7f37"
    code_bg = "#f6f8fa"
    range_highlight = "#ddf4ff"
    ```
