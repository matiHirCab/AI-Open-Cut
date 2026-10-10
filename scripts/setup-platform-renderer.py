"""Prepare owned native evidence inputs; fail if actual tools/fonts are unavailable."""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys

# Existing bundled DejaVu family; provenance and license live beside these files.
FACES = {
    "DejaVuSans.ttf": "7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954",
    "DejaVuSans-Bold.ttf": "e6476c1b80502924294eed40894c5b18e06c181444ca953e5334262df9c27724",
    "DejaVuSans-Oblique.ttf": "4af75fa16ee6d3ad43e1ecec41862c24954af26a55c6bb1ebb27bd486a50f5f4",
    "DejaVuSans-BoldOblique.ttf": "eb436dca0c2594b73d8b603b892e374fdfd8d885d25ffb4f18df4c4c0b49e50f",
}

def prepare(repository, root):
    root.mkdir(exist_ok=False)
    fonts = root / "fonts"
    fonts.mkdir()
    for face, expected in FACES.items():
        source = repository / "crates/editor-core/resources/fonts" / face
        contents = source.read_bytes()
        actual = hashlib.sha256(contents).hexdigest()
        if actual != expected:
            raise RuntimeError(f"Fixed font checksum mismatch: {face}")
        (fonts / face).write_bytes(contents)
        print(f"{face} SHA256: {actual}")
    values = {"OPENCUT_TEST_FONT_PATH": str(fonts / "DejaVuSans.ttf"), "OPENCUT_PLATFORM_EVIDENCE_DIR": str(repository / "target/platform-renderer-evidence"), "OPENCUT_TEST_PYTHON": sys.executable}
    for name, tool in (("OPENCUT_FFMPEG_PATH", "ffmpeg"), ("OPENCUT_FFPROBE_PATH", "ffprobe")):
        executable = shutil.which(tool)
        if not executable:
            raise RuntimeError(f"Required native tool unavailable: {tool}")
        subprocess.run([executable, "-version"], check=True)
        values[name] = executable
    Path(values["OPENCUT_PLATFORM_EVIDENCE_DIR"]).mkdir(parents=True, exist_ok=True)
    return values

def main():
    values = prepare(Path.cwd(), Path(os.environ["RUNNER_TEMP"]) / "opencut-platform-inputs")
    with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as environment:
        for name, value in values.items():
            if "\n" in value or "\r" in value:
                raise RuntimeError("Invalid environment path")
            environment.write(f"{name}={value}\n")
    print(f"Native Python: {sys.version}")

if __name__ == "__main__":
    main()
