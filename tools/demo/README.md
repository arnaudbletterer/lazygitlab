# Demo assets

Screenshots and the demo animation are generated against a fictional GitLab, so
no real instance, name or project ever appears in them.

```bash
make demo
```

This builds the binary, creates `tools/demo/.venv` with Pillow (needs network
the first time), and writes:

- `docs/assets/marketing/{dashboard,mr-review,pipelines,boards}.png` (gitignored)
- `docs/assets/marketing/demo.gif` (about 18 seconds, well under 2 MB)

Requires `python3` and `ffmpeg`. The font is `docs/assets/fonts/JetBrainsMono-Regular.ttf`
(not a Nerd Font); missing glyphs fall back to system fonts when present.

The docs workflow (`scripts/build_docs.py`) runs the same generation after
`cargo build`, copies the files to `site/assets/marketing/` and fails if any is
missing or empty. Published at
`https://abletterer.github.io/lazygitlab/assets/marketing/<file>`.

## How it works

- `mock_gitlab.py` serves the GitLab REST v4 endpoints the app calls, from the
  JSON fixtures in `fixtures/`. The world is `acme/widgets` with the users ada,
  grace, alan, margaret and edsger. Set `MOCK_DEBUG=1` to log routes it does not
  handle.
- `scenarios/*.txt` are headless event scripts (see the README section
  "Headless mode"). Two extra directives are expanded by `make_demo.py`:
  `wait` / `wait:N` insert idle ticks so network data can arrive, and
  `# args:` / `# size: WxH` set CLI arguments and the terminal size.
- `make_demo.py` runs each scenario with `--headless --script --frames-dir`
  against the mock (with an empty `HOME` and working directory, so no local
  config or token is read), then renders the last frame to a PNG. The `demo`
  scenario is rendered frame by frame and assembled with ffmpeg.
- `render.py` draws text frames with Pillow using the Catppuccin Mocha palette
  and a monospace font found on the system (JetBrains Mono, DejaVu Sans Mono or
  Liberation Mono). Headless frames carry no colour, so colours are inferred
  from glyphs and tokens (diff markers, `@user`, `!123`, status words).

## Adding a screen

Add or extend fixtures, add `scenarios/<name>.txt`, and list the name in
`SHOT_NAMES` in `make_demo.py`. Run a subset with
`tools/demo/.venv/bin/python tools/demo/make_demo.py boards demo`.
