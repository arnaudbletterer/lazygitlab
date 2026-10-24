#!/usr/bin/env python3
"""Regenerate docs/assets screenshots and demo.gif from the fictional mock.

Starts mock_gitlab.py, drives the headless TUI with each scenario script in
./scenarios, and renders the resulting frames. Run through `make demo`.
"""
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import render

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
BIN = ROOT / "target" / "debug" / "lazygitlab"
SHOTS = ROOT / "docs" / "assets" / "screenshots"
GIF = ROOT / "docs" / "assets" / "demo.gif"
WIDTH, HEIGHT = 150, 40
PAD_TICKS = 100
SHOT_NAMES = ["dashboard", "mr-review", "pipelines", "boards"]
GIF_SECONDS = 18.0


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def wait_for(port):
    for _ in range(50):
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.2).close()
            return
        except OSError:
            time.sleep(0.1)
    raise SystemExit("mock server did not start")


def expand(script):
    out, args, size = [], [], (WIDTH, HEIGHT)
    for line in script.splitlines():
        s = line.strip()
        if s.startswith("# args:"):
            args = s[len("# args:"):].split()
        elif s.startswith("# size:"):
            w, h = s[len("# size:"):].lower().split("x")
            size = (int(w), int(h))
        elif s == "wait":
            out += ["<F12>"] * PAD_TICKS
        elif s.startswith("wait:"):
            out += ["<F12>"] * int(s[5:])
        else:
            out.append(line)
    return "\n".join(out) + "\nq\n", args, size


def run_scenario(name, port, work):
    script, args, size = expand((HERE / "scenarios" / f"{name}.txt").read_text())
    sdir = work / name
    sdir.mkdir()
    (sdir / "script.txt").write_text(script)
    home = sdir / "home"
    home.mkdir()
    env = {
        "PATH": os.environ.get("PATH", ""),
        "HOME": str(home),
        "GITLAB_URL": f"http://127.0.0.1:{port}",
        "GITLAB_TOKEN": "demo-token",
    }
    cmd = [str(BIN), "--headless", "--width", str(size[0]), "--height", str(size[1]),
           "--script", str(sdir / "script.txt"), "--frames-dir", str(sdir / "frames")] + args
    subprocess.run(cmd, cwd=sdir, env=env, check=True, timeout=120,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    frames = sorted((sdir / "frames").glob("frame_*.txt"))
    if not frames:
        raise SystemExit(f"scenario {name} produced no frames")
    return frames


def build_gif(frames, work):
    texts = [f.read_text(encoding="utf-8") for f in frames]
    keep = [t for t in texts[:-1] if "LOADING" not in t.split("\n", 1)[0]] + [texts[-1]]
    per = min(1.2, max(0.35, (GIF_SECONDS - 2.5) / max(1, len(keep) - 1)))
    gdir = work / "gif"
    gdir.mkdir()
    lines = []
    for i, t in enumerate(keep):
        png = gdir / f"f{i:04d}.png"
        render.render(t, png, scale=0.7)
        lines += [f"file '{png}'", f"duration {2.5 if i == len(keep) - 1 else per:.3f}"]
    lines.append(f"file '{gdir / f'f{len(keep) - 1:04d}.png'}'")
    (gdir / "list.txt").write_text("\n".join(lines) + "\n")
    GIF.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0",
         "-i", str(gdir / "list.txt"),
         "-vf", "split[a][b];[a]palettegen=max_colors=64[p];[b][p]paletteuse=dither=none",
         "-loop", "0", str(GIF)],
        check=True)
    print(f"{GIF.relative_to(ROOT)}: {len(keep)} frames, {GIF.stat().st_size // 1024} KiB")


def main():
    if not BIN.exists():
        raise SystemExit(f"build the binary first: cargo build ({BIN})")
    port = free_port()
    mock = subprocess.Popen([sys.executable, str(HERE / "mock_gitlab.py"), str(port)])
    work = Path(tempfile.mkdtemp(prefix="lazygitlab-demo-"))
    try:
        wait_for(port)
        SHOTS.mkdir(parents=True, exist_ok=True)
        only = sys.argv[1:] or None
        for name in [n for n in SHOT_NAMES if only is None or n in only]:
            frames = run_scenario(name, port, work)
            out = SHOTS / f"{name}.png"
            render.render(frames[-1].read_text(encoding="utf-8"), out)
            print(f"{out.relative_to(ROOT)}: {len(frames)} frames")
        if only is None or "demo" in only:
            build_gif(run_scenario("demo", port, work), work)
    finally:
        mock.terminate()
        keep = os.environ.get("DEMO_KEEP")
        if keep:
            shutil.copytree(work, keep, dirs_exist_ok=True)
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
