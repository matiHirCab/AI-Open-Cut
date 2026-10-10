"""Hermetic setup failures; actual native execution is a separate mandatory check."""
import importlib.util
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("platform_setup", Path(__file__).with_name("setup-platform-renderer.py"))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
REPOSITORY = Path(__file__).resolve().parents[1]

class PlatformSetupTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="opencut-platform-setup-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repository = self.root / "repository"
        destination = self.repository / "crates/editor-core/resources/fonts"
        destination.parent.mkdir(parents=True)
        shutil.copytree(REPOSITORY / "crates/editor-core/resources/fonts", destination)

    def test_fixed_family_and_explicit_actual_tool_invocations(self):
        with patch.object(MODULE.shutil, "which", side_effect=lambda tool: f"/owned/{tool}"), patch.object(MODULE.subprocess, "run") as run:
            values = MODULE.prepare(self.repository, self.root / "inputs")
        self.assertEqual(run.call_args_list[0].args[0], ["/owned/ffmpeg", "-version"])
        self.assertEqual(run.call_args_list[1].args[0], ["/owned/ffprobe", "-version"])
        for call in run.call_args_list:
            self.assertTrue(call.kwargs["check"])
        for face in MODULE.FACES:
            self.assertTrue((Path(values["OPENCUT_TEST_FONT_PATH"]).parent / face).is_file())

    def test_tampered_font_is_not_rebaselined(self):
        (self.repository / "crates/editor-core/resources/fonts/DejaVuSans.ttf").write_bytes(b"wrong")
        with self.assertRaisesRegex(RuntimeError, "checksum mismatch"):
            MODULE.prepare(self.repository, self.root / "inputs")

    def test_missing_font_is_required(self):
        (self.repository / "crates/editor-core/resources/fonts/DejaVuSans-Bold.ttf").unlink()
        with self.assertRaises(FileNotFoundError):
            MODULE.prepare(self.repository, self.root / "inputs")

    def test_missing_tool_is_required(self):
        with patch.object(MODULE.shutil, "which", return_value=None), self.assertRaisesRegex(RuntimeError, "Required native tool unavailable"):
            MODULE.prepare(self.repository, self.root / "inputs")

    def test_unusable_tool_is_required(self):
        with patch.object(MODULE.shutil, "which", return_value="/owned/tool"), patch.object(MODULE.subprocess, "run", side_effect=MODULE.subprocess.CalledProcessError(1, "tool")), self.assertRaises(MODULE.subprocess.CalledProcessError):
            MODULE.prepare(self.repository, self.root / "inputs")

if __name__ == "__main__":
    unittest.main()
