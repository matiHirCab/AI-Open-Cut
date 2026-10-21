"""Verify real source CLI/native artwork output with independent pixel oracles.

Requires Linux, Bun and a built headless binary. Default acceptance also requires
genuine FFmpeg/FFprobe. Explicit --hermetic mode is for source smoke without those
tools; its probe and independent decoder are identified separately in evidence.
Retains its small synthetic project and JSON evidence in a temporary directory.
No provider, downloaded model, user media or external service is used.
"""

import argparse
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


def paeth(left, above, upper_left):
    estimate = left + above - upper_left
    distances = [abs(estimate - value) for value in (left, above, upper_left)]
    return (left, above, upper_left)[distances.index(min(distances))]


def decode_rgba_png(data):
    """Independent bounded decoder for the production encoder's RGBA8 PNGs."""
    require(data.startswith(b"\x89PNG\r\n\x1a\n"), "PNG signature missing")
    require(len(data) <= 8 * 1024 * 1024, "Review PNG exceeds encoded bound")
    position = 8
    dimensions = None
    compressed = bytearray()
    ended = False
    while position < len(data):
        require(position + 12 <= len(data), "Truncated PNG chunk header")
        length = struct.unpack_from(">I", data, position)[0]
        kind = data[position + 4 : position + 8]
        end = position + length + 12
        require(end <= len(data), "Truncated PNG chunk")
        payload = data[position + 8 : end - 4]
        expected_crc = struct.unpack_from(">I", data, end - 4)[0]
        require(zlib.crc32(kind + payload) & 0xFFFFFFFF == expected_crc, "PNG CRC mismatch")
        if kind == b"IHDR":
            require(position == 8 and dimensions is None and length == 13, "Invalid PNG header")
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", payload)
            require(1 <= width <= 544 and 1 <= height <= 544, "Unexpected PNG dimensions")
            require((depth, color, compression, filtering, interlace) == (8, 6, 0, 0, 0), "Expected noninterlaced RGBA8 PNG")
            dimensions = (width, height)
        elif kind == b"IDAT":
            require(dimensions is not None, "PNG image precedes header")
            compressed.extend(payload)
        elif kind == b"IEND":
            require(length == 0 and end == len(data), "Invalid PNG ending")
            ended = True
            break
        else:
            require(kind not in (b"acTL", b"fcTL", b"fdAT"), "Unexpected animated PNG")
            require(kind[0] & 32, "Unknown critical PNG chunk")
        position = end
    require(ended and dimensions is not None and compressed, "Incomplete PNG")
    width, height = dimensions
    stride = width * 4
    expected_size = (stride + 1) * height
    inflater = zlib.decompressobj()
    filtered = inflater.decompress(bytes(compressed), expected_size + 1)
    require(
        len(filtered) == expected_size and inflater.eof
        and not inflater.unused_data and not inflater.unconsumed_tail,
        "PNG decompressed size or stream ending is invalid",
    )
    previous = bytearray(stride)
    pixels = bytearray()
    for y in range(height):
        start = y * (stride + 1)
        filter_type = filtered[start]
        require(filter_type in range(5), "Unsupported PNG filter")
        row = bytearray(filtered[start + 1 : start + 1 + stride])
        for index in range(stride):
            left = row[index - 4] if index >= 4 else 0
            above = previous[index]
            upper_left = previous[index - 4] if index >= 4 else 0
            prediction = (
                0, left, above, (left + above) // 2, paeth(left, above, upper_left)
            )[filter_type]
            row[index] = (row[index] + prediction) & 255
        pixels.extend(row)
        previous = row
    return bytes(pixels)


def verify_independent_decoder():
    """Literal encoded rows exercise every PNG filter independently of core."""
    first = bytes([10, 20, 30, 40, 50, 60, 70, 80])
    second = bytes([20, 30, 40, 50, 60, 70, 80, 90])
    filtered_seconds = (
        second,
        bytes([20, 30, 40, 50, 40, 40, 40, 40]),
        bytes([10] * 8),
        bytes([15, 20, 25, 30, 25, 25, 25, 25]),
        bytes([10] * 8),
    )
    for filter_type, row in enumerate(filtered_seconds):
        encoded = (
            b"\x89PNG\r\n\x1a\n"
            + png_chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 6, 0, 0, 0))
            + png_chunk(b"IDAT", zlib.compress(b"\0" + first + bytes([filter_type]) + row))
            + png_chunk(b"IEND", b"")
        )
        require(decode_rgba_png(encoded) == first + second, f"Independent PNG filter{filter_type} control failed")
    require(paeth(100, 0, 0) == 100, "Paeth left control failed")
    require(paeth(0, 100, 0) == 100, "Paeth above control failed")
    require(paeth(10, 20, 15) == 15, "Paeth upper-left control failed")


def hermetic_probe(root):
    """Probe only the exact generated source; this is not media-tool readiness."""
    path = root / "synthetic-png-probe"
    source = root / "media" / "synthetic-original.png"
    arguments = [
        "-v", "error", "-show_entries",
        "format=duration,format_name:stream=codec_type,codec_name,width,height,channels,sample_rate",
        "-of", "json", str(source),
    ]
    metadata = {
        "format": {"format_name": "png_pipe"},
        "streams": [{"codec_type": "video", "codec_name": "png", "width": 3, "height": 3}],
    }
    path.write_text(
        f"#!{sys.executable}\n"
        "import hashlib,json,sys\nfrom pathlib import Path\n"
        f"if sys.argv[1:] != {arguments!r}: raise SystemExit(14)\n"
        f"if hashlib.sha256(Path({str(source)!r}).read_bytes()).hexdigest() != {sha256(synthetic_png())!r}: raise SystemExit(15)\n"
        f"print(json.dumps({metadata!r}))\n",
        encoding="utf-8",
    )
    path.chmod(0o700)
    return str(path)


def verify(root, bun, headless, ffmpeg, ffprobe, hermetic=False):
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
        if hermetic:
            return decode_rgba_png(path.read_bytes())
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

    # Independently decode actual production PNGs with the selected backend. These literal
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
    require(state["schemaVersion"] == 48, "Unexpected current schema")
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
        "source": (
            "source CLI, actual native headless, synthetic-only probe and independent stdlib PNG decoding"
            if hermetic else "source CLI, actual native headless and independent FFmpeg decoding"
        ),
        "verificationMode": "hermetic" if hermetic else "genuine-media-tools",
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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hermetic", action="store_true", help="Source-smoke mode: exact synthetic probe and independent stdlib PNG decoder; no media-tool readiness claim")
    arguments = parser.parse_args()
    if sys.platform != "linux":
        print(json.dumps({"passed": False, "code": "DEPENDENCY_UNAVAILABLE", "reason": "Artwork file-review acceptance requires Linux"}))
        return 2
    bun = executable(os.environ.get("OPENCUT_BUN_PATH", os.environ.get("BUN", "bun")))
    headless = executable(os.environ.get("OPENCUT_TEST_HEADLESS_PATH") or os.environ.get("OPENCUT_HEADLESS_PATH") or REPOSITORY / "target/debug/opencut-headless")
    root = Path(tempfile.mkdtemp(prefix="opencut-artwork-acceptance-"))
    if arguments.hermetic:
        verify_independent_decoder()
        ffmpeg = str(root / "unused-ffmpeg-not-a-renderer")
        ffprobe = hermetic_probe(root)
    else:
        ffmpeg = executable(os.environ.get("OPENCUT_FFMPEG_PATH", "ffmpeg"))
        ffprobe = executable(os.environ.get("OPENCUT_FFPROBE_PATH", "ffprobe"))
    try:
        evidence = verify(root, bun, headless, ffmpeg, ffprobe, arguments.hermetic)
    except Exception:
        print(json.dumps({"passed": False, "root": str(root)}), file=sys.stderr)
        raise
    (root / "evidence.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(evidence, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
