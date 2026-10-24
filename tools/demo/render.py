#!/usr/bin/env python3
"""Render lazygitlab headless text frames to PNG (Catppuccin Mocha).

Frames carry no colour, so colours are inferred from the glyphs and tokens on
each line. Requires Pillow (see make demo, which installs it in .venv).
"""
import re
import sys
import unicodedata

from PIL import Image, ImageDraw, ImageFont

BASE = (30, 30, 46)
MANTLE = (24, 24, 37)
SURFACE0 = (49, 50, 68)
SURFACE2 = (88, 91, 112)
TEXT = (205, 214, 244)
DIM = (147, 153, 178)
BLUE = (137, 180, 250)
SKY = (137, 220, 235)
GREEN = (166, 227, 161)
RED = (243, 139, 168)
YELLOW = (249, 226, 175)
MAUVE = (203, 166, 247)
PINK = (245, 194, 231)
ADD_BG = (36, 52, 46)
DEL_BG = (58, 38, 48)

FONT_PATHS = [
    "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf",
    "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
]
FALLBACK_PATHS = [
    "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
]
EMOJI_FONT = "/usr/share/fonts/noto/NotoColorEmoji.ttf"
BORDERS = set("─│╭╮╰╯┌┐└┘├┤┬┴┼")
FONT_SIZE = 15

TOKENS = [
    (re.compile(r"!\d+"), MAUVE),
    (re.compile(r"#\d+"), GREEN),
    (re.compile(r"@\w+"), PINK),
    (re.compile(r"\b(failed|FAILED|ERROR|Job failed)\b"), RED),
    (re.compile(r"\b(success|succeeded|mergeable|passed|approved|ok)\b"), GREEN),
    (re.compile(r"\b(running|LIVE|LOADING)\b"), BLUE),
    (re.compile(r"\b(pending|created|Draft|draft)\b"), YELLOW),
    (re.compile(r"\b(skipped|canceled|Loading)\b"), DIM),
    (re.compile(r"\[[^\]\s][^\]]{0,24}\]"), SKY),
]
VERSION = re.compile(r"\s*v\d+\.\d+\.\d+\S*\s*$")


def load_font():
    for path in FONT_PATHS:
        try:
            return ImageFont.truetype(path, FONT_SIZE)
        except OSError:
            continue
    raise SystemExit("no monospace font found; install JetBrains Mono, DejaVu Sans Mono or Liberation Mono")


def load_fallbacks():
    out = []
    for path in FALLBACK_PATHS:
        try:
            out.append(ImageFont.truetype(path, FONT_SIZE))
        except OSError:
            continue
    return out


def _sig(font, ch):
    m = font.getmask(ch)
    return (m.size, m.getbbox(), tuple(m.histogram()))


def has_glyph(font, ch):
    return _sig(font, ch) != _sig(font, "\U0010fffe")


def pick_font(font, fallbacks, ch):
    if has_glyph(font, ch):
        return font
    for fb in fallbacks:
        if has_glyph(fb, ch):
            return fb
    return font


BOX = {
    "─": (1, 1, 0, 0), "│": (0, 0, 1, 1),
    "╭": (0, 1, 0, 1), "┌": (0, 1, 0, 1), "╮": (1, 0, 0, 1), "┐": (1, 0, 0, 1),
    "╰": (0, 1, 1, 0), "└": (0, 1, 1, 0), "╯": (1, 0, 1, 0), "┘": (1, 0, 1, 0),
    "├": (0, 1, 1, 1), "┤": (1, 0, 1, 1), "┬": (1, 1, 0, 1), "┴": (1, 1, 1, 0),
    "┼": (1, 1, 1, 1),
}


def draw_box(draw, ch, px, py, cw, chh, colour):
    left, right, up, down = BOX[ch]
    cx, cy = px + cw // 2, py + chh // 2
    if left:
        draw.line([(px, cy), (cx, cy)], fill=colour)
    if right:
        draw.line([(cx, cy), (px + cw - 1, cy)], fill=colour)
    if up:
        draw.line([(cx, py), (cx, cy)], fill=colour)
    if down:
        draw.line([(cx, cy), (cx, py + chh - 1)], fill=colour)


def is_wide(ch):
    return unicodedata.east_asian_width(ch) in ("W", "F")


def style_line(y, line, last, split):
    n = len(line)
    fg = [TEXT] * n
    bg = [None] * n
    for i, ch in enumerate(line):
        if ch in BORDERS:
            fg[i] = SURFACE2
    if y == 0:
        fg = [BLUE] * n
    elif y == last:
        fg = [DIM] * n
    else:
        for rx, colour in TOKENS:
            for m in rx.finditer(line):
                for i in range(m.start(), m.end()):
                    if line[i] not in BORDERS:
                        fg[i] = colour
        for m in re.finditer(r"[╭╰][─ ]*([^─╮╯│]+?)\s*[─╮╯]", line):
            for i in range(m.start(1), m.end(1)):
                fg[i] = BLUE
    if y != 0 and y != last:
        bar = line.find("▎")
        if bar >= 0:
            left = line.rfind("│", 0, bar)
            right = line.find("│", bar)
            if right < 0:
                right = n
            for i in range(left + 1, right):
                bg[i] = SURFACE0
    if split and 5 < y < last - 1 and n > split + 2:
        region = line[split:]
        body = region.lstrip(" >")
        tag = body[:1]
        cursor = region[:1] == ">"
        start = split + 1
        if tag == "+":
            for i in range(start, n):
                bg[i] = ADD_BG
                fg[i] = GREEN
        elif tag == "-":
            for i in range(start, n):
                bg[i] = DEL_BG
                fg[i] = RED
        elif body[:2] == "@@":
            for i in range(start, n):
                fg[i] = SKY
        elif "[+]" in region or "[-]" in region or "RESOLVED" in region or re.match(r"\s+@\w+:", region):
            for i in range(split - 1, n):
                bg[i] = SURFACE0
                fg[i] = YELLOW
        if cursor:
            for i in range(split, n):
                if bg[i] is None:
                    bg[i] = SURFACE0
            fg[split] = BLUE
        if line.startswith("│>>"):
            for i in range(1, split - 2):
                bg[i] = SURFACE0
                fg[i] = BLUE
    return fg, bg


def render(text, out_path, scale=1.0):
    lines = text.split("\n")
    while lines and lines[-1] == "":
        lines.pop()
    lines = [VERSION.sub("", ln) if i == len(lines) - 1 else ln for i, ln in enumerate(lines)]
    width = max(len(ln) for ln in lines)
    font = load_font()
    fallbacks = load_fallbacks()
    cw = round(font.getlength("M"))
    ch = int(FONT_SIZE * 1.4)
    pad = 12
    img = Image.new("RGB", (width * cw + 2 * pad, len(lines) * ch + 2 * pad), BASE)
    draw = ImageDraw.Draw(img)
    try:
        emoji_font = ImageFont.truetype(EMOJI_FONT, 109)
    except OSError:
        emoji_font = None
    diff_mode = lines[-1].startswith(" DIFF")
    split = 0
    if diff_mode:
        header = next((ln for ln in lines if "Files" in ln and "╮" in ln), "")
        split = header.index("╮") + 1 if header else 0
    last = len(lines) - 1
    for y, line in enumerate(lines):
        line = line.ljust(width)
        fg, bg = style_line(y, line, last, split)
        py = pad + y * ch
        x = 0
        while x < width:
            c = line[x]
            span = 2 if is_wide(c) else 1
            px = pad + x * cw
            if bg[x] is not None:
                draw.rectangle([px, py, px + cw * span - 1, py + ch - 1], fill=bg[x])
            if c != " ":
                if span == 2:
                    if emoji_font is not None:
                        tile = Image.new("RGBA", (136, 128), (0, 0, 0, 0))
                        ImageDraw.Draw(tile).text((0, 0), c, font=emoji_font, embedded_color=True)
                        tile = tile.resize((cw * 2 - 2, ch - 2), Image.LANCZOS)
                        img.paste(tile, (px + 1, py + 1), tile)
                elif c in BOX:
                    draw_box(draw, c, px, py, cw, ch, fg[x])
                else:
                    draw.text((px, py + 2), c, font=pick_font(font, fallbacks, c), fill=fg[x])
            x += span
    if scale != 1.0:
        img = img.resize((int(img.width * scale), int(img.height * scale)), Image.LANCZOS)
    img.save(out_path)


if __name__ == "__main__":
    with open(sys.argv[1], encoding="utf-8") as fh:
        render(fh.read(), sys.argv[2])
