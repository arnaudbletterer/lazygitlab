# Theming & Color Schemes

`lazygitlab` comes with a rich set of built-in classical TUI themes and supports complete, granular color customization via `theme.toml`.

---

## 🎨 Interactive Theme Visualizer

Select a theme below to preview how it looks live in `lazygitlab`:

<div class="theme-chips" id="theme-selector">
  <button class="theme-chip active" onclick="applyTheme('catppuccin-mocha')">Catppuccin Mocha</button>
  <button class="theme-chip" onclick="applyTheme('tokyo-night')">Tokyo Night</button>
  <button class="theme-chip" onclick="applyTheme('nord')">Nord</button>
  <button class="theme-chip" onclick="applyTheme('dracula')">Dracula</button>
  <button class="theme-chip" onclick="applyTheme('gruvbox-dark')">Gruvbox Dark</button>
  <button class="theme-chip" onclick="applyTheme('github-dark')">GitHub Dark</button>
  <button class="theme-chip" onclick="applyTheme('catppuccin-latte')">Catppuccin Latte (Light)</button>
  <button class="theme-chip" onclick="applyTheme('github-light')">GitHub Light (Light)</button>
</div>

<div id="mock-tui" style="border-radius: 12px; padding: 16px; font-family: 'JetBrains Mono', 'SF Mono', monospace; font-size: 12.5px; line-height: 1.4; transition: all 0.3s ease; box-shadow: 0 16px 36px rgba(0,0,0,0.35); border: 1px solid rgba(255,255,255,0.12); margin-bottom: 2rem;">
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
  <div style="display: grid; grid-template-columns: 220px 1fr; gap: 14px;">
    <!-- Side Panel -->
    <div style="border: 1px solid var(--tui-border); border-radius: 6px; padding: 8px;">
      <div style="color: var(--tui-header1); font-weight: bold; margin-bottom: 6px;">📋 MERGE REQUESTS</div>
      <div style="background: var(--tui-highlight); color: #ffffff; padding: 2px 6px; border-radius: 4px; font-weight: 500;">!124 feat(diff): split review</div>
      <div style="color: var(--tui-dim); padding: 2px 6px;">!123 fix(pipeline): cancel</div>
      <div style="color: var(--tui-dim); padding: 2px 6px;">!122 perf(tree): fast load</div>
      <div style="margin-top: 10px; color: var(--tui-header2); font-weight: bold;">⚡ STATUS</div>
      <div style="color: var(--tui-success);">✔ 14 Passed</div>
      <div style="color: var(--tui-running);">⏳ 1 Running</div>
    </div>

    <!-- Detail Panel -->
    <div style="border: 1px solid var(--tui-border); border-radius: 6px; padding: 10px;">
      <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
        <span style="color: var(--tui-mr); font-weight: bold;">!124 feat(diff): add syntax-highlighted diff review</span>
        <span style="background: var(--tui-success); color: #000; font-size: 11px; font-weight: bold; padding: 1px 6px; border-radius: 3px;">✔ APPROVED</span>
      </div>
      <div style="color: var(--tui-dim); margin-bottom: 8px;">
        Author: <span style="color: var(--tui-author);">@abletterer</span> • Target: <span style="color: var(--tui-project);">master</span>
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
  <div style="margin-top: 12px; padding-top: 8px; border-top: 1px solid var(--tui-border); display: flex; gap: 12px; font-size: 11.5px;">
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
    bg: '#1e1e2e', border: '#a6e3a1', header: '#89b4fa', header1: '#89dceb', header2: '#cba6f7',
    dim: '#6c7086', highlight: '#89b4fa', mr: '#cba6f7', author: '#f5c2e7', project: '#89dceb',
    success: '#a6e3a1', running: '#89b4fa', codeBg: '#181825', keyword: '#f38ba8', code: '#a6e3a1',
    rangeHl: '#313244', key: '#89dceb', desc: '#cdd6f4'
  },
  'tokyo-night': {
    bg: '#1a1b26', border: '#73daca', header: '#7aa2f7', header1: '#7dcfff', header2: '#bb9af7',
    dim: '#565f89', highlight: '#3d59a1', mr: '#bb9af7', author: '#ff007c', project: '#7dcfff',
    success: '#9ece6a', running: '#7aa2f7', codeBg: '#16161e', keyword: '#f7768e', code: '#9ece6a',
    rangeHl: '#292e42', key: '#7dcfff', desc: '#c0caf5'
  },
  'nord': {
    bg: '#2e3440', border: '#88c0d0', header: '#81a1c1', header1: '#8fbcbb', header2: '#b48ead',
    dim: '#616e88', highlight: '#5e81ac', mr: '#b48ead', author: '#d8dee9', project: '#88c0d0',
    success: '#a3be8c', running: '#81a1c1', codeBg: '#242933', keyword: '#bf616a', code: '#a3be8c',
    rangeHl: '#434c5e', key: '#88c0d0', desc: '#e5e9f0'
  },
  'dracula': {
    bg: '#282a36', border: '#bd93f9', header: '#bd93f9', header1: '#8be9fd', header2: '#ff79c6',
    dim: '#6272a4', highlight: '#6272a4', mr: '#ff79c6', author: '#50fa7b', project: '#8be9fd',
    success: '#50fa7b', running: '#8be9fd', codeBg: '#1e1f29', keyword: '#ff5555', code: '#50fa7b',
    rangeHl: '#44475a', key: '#8be9fd', desc: '#f8f8f2'
  },
  'gruvbox-dark': {
    bg: '#282828', border: '#b8bb26', header: '#fe8019', header1: '#8ec07c', header2: '#d3869b',
    dim: '#928374', highlight: '#458588', mr: '#d3869b', author: '#fabd2f', project: '#8ec07c',
    success: '#b8bb26', running: '#83a598', codeBg: '#1d2021', keyword: '#fb4934', code: '#b8bb26',
    rangeHl: '#504945', key: '#fe8019', desc: '#ebdbb2'
  },
  'github-dark': {
    bg: '#0d1117', border: '#30363d', header: '#58a6ff', header1: '#39c5bb', header2: '#bc8cff',
    dim: '#6e7681', highlight: '#1f6feb', mr: '#bc8cff', author: '#d2a8ff', project: '#58a6ff',
    success: '#3fb950', running: '#58a6ff', codeBg: '#161b22', keyword: '#ff7b72', code: '#7ee787',
    rangeHl: '#21262d', key: '#58a6ff', desc: '#c9d1d9'
  },
  'catppuccin-latte': {
    bg: '#eff1f5', border: '#40a02b', header: '#1e66f5', header1: '#209fb5', header2: '#8839ef',
    dim: '#9ca0b0', highlight: '#1e66f5', mr: '#8839ef', author: '#ea76cb', project: '#209fb5',
    success: '#40a02b', running: '#1e66f5', codeBg: '#e6e9ef', keyword: '#d20f39', code: '#40a02b',
    rangeHl: '#ccd0da', key: '#1e66f5', desc: '#4c4f69'
  },
  'github-light': {
    bg: '#ffffff', border: '#d0d7de', header: '#0969da', header1: '#0550ae', header2: '#8250df',
    dim: '#6e7781', highlight: '#0969da', mr: '#8250df', author: '#953800', project: '#0550ae',
    success: '#1a7f37', running: '#0969da', codeBg: '#f6f8fa', keyword: '#cf222e', code: '#1a7f37',
    rangeHl: '#ddf4ff', key: '#0969da', desc: '#1f2328'
  }
};

function applyTheme(name) {
  const t = themes[name];
  if (!t) return;
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
  box.style.setProperty('--tui-code-bg', t.codeBg);
  box.style.setProperty('--tui-keyword', t.keyword);
  box.style.setProperty('--tui-code', t.code);
  box.style.setProperty('--tui-range-hl', t.rangeHl);
  box.style.setProperty('--tui-key', t.key);
  box.style.setProperty('--tui-desc', t.desc);

  document.querySelectorAll('.theme-chip').forEach(el => el.classList.remove('active'));
  event.target.classList.add('active');
}

// Initial default
applyTheme('catppuccin-mocha');
</script>

---

## ⚙️ Selecting a Preset Theme

To activate any preset theme, create or edit `theme.toml` in your working directory or `~/.config/lazygitlab/theme.toml`:

```toml
theme = "tokyo-night"
```

Available preset names:
* `"catppuccin-mocha"` (Default)
* `"catppuccin-latte"` (Light mode)
* `"tokyo-night"`
* `"nord"`
* `"dracula"`
* `"gruvbox-dark"`
* `"github-dark"`
* `"github-light"` (Light mode)

---

## 🛠️ Complete Preset Configurations

=== "Catppuccin Mocha"
    ```toml
    [theme]
    border = "#a6e3a1"
    highlight_bg = "#89b4fa"
    author = "#f5c2e7"
    dim = "#6c7086"
    success = "#a6e3a1"
    failed = "#f38ba8"
    running = "#89b4fa"
    pending = "#f9e2af"
    open_mr = "#89b4fa"
    merged_mr = "#cba6f7"
    closed_mr = "#f38ba8"
    header = "#89b4fa"
    header1 = "#89dceb"
    header2 = "#cba6f7"
    project = "#89dceb"
    issue = "#a6e3a1"
    mr = "#cba6f7"
    shortcut_key = "#89dceb"
    shortcut_desc = "#cdd6f4"
    keyword = "#f38ba8"
    code = "#a6e3a1"
    code_bg = "#181825"
    range_highlight = "#313244"
    ```

=== "Tokyo Night"
    ```toml
    [theme]
    border = "#73daca"
    highlight_bg = "#3d59a1"
    author = "#bb9af7"
    dim = "#565f89"
    success = "#9ece6a"
    failed = "#f7768e"
    running = "#7aa2f7"
    pending = "#e0af68"
    open_mr = "#7aa2f7"
    merged_mr = "#bb9af7"
    closed_mr = "#f7768e"
    header = "#7aa2f7"
    header1 = "#7dcfff"
    header2 = "#bb9af7"
    project = "#7dcfff"
    issue = "#9ece6a"
    mr = "#bb9af7"
    shortcut_key = "#7dcfff"
    shortcut_desc = "#c0caf5"
    keyword = "#f7768e"
    code = "#9ece6a"
    code_bg = "#16161e"
    range_highlight = "#292e42"
    ```

=== "Nord"
    ```toml
    [theme]
    border = "#88c0d0"
    highlight_bg = "#5e81ac"
    author = "#b48ead"
    dim = "#616e88"
    success = "#a3be8c"
    failed = "#bf616a"
    running = "#81a1c1"
    pending = "#ebcb8b"
    open_mr = "#81a1c1"
    merged_mr = "#b48ead"
    closed_mr = "#bf616a"
    header = "#88c0d0"
    header1 = "#8fbcbb"
    header2 = "#b48ead"
    project = "#88c0d0"
    issue = "#a3be8c"
    mr = "#b48ead"
    shortcut_key = "#88c0d0"
    shortcut_desc = "#e5e9f0"
    keyword = "#bf616a"
    code = "#a3be8c"
    code_bg = "#242933"
    range_highlight = "#434c5e"
    ```

=== "Dracula"
    ```toml
    [theme]
    border = "#bd93f9"
    highlight_bg = "#6272a4"
    author = "#ff79c6"
    dim = "#6272a4"
    success = "#50fa7b"
    failed = "#ff5555"
    running = "#8be9fd"
    pending = "#f1fa8c"
    open_mr = "#8be9fd"
    merged_mr = "#bd93f9"
    closed_mr = "#ff5555"
    header = "#bd93f9"
    header1 = "#8be9fd"
    header2 = "#ff79c6"
    project = "#8be9fd"
    issue = "#50fa7b"
    mr = "#ff79c6"
    shortcut_key = "#8be9fd"
    shortcut_desc = "#f8f8f2"
    keyword = "#ff5555"
    code = "#50fa7b"
    code_bg = "#1e1f29"
    range_highlight = "#44475a"
    ```

=== "GitHub Dark"
    ```toml
    [theme]
    border = "#30363d"
    highlight_bg = "#1f6feb"
    author = "#bc8cff"
    dim = "#6e7681"
    success = "#3fb950"
    failed = "#f85149"
    running = "#58a6ff"
    pending = "#d29922"
    open_mr = "#58a6ff"
    merged_mr = "#bc8cff"
    closed_mr = "#f85149"
    header = "#58a6ff"
    header1 = "#39c5bb"
    header2 = "#bc8cff"
    project = "#58a6ff"
    issue = "#3fb950"
    mr = "#bc8cff"
    shortcut_key = "#58a6ff"
    shortcut_desc = "#c9d1d9"
    keyword = "#ff7b72"
    code = "#7ee787"
    code_bg = "#161b22"
    range_highlight = "#21262d"
    ```

=== "GitHub Light"
    ```toml
    [theme]
    border = "#d0d7de"
    highlight_bg = "#0969da"
    author = "#8250df"
    dim = "#6e7781"
    success = "#1a7f37"
    failed = "#cf222e"
    running = "#0969da"
    pending = "#9a6700"
    open_mr = "#0969da"
    merged_mr = "#8250df"
    closed_mr = "#cf222e"
    header = "#0969da"
    header1 = "#0550ae"
    header2 = "#8250df"
    project = "#0550ae"
    issue = "#1a7f37"
    mr = "#8250df"
    shortcut_key = "#0969da"
    shortcut_desc = "#1f2328"
    keyword = "#cf222e"
    code = "#1a7f37"
    code_bg = "#f6f8fa"
    range_highlight = "#ddf4ff"
    ```
