# Renderer availability and portable packaging

Issue #78 verifies the existing APIs through source and the default assembled runtime on Linux, Windows and macOS. It adds package integrity checks and native execution evidence; project schema44, headless protocol1, MCP/provider contracts and rendering semantics retain their existing versions.

Call `editor_get_status` and inspect `subsystems.rendering.ready`, its error and advertised capabilities. EditorCore owns FFmpeg filter and FFprobe usability checks. A readable executable path in `paths.ffmpeg` or `--doctor` diagnostics only establishes path access. An installed build can lack required filters: the native inventory fixture proves a readable executable can coexist with unavailable rendering. Do not infer filter support from a filename or invent replacement processing.

Base readiness requires overlay, drawtext, amix, geq, remap, blend, nullsrc, split, pad, crop and format plus an executable FFprobe. Optional audio DSP, ducking, analysis and normalization retain separate canonical checks and their existing capability identifiers. If their requirements are unavailable, clients must refuse a requested unsupported operation rather than silently remove it. The native test authors master DSP with an incomplete inventory and requires the render job to fail with `DEPENDENCY_UNAVAILABLE`, without an artifact or authoritative file/revision changes.

Missing FFmpeg, FFprobe or required base filters leaves the editor usable. Create/edit, undo/redo and fresh reopen remain available; renders fail with their existing stable errors. Speech/transcription readiness is separate: packaged worker files alone do not install Python packages, models or prove inference quality. No model inference is needed for the local deterministic imported-PCM renderer checks.

## Runtime inventory

`assembleRuntimePackage` retains manifest version1: two root binaries using the platform's normal executable suffix, `kokoro-tts/worker.py` and `faster-whisper/worker.py`. `manifest.json` records each regular file's SHA256 and byte count. The verifier rejects unknown manifest/entry keys, future versions, invalid hashes/counts, duplicate/case/normalization collisions, ambiguous portable paths, links/junctions, missing/non-regular files and undeclared files or directories. POSIX root binaries must have executable permission. Portable names include [Windows device-name restrictions](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file), including reserved superscript COM/LPT suffixes. Manifest reading is limited to16KiB; binary hashing streams64KiB chunks. Name/source preflight fails before replacement of an existing destination. Verification assumes a static package owned by its caller; it does not introduce a concurrent filesystem publication transaction.

Keep projects, exports, logs, model caches and generated media outside this sealed runtime. The native driver verifies inventory before and after execution and uses the repository's default release headless binary and compiled bridge, with both original workers. Issue #77's private instrumented cache binary remains separate evidence.

## Reproduce native verification

Use the repository toolchain pins in `.prototools` (Rust1.97.0, Bun1.4.0, Moon2.3.3) and Python3.11 in CI. Supply actual FFmpeg/FFprobe and the four fixed bundled DejaVu faces. Existing font provenance, redistribution license and contract hashes live in `crates/editor-core/resources/fonts` and `contracts/text-layout-v2.json`; setup verifies these bytes without a download or rebaseline. The setup has hermetic tamper/missing/unusable dependency negative tests.

The independent workflow `.github/workflows/motion-platform-renderer.yml` installs Linux FFmpeg, [Homebrew ffmpeg@7](https://formulae.brew.sh/formula/ffmpeg@7) with explicit keg PATH on macOS, and [Chocolatey FFmpeg7.1.1](https://community.chocolatey.org/packages/ffmpeg/7.1.1) on Windows. Actual version/status/media checks determine usability; package metadata alone is insufficient. Existing required workflows, deadlines and foundation gates remain unchanged.

```sh
# Set all paths to real local dependencies and an owned evidence destination.
OPENCUT_FFMPEG_PATH=/usr/bin/ffmpeg \
OPENCUT_FFPROBE_PATH=/usr/bin/ffprobe \
OPENCUT_TEST_FONT_PATH="$PWD/crates/editor-core/resources/fonts/DejaVuSans.ttf" \
OPENCUT_TEST_PYTHON=python \
OPENCUT_PLATFORM_EVIDENCE_DIR="$PWD/target/platform-renderer-evidence" \
bun run apps/agent-bridge/scripts/run-platform-renderer.ts
```

The driver enables mandatory native execution and validates the actual JSON execution report for exactly both passing cases and zero skips; missing inputs/dependencies and failing jobs are errors. The two source/default-package cases produce actual192x10810fps one-second PNG/H264+stereo48kAAC media, require matched SSIM>=0.99, floatPCM RMS<=0.0001 and one-frame duration tolerance, and verify independent solid-color and nonzero audio witnesses. Invalid input, missing asset and stale revision retain authoritative bytes; history and reopen compare authored state and frame identity. Four missing/partial dependency scenarios preserve editing/history/reopen and fail rendering closed. Runtime manifests, actual tool versions, both native reports and original frame/export files are uploaded per platform. These checks establish media correctness and fallback; they make no listening, model-quality or desktop-GUI claim.

Exact published-head Windows, macOS and Linux results plus all prior required checks must reach terminal success before issue acceptance. Local Linux evidence alone is insufficient.
