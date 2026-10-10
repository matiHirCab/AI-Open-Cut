"""Select installed direct Windows release tools, preserving cache identity rules."""
import os
from pathlib import Path


def direct_cli(path):
    marker = b"%s version "
    with path.open("rb") as stream:
        if stream.read(2) != b"MZ":
            return False
        previous = b""
        while chunk := stream.read(65536):
            chunk = previous + chunk
            if marker in chunk:
                return True
            previous = chunk[-(len(marker) - 1):]
    return False


def backend_directory(installation):
    if not installation.is_absolute():
        raise RuntimeError("Required absolute Chocolatey installation")
    root = (installation / "lib/ffmpeg/tools").resolve(strict=True)
    if not root.is_relative_to(installation.resolve(strict=True)) or not root.is_dir():
        raise RuntimeError("Required installed FFmpeg tools directory")
    candidates = list(root.rglob("ffmpeg.exe"))
    if len(candidates) != 1:
        raise RuntimeError("Required unique direct installed FFmpeg pair")
    directory = candidates[0].parent
    for name in ("ffmpeg.exe", "ffprobe.exe"):
        path = (directory / name).resolve(strict=True)
        if not path.is_relative_to(root) or not path.is_file() or not direct_cli(path):
            raise RuntimeError(f"Required direct installed native backend: {name}")
    directory = directory.resolve(strict=True)
    if "\n" in str(directory) or "\r" in str(directory):
        raise RuntimeError("Unsafe release backend directory")
    return directory


def publish(installation, path_file):
    directory = backend_directory(installation)
    if not path_file.is_absolute() or not path_file.is_file():
        raise RuntimeError("Required existing absolute GitHub PATH file")
    with path_file.open("a", encoding="utf-8") as output:
        output.write(f"{directory}\n")
    print(f"Direct installed release backends: {directory}")
    return directory


if __name__ == "__main__":
    if os.name != "nt":
        raise RuntimeError("Direct Chocolatey release setup requires Windows")
    publish(Path(os.environ["ChocolateyInstall"]), Path(os.environ["GITHUB_PATH"]))
