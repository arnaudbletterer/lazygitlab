import contextlib
import io
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import update_packaging as up  # noqa: E402

REPO_ROOT = HERE.parent
TAG = "v1.2.3"
H = {
    "x86_64-unknown-linux-gnu": "a" * 64,
    "aarch64-apple-darwin": "b" * 64,
    "x86_64-apple-darwin": "c" * 64,
    "x86_64-pc-windows-msvc": "d" * 64,
}
SUMS = "".join(
    f"{H[t]}  lazygitlab-{TAG}-{t}.{ext}\n" for t, ext in up.TARGETS.items()
)


class UpdatePackagingTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp)
        for rel in up.FILES:
            dest = self.tmp / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(REPO_ROOT / rel, dest)
        self.sums_file = self.tmp / "SHA256SUMS"
        self.sums_file.write_text(SUMS)

    def run_main(self, *extra):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = up.main([TAG, "--sums-file", str(self.sums_file), "--root", str(self.tmp), *extra])
        return code, out.getvalue()

    def read(self, rel):
        return (self.tmp / rel).read_text()

    def test_parse_sums_handles_binary_marker(self):
        digest = "e" * 64
        self.assertEqual(up.parse_sums(f"{digest} *file.zip\nnoise\n"), {"file.zip": digest})

    def test_updates_all_files(self):
        code, _ = self.run_main()
        self.assertEqual(code, 0)
        brew = self.read("packaging/homebrew/lazygitlab.rb")
        self.assertIn('version "1.2.3"', brew)
        for t in ("x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-apple-darwin"):
            self.assertIn(H[t], brew)
        aur = self.read("packaging/aur/lazygitlab-bin/PKGBUILD")
        self.assertIn("pkgver=1.2.3", aur)
        self.assertIn(f"sha256sums_x86_64=('{H['x86_64-unknown-linux-gnu']}')", aur)
        self.assertIn("pkgver=1.2.3", self.read("packaging/aur/lazygitlab/PKGBUILD"))
        scoop_text = self.read("packaging/scoop/lazygitlab.json")
        scoop = json.loads(scoop_text)
        self.assertEqual(scoop["version"], "1.2.3")
        arch = scoop["architecture"]["64bit"]
        self.assertEqual(arch["hash"], H["x86_64-pc-windows-msvc"])
        self.assertTrue(arch["url"].endswith("/v1.2.3/lazygitlab-v1.2.3-x86_64-pc-windows-msvc.zip"))
        self.assertEqual(scoop["extract_dir"], "lazygitlab-v1.2.3-x86_64-pc-windows-msvc")
        self.assertIn("$version", scoop["autoupdate"]["architecture"]["64bit"]["url"])

    def test_idempotent(self):
        self.run_main()
        _, out = self.run_main()
        self.assertIn("nothing to change", out)

    def test_dry_run_writes_nothing(self):
        before = {rel: self.read(rel) for rel in up.FILES}
        _, out = self.run_main("--dry-run")
        self.assertIn("would update", out)
        self.assertEqual(before, {rel: self.read(rel) for rel in up.FILES})

    def test_missing_hash_fails(self):
        self.sums_file.write_text(SUMS.splitlines()[0] + "\n")
        with self.assertRaises(SystemExit):
            self.run_main()

    def test_rejects_bad_tag(self):
        with self.assertRaises(SystemExit):
            up.main(["1.2", "--sums-file", str(self.sums_file)])


if __name__ == "__main__":
    unittest.main()
