"""Hermetic setup controls; candidate bytes never substitute for actual media."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("direct_setup", Path(__file__).with_name("setup-motion-release-backends.py"))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class DirectBackendTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="opencut-direct-backends-")
        self.addCleanup(self.temporary.cleanup)
        self.installation = Path(self.temporary.name)
        self.directory = self.installation / "lib/ffmpeg/tools/ffmpeg-7.1.1/bin"
        self.directory.mkdir(parents=True)
        # Split the actual owner marker across the streaming boundary.
        contents = b"MZ" + b"x" * 65530 + b"%s version "
        for name in ("ffmpeg.exe", "ffprobe.exe"):
            (self.directory / name).write_bytes(contents)
        self.output = self.installation / "github-path"
        self.output.write_text("prior-path\n")

    def test_unique_direct_pair_and_atomic_path_publication(self):
        self.assertEqual(MODULE.publish(self.installation, self.output), self.directory.resolve())
        self.assertEqual(self.output.read_text(), f"prior-path\n{self.directory.resolve()}\n")

    def refuse(self, pattern):
        with self.assertRaisesRegex((RuntimeError, FileNotFoundError), pattern):
            MODULE.publish(self.installation, self.output)
        self.assertEqual(self.output.read_text(), "prior-path\n")

    def test_missing_and_ambiguous_pair(self):
        (self.directory / "ffprobe.exe").unlink()
        self.refuse("ffprobe")
        (self.directory / "ffprobe.exe").write_bytes(b"MZ%s version ")
        duplicate = self.directory.parent / "second/ffmpeg.exe"
        duplicate.parent.mkdir()
        duplicate.write_bytes(b"MZ%s version ")
        self.refuse("unique")

    def test_non_native_and_forwarding_files(self):
        for contents in (b"#!/bin/sh\n%s version ", b"MZnative forwarding wrapper", b"MZ"):
            (self.directory / "ffmpeg.exe").write_bytes(contents)
            self.refuse("direct installed native")

    def test_absent_directory_and_empty_inventory(self):
        (self.directory / "ffmpeg.exe").unlink()
        self.refuse("unique")
        with self.assertRaises(FileNotFoundError):
            MODULE.backend_directory(self.installation / "missing")

    def test_relative_installation_and_path_output(self):
        with self.assertRaisesRegex(RuntimeError, "absolute Chocolatey"):
            MODULE.backend_directory(Path("relative"))
        with self.assertRaisesRegex(RuntimeError, "GitHub PATH"):
            MODULE.publish(self.installation, Path("relative-path-output"))
        self.assertEqual(self.output.read_text(), "prior-path\n")

    def test_unsafe_newline_and_nonfile_candidate(self):
        (self.directory / "ffprobe.exe").unlink()
        (self.directory / "ffprobe.exe").mkdir()
        self.refuse("direct installed native")
        unsafe = self.installation / "lib/ffmpeg/tools/unsafe\npath"
        unsafe.mkdir()
        for name in ("ffmpeg.exe", "ffprobe.exe"):
            (unsafe / name).write_bytes(b"MZ%s version ")
        (self.directory / "ffmpeg.exe").unlink()
        self.refuse("Unsafe")

    def test_candidate_cannot_escape_installed_package(self):
        outside = self.installation / "outside.exe"
        outside.write_bytes(b"MZ%s version ")
        candidate = (self.directory / "ffprobe.exe").resolve()
        outside = outside.resolve()
        resolve = Path.resolve
        def escaped(path, *args, **kwargs):
            return outside if path == candidate else resolve(path, *args, **kwargs)
        # Inject the canonical target to exercise escape refusal on every OS
        # without requiring privileged Windows junction/symlink creation.
        with patch.object(Path, "resolve", escaped):
            self.refuse("direct installed native")


if __name__ == "__main__":
    unittest.main()
