"""Verify real source CLI/native artwork output with independent pixel oracles.

Requires Linux, Bun, a built headless binary, FFmpeg and FFprobe. Retains its
small synthetic project and JSON evidence in the reported temporary directory.
No provider, downloaded model, user media or external service is used.
"""

import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import time
import zlib


REPOSITORY = Path(__file__).resolve().parents[1]
BRIDGE = REPOSITORY / "apps" / "agent-bridge"


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def file_sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def executable(value):
    resolved = shutil.which(str(value))
    if resolved is None:
        raise RuntimeError(f"Required executable is unavailable: {value}")
    return str(Path(resolved).resolve())


def png_chunk(kind, data):
    return (
        struct.pack(">I", len(data))
        + kind
        + data
        + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    )


def synthetic_png():
    """Original 3x3 opaque white border with one red center; no production codec."""
    pixels = bytearray([255] * 3 * 3 * 4)
    pixels[16:20] = bytes([220, 20, 30, 255])
    rows = b"".join(b"\0" + pixels[y * 12 : (y + 1) * 12] for y in range(3))
    return (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", struct.pack(">IIBBBBB", 3, 3, 8, 6, 0, 0, 0))
        + png_chunk(b"IDAT", zlib.compress(rows))
        + png_chunk(b"IEND", b"")
    )


def verify(root, bun, headless, ffmpeg, ffprobe):
    exports = root / "exports"
    exports.mkdir()
    media = root / "media"
    media.mkdir()
    projects = root / "projects"
    environment = dict(
        os.environ,
        OPENCUT_PROJECTS_DIR=str(projects),
        OPENCUT_EXPORTS_DIR=str(exports),
        OPENCUT_ALLOWED_MEDIA_DIRS=str(media),
        OPENCUT_HEADLESS_PATH=headless,
        OPENCUT_FFMPEG_PATH=ffmpeg,
        OPENCUT_FFPROBE_PATH=ffprobe,
    )
    cli_command = [bun, "run", "src/artwork-cli.ts"]

    def invoke(command, request, success=True):
        process = subprocess.run(
            command,
            input=json.dumps(request).encode("utf-8"),
            capture_output=True,
            env=environment,
            cwd=BRIDGE,
            timeout=30,
            check=False,
        )
        stdout = [json.loads(line) for line in process.stdout.splitlines() if line.strip()]
        stderr = [json.loads(line) for line in process.stderr.splitlines() if line.strip()]
        require(
            (process.returncode == 0) == success,
            f"Unexpected command result: {process.returncode}, {stdout}, {stderr}",
        )
        return stdout, stderr

    def core(request):
        stdout, _ = invoke([headless], request)
        return stdout[-1]["result"]

    def cli(request, success=True):
        stdout, stderr = invoke(cli_command, request, success)
        return stdout[-1] if success else stderr[-1]

    def rgba(path):
        process = subprocess.run(
            [ffmpeg, "-v", "error", "-i", str(path), "-f", "rawvideo", "-pix_fmt", "rgba", "-"],
            capture_output=True,
            check=True,
            timeout=10,
        )
        return process.stdout

    source = synthetic_png()
    source_file = media / "synthetic-original.png"
    source_file.write_bytes(source)
    created = core({
        "operation": "create_project",
        "name": "Independent artwork acceptance",
        "settings": {"width": 64, "height": 64, "fps": 10},
    })
    project_id = created["projectId"]
    imported = core({
        "operation": "import_asset", "projectId": project_id,
        "expectedRevision": 0, "path": str(source_file), "mediaType": "image",
    })
    source_id = imported["changedIds"][0]
    project_directory = projects / project_id

    def authoritative_hashes():
        return {
            name: file_sha256(project_directory / name)
            for name in ("project.json", "history.json")
        }

    before = authoritative_hashes()
    review = exports / "review-one"
    parameters = {
        "cleanup": {"color": [255, 255, 255], "tolerance": 0},
        "outline": {"radius": 1, "color": [20, 30, 40, 255]},
    }
    preview = cli({
        "operation": "preview", "projectId": project_id, "expectedRevision": 1,
        "sourceAssetId": source_id, "parameters": parameters, "outputDirectory": str(review),
    })
    require(authoritative_hashes() == before, "Preview changed authoritative documents")
    require(
        sorted(path.name for path in review.iterdir())
        == ["dark.png", "light.png", "manifest.json", "mask.png", "result.png"],
        "Unexpected review package inventory",
    )
    manifest = json.loads((review / "manifest.json").read_text(encoding="utf-8"))
    require(manifest == preview["manifest"], "Published manifest differs from CLI response")
    require(manifest["sourceHash"] == sha256(source), "Source hash differs from original")
    for name in ("result", "mask", "light", "dark"):
        require(
            manifest["reviewHashes"][name] == file_sha256(review / f"{name}.png"),
            f"Incorrect {name} review hash",
        )

    # Independently decode actual production PNGs with FFmpeg. These literal
    # expectations neither call nor reproduce the production transform helper.
    result = rgba(review / "result.png")
    mask = rgba(review / "mask.png")
    expected = bytearray(5 * 5 * 4)
    expected_mask = bytearray([0, 0, 0, 255] * 25)
    for y in range(1, 4):
        for x in range(1, 4):
            expected[(y * 5 + x) * 4 : (y * 5 + x) * 4 + 4] = bytes([20, 30, 40, 255])
    expected[48:52] = bytes([220, 20, 30, 255])
    expected_mask[48:52] = bytes([255, 255, 255, 255])
    require(result == expected, "Result does not match independent exact RGBA oracle")
    require(mask == expected_mask, "Mask does not match independent exact alpha oracle")
    for name, background in (("light", 255), ("dark", 32)):
        plane = rgba(review / f"{name}.png")
        expected_plane = bytearray([background, background, background, 255] * 25)
        for y in range(1, 4):
            for x in range(1, 4):
                expected_plane[(y * 5 + x) * 4 : (y * 5 + x) * 4 + 4] = bytes([20, 30, 40, 255])
        expected_plane[48:52] = bytes([220, 20, 30, 255])
        require(plane == expected_plane, f"Incorrect independent {name} background oracle")

    applied = cli({"operation": "apply", "directory": str(review), "approve": True})
    require(applied["result"]["revision"] == 2, "Apply did not create exactly one revision")
    state = core({"operation": "get_state", "projectId": project_id})["project"]
    require(state["schemaVersion"] == 47, "Unexpected current schema")
    require(len(state["assets"]) == 2, "Apply did not create exactly one new asset")
    require(all(not track["items"] for track in state["tracks"]), "Apply changed timeline placement")
    asset = next(a for a in state["assets"] if a["id"] == applied["result"]["changedIds"][0])
    original = next(a for a in state["assets"] if a["id"] == source_id)
    require(
        (project_directory / asset["projectRelativePath"]).read_bytes() == (review / "result.png").read_bytes(),
        "Committed bytes differ from reviewed result",
    )
    require(asset["artworkDerivation"]["sourceAssetId"] == source_id, "Wrong source attribution")
    require((project_directory / original["projectRelativePath"]).read_bytes() == source, "Original source changed")
    stable = authoritative_hashes()
    refusal = cli({"operation": "apply", "directory": str(review), "approve": True}, False)
    require(refusal["error"]["code"] == "REVISION_CONFLICT", "Repeated apply was not rejected")
    require(authoritative_hashes() == stable, "Repeated apply changed authoritative documents")

    core({"operation": "undo", "projectId": project_id, "expectedRevision": 2})
    require(len(core({"operation": "get_state", "projectId": project_id})["project"]["assets"]) == 1, "Undo did not remove derivative")
    core({"operation": "redo", "projectId": project_id, "expectedRevision": 3})
    restored = core({"operation": "get_state", "projectId": project_id})["project"]
    require(len(restored["assets"]) == 2, "Redo did not restore derivative")
    require(restored["assets"][1]["artworkDerivation"] == asset["artworkDerivation"], "Redo changed provenance")
    reopened = core({"operation": "open_project", "projectId": project_id})["project"]
    require(reopened == restored, "Reopened project differs from restored project")
    stable = authoritative_hashes()
    discarded = cli({"operation": "discard", "directory": str(review)})
    require(discarded["discarded"] and not review.exists(), "Discard did not remove review package")
    require(authoritative_hashes() == stable, "Discard changed authoritative documents")
    require(file_sha256(project_directory / asset["projectRelativePath"]) == manifest["resultHash"], "Discard changed managed derivative")
    require((project_directory / original["projectRelativePath"]).read_bytes() == source, "Discard changed managed original")

    # Keep the child's stdin open. communicate() would close it and hide the
    # regression where abort emits JSON but leaves an active input handle alive.
    child = subprocess.Popen(
        cli_command, cwd=BRIDGE, env=environment,
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    try:
        time.sleep(2)
        require(child.poll() is None, "Idle CLI exited before cancellation")
        started = time.monotonic()
        child.send_signal(signal.SIGTERM)
        child.wait(timeout=3)
        elapsed = time.monotonic() - started
        errors = [json.loads(line) for line in child.stderr.read().splitlines() if line.strip()]
        require(child.returncode == 1, "Cancellation did not finish through the CLI error path")
        require(not child.stdout.read(), "Idle cancelled CLI emitted a successful result")
        require(len(errors) == 1 and errors[0]["error"]["code"] == "JOB_CANCELLED", "Idle cancellation lacks its typed failure")
        require(authoritative_hashes() == stable, "Idle cancellation changed the project")
    finally:
        if child.poll() is None:
            child.kill()
            child.wait(timeout=3)
        for stream in (child.stdin, child.stdout, child.stderr):
            stream.close()

    # Reject excessive input before EOF as well. A marker executable records
    # any accidental headless dispatch independently of project-state checks.
    dispatch_marker = root / "unexpected-headless-dispatch"
    forbidden_headless = root / "forbidden-headless"
    forbidden_headless.write_text(
        f"#!{sys.executable}\n"
        "from pathlib import Path\n"
        f"Path({str(dispatch_marker)!r}).write_text('unexpected dispatch')\n"
        "raise SystemExit(13)\n",
        encoding="utf-8",
    )
    forbidden_headless.chmod(0o700)
    contract = json.loads(
        (REPOSITORY / "contracts/artwork-preparation-v1.json").read_text(encoding="utf-8")
    )
    excessive_input = b"{" * (contract["limits"]["manifestBytes"] + 1)
    child = subprocess.Popen(
        cli_command, cwd=BRIDGE,
        env=dict(environment, OPENCUT_HEADLESS_PATH=str(forbidden_headless)),
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    try:
        child.stdin.write(excessive_input)
        child.stdin.flush()
        started = time.monotonic()
        # Do not close stdin: success must not depend on receiving EOF.
        child.wait(timeout=3)
        excessive_elapsed = time.monotonic() - started
        errors = [json.loads(line) for line in child.stderr.read().splitlines() if line.strip()]
        require(child.returncode == 1, "Excessive input did not exit through the CLI error path")
        require(not child.stdout.read(), "Excessive input emitted a successful result")
        require(len(errors) == 1 and errors[0]["error"]["code"] == "INVALID_ARGUMENT", "Excessive input lacks its typed failure")
        require(not dispatch_marker.exists(), "Excessive input dispatched a headless request")
        require(authoritative_hashes() == stable, "Excessive input changed the project")
    finally:
        if child.poll() is None:
            child.kill()
            child.wait(timeout=3)
        for stream in (child.stdin, child.stdout, child.stderr):
            stream.close()

    return {
        "passed": True,
        "source": "source CLI, actual native headless and independent FFmpeg decoding",
        "root": str(root), "headlessSha256": file_sha256(Path(headless)),
        "sourceSha256": sha256(source), "derivativeSha256": manifest["resultHash"],
        "projectId": project_id, "resultAssetId": asset["id"],
        "idleCancellationSeconds": elapsed,
        "excessiveInputExitSeconds": excessive_elapsed,
        "verified": [
            "preview preserves authoritative bytes", "exact independent RGBA/mask/light/dark oracles",
            "all review hashes", "one asset and revision commit", "source preserved",
            "repeated apply conflicts without changes", "undo/redo/reopen", "discard preserves managed media",
            "actual idle stdin SIGTERM emits JOB_CANCELLED and exits within three seconds",
            "excessive open stdin emits INVALID_ARGUMENT and exits before EOF without dispatch",
        ],
        "limits": ["Linux only", "no GUI evidence", "no cross-platform readiness claim"],
    }


def main():
    if sys.platform != "linux":
        print(json.dumps({"passed": False, "code": "DEPENDENCY_UNAVAILABLE", "reason": "Artwork file-review acceptance requires Linux"}))
        return 2
    bun = executable(os.environ.get("OPENCUT_BUN_PATH", os.environ.get("BUN", "bun")))
    headless = executable(os.environ.get("OPENCUT_TEST_HEADLESS_PATH") or os.environ.get("OPENCUT_HEADLESS_PATH") or REPOSITORY / "target/debug/opencut-headless")
    ffmpeg = executable(os.environ.get("OPENCUT_FFMPEG_PATH", "ffmpeg"))
    ffprobe = executable(os.environ.get("OPENCUT_FFPROBE_PATH", "ffprobe"))
    root = Path(tempfile.mkdtemp(prefix="opencut-artwork-acceptance-"))
    try:
        evidence = verify(root, bun, headless, ffmpeg, ffprobe)
    except Exception:
        print(json.dumps({"passed": False, "root": str(root)}), file=sys.stderr)
        raise
    (root / "evidence.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(evidence, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
