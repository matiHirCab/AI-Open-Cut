# OpenCut Agent Bridge

## Architecture status and decision

This repository is the in-progress OpenCut rewrite. At the start of this work, the GPUI desktop contained layout placeholders, the web editor said “Coming soon,” and the API exposed only health and echo routes. There was no project model, timeline command layer, renderer, export pipeline, plugin API, headless API, or MCP server to reuse. The release changelogs describe the archived classic implementation and do not correspond to code in this branch.

The bridge therefore establishes the rewrite's first canonical editor domain in `crates/editor-core`. The Rust core owns project files, timeline validation, optimistic revisions, undo/redo, path policy, media ownership, and FFmpeg rendering. `apps/headless` exposes only typed JSON requests to that core. `apps/agent-bridge` adapts MCP tools to the headless process and intentionally contains no independent timeline model. Its MCP registration is split into injected projects, timeline, render, speech, and jobs capability modules that share only explicit session state and transport helpers.

Projects are directories below `OPENCUT_PROJECTS_DIR`. Each contains `project.json`, `history.json`, a lock file, hash-addressed assets, durable drafts, and generated previews. Project state, retained history, and committed-draft consumption use a recoverable write-ahead transaction so interruption cannot expose a mixed logical generation; see [Project persistence and recovery](project-persistence.md) for the commit point, warnings, recovery procedure, and backup/downgrade guidance. Schema version 5 adds locked/hidden/muted tracks, hidden items, and semantic caption tracks/items while preserving provider-neutral speech intent and wire compatibility. Legacy projects and retained history are migrated together under the project lock. Missing or changed media fails with `ASSET_INTEGRITY_FAILED` rather than silently accepting drift.

Physical media is stored extensionlessly at `assets/sha256/<prefix>/<digest>`. Imports and generated media with identical bytes share that file, while their logical asset IDs, display names, and provenance remain distinct. `asset_delete` removes only an unused logical asset, checks the expected revision, and participates in undo/redo. Lock-held mark-and-sweep treats current state and every retained undo/redo snapshot as roots. It deletes only unreferenced files below the managed asset directory; a deletion problem is returned as the non-fatal `ASSET_GC_FAILED` warning and retried on later opens or mutations.

## Requirements and configuration

Install the pinned tools from the repository root:

```powershell
proto use
```

Install FFmpeg and FFprobe and either put both on `PATH` or configure their executable paths. The installed FFmpeg build must provide `overlay`, `drawtext`, and `amix`, plus H.264 and AAC encoders for export.

Create local directories and set these variables. Multiple allowed media directories use the platform path separator (`;` on Windows and `:` on macOS/Linux).

`OPENCUT_CONFIG_ROOT` is resolved once to an absolute path at startup and defaults to the bridge working directory. Every documented relative project, media, export, generated-media, headless, font, Python, worker, model, and TTS-work path is resolved from that root. Bare command names such as `python`, `ffmpeg`, and `ffprobe` continue to use `PATH`; values containing path separators use the configuration root.

```powershell
$env:OPENCUT_CONFIG_ROOT = "C:\placeholder\AI-Open-Cut"
$env:OPENCUT_PROJECTS_DIR = "C:\placeholder\OpenCutData\projects"
$env:OPENCUT_ALLOWED_MEDIA_DIRS = "C:\placeholder\OpenCutMedia"
$env:OPENCUT_EXPORTS_DIR = "C:\placeholder\OpenCutExports"
$env:OPENCUT_FFMPEG_PATH = "ffmpeg"
$env:OPENCUT_FFPROBE_PATH = "ffprobe"
# Optional additional generated-media roots; OPENCUT_TTS_WORK_DIR is included automatically
$env:OPENCUT_GENERATED_MEDIA_DIRS = "C:\placeholder\GeneratedMedia"
# Optional: deterministic text rendering
$env:OPENCUT_DEFAULT_FONT_PATH = "C:\placeholder\Fonts\default.ttf"
$env:OPENCUT_LOG_LEVEL = "info" # error, warn, info, or debug
```

STDIO remains the default. Set `OPENCUT_TRANSPORT=http` for Streamable HTTP, which defaults to `127.0.0.1:3002`. Configure `OPENCUT_HTTP_HOST`, `OPENCUT_HTTP_PORT`, `OPENCUT_HTTP_AUTH_TOKEN`, a comma-separated `OPENCUT_HTTP_ALLOWED_ORIGINS` without wildcards, and `OPENCUT_HTTP_MAX_BODY_BYTES` as needed. Non-loopback startup is rejected without a bearer token. MCP is mounted at `/mcp`; `/health` exposes sanitized readiness only.

No directory is exposed unless explicitly configured. Imports reject URLs, `..` traversal, and canonical paths outside the media roots. Exports accept only paths relative to the export root and never overwrite without `overwrite: true`.

## Build, development, and readiness

```powershell
moon run editor-core:test
moon run headless:build
moon run agent-bridge:typecheck
moon run agent-bridge:test
moon run agent-bridge:test-integration
moon run agent-bridge:test-smoke
moon run agent-bridge:build
moon run agent-bridge:health
moon run agent-bridge:doctor
```

`moon run agent-bridge:dev` starts the STDIO bridge after building its headless dependency. `moon run agent-bridge:dev-with-desktop` additionally launches the current GPUI shell, with desktop output kept off MCP stdout. A client must own the STDIO process; do not start it in a public listener.

The production build assembles `opencut-agent-bridge`, `opencut-headless` (with `.exe` suffixes on Windows), `kokoro-tts/worker.py`, and `faster-whisper/worker.py` under `apps/agent-bridge/dist`. `manifest.json` records each relative path, byte size, and SHA-256; setup scripts, tests, Moon files, environments, and model weights are excluded. Health reports editor, rendering, speech, and transcription independently without paths or provider internals.

Run `opencut-agent-bridge --doctor` or `bun run doctor` for strict local diagnostics. It emits one structured JSON report covering the configured Python version, free disk, directory writeability, FFmpeg/FFprobe, readiness-marker/model state, provider queue state, and an actual short synthesis with verified cleanup. Less than 5 GiB free is a warning; missing required dependencies, failed writes, invalid model readiness, synthesis failure, or cleanup failure produce a nonzero exit.

Runtime logs are JSON Lines on stderr and are filtered by `OPENCUT_LOG_LEVEL` (`info` by default). Events include request/job/provider correlation IDs, queue wait, headless/provider/synthesis timings, character/chunk counts, and cleanup outcomes. Full speech text, pronunciation entries, artifact tokens, private paths, provider stderr, and internal provider details are never logged. MCP protocol output remains exclusively on stdout.

The bridge intentionally retains one typed STDIO headless process per request for fault and environment isolation. The repeatable release benchmark (`bun run bench:headless`) performs one warm-up and 30 status requests. On Windows 11 x86-64 on 2026-08-27 it measured a 42.84 ms median and 46.35 ms p95, below the 100 ms median and 250 ms p95 follow-up thresholds. Crossing either threshold should create a persistent-service/library-binding architecture follow-up; it does not silently change the transport.

`contracts/error-codes-v1.json` is the canonical public error catalog. It fixes each code's retryability, owning layer, and path-safe public description; Rust, TypeScript, and Python tests enforce parity. Unknown provider codes map to `TTS_PROVIDER_FAILED`, and provider stderr, paths, and internal messages are not returned to clients.

## Agent editing, drafts, and captions

Typed split, duplicate, batch-edit, track, and visibility tools use optimistic revisions. Batches validate on a clone and commit as one undoable revision. Locked tracks reject item mutations until explicitly unlocked; hidden items/tracks are omitted from rendering, and muted tracks contribute no audio. MCP resources expose project lists, revisioned state/timelines, jobs, and drafts without filesystem paths, while workflow prompts guide clients through reads, previews, polling, and explicit overwrite approval.

Durable drafts retain up to 100 typed operations under the project lock. They survive restarts, stay outside project history, do not increment revisions, and remain after conflicts until committed or explicitly discarded.

## Local faster-whisper transcription

Run `powershell -ExecutionPolicy Bypass -File .\apps\faster-whisper\setup.ps1` from the repository root. It creates an ignored Python 3.11 environment and prepares the default `small` model under `local-data/transcription`. Inference is local CPU `int8`, with VAD, word timestamps, optional supplied language, and offline execution after preparation. Override discovery using `OPENCUT_TRANSCRIPTION_PYTHON`, `OPENCUT_TRANSCRIPTION_WORKER`, `OPENCUT_TRANSCRIPTION_MODEL_DIR`, and `OPENCUT_TRANSCRIPTION_MODEL`.

`transcription_preview` is a cancellable job that returns segments and an expiring token without editing. `transcription_commit_preview` atomically creates caption items—and a caption track if needed—while preserving recognized text and provider/model/language provenance. Revision conflicts retain the token. `transcription_get_status`, `transcription_estimate`, and `transcription_discard_preview` cover readiness, zero-cost estimates, and cleanup.

The default `bun run test`/`moon run agent-bridge:test` suite is hermetic: it does not use `dist`, a release headless binary, FFmpeg, network access, Kokoro weights, or the real model environment. `test-integration` runs the source bridge against an explicitly built debug headless binary and the fake speech provider. `test-smoke` always builds release headless and a compiled bridge into a new temporary directory before exercising them, so stale or locked canonical `dist` artifacts cannot be selected. `test-tts-real` is opt-in and outside CI.

## Connect ChatGPT desktop

Current ChatGPT desktop and Codex share MCP configuration on the same Codex host.

1. Open **Settings**, then **MCP servers**.
2. Select **Add server**.
3. Name it `opencut`, choose **STDIO**, and select the absolute `opencut-agent-bridge.exe` produced by the build.
4. Add the three required directory variables and any FFmpeg/font overrides. Do not put secrets in them.
5. Save, select **Restart**, then type `/mcp` in the composer and call `editor_get_status`.

These steps follow the [official OpenAI MCP documentation](https://learn.chatgpt.com/docs/extend/mcp?surface=cli). ChatGPT connectivity must be tested from the desktop app; the automated smoke test verifies the same MCP STDIO protocol but not the ChatGPT UI.

### Canonical preview and export semantics

A ready rendering subsystem reports `evaluated_scene_rendering`. Frame preview, audiovisual range preview, draft preview, and final export then consume one editor-core evaluated scene: top-left pixel coordinates, integer-millisecond half-open intervals, bottom-to-top track/item array order, transforms, transitions, text, media, and resolved audio behavior are shared across every entry point. Output intent only selects seeking, duration, audio inclusion, and encoding.

Scene evaluation enforces finite values and explicit limits before workspace or process I/O. Media and fonts resolve only through project/font-root-constrained bindings; raw FFmpeg expressions, executable SVG, arbitrary paths, and network resources are not accepted. Renderer fallback is deterministic and local, and succeeds only for a backend that supports the complete scene; otherwise rendering returns the existing stable `DEPENDENCY_UNAVAILABLE` behavior and publishes no partial artifact. Equivalent semantic plans match exactly; fixed synthetic preview/export fixtures use SSIM `>= 0.99`, decoded float-PCM RMS error `<= 0.0001`, and timing within one output frame.

## Connect Codex

CLI example with placeholders:

```powershell
codex mcp add opencut `
  --env OPENCUT_PROJECTS_DIR=C:\placeholder\OpenCutData\projects `
  --env OPENCUT_ALLOWED_MEDIA_DIRS=C:\placeholder\OpenCutMedia `
  --env OPENCUT_EXPORTS_DIR=C:\placeholder\OpenCutExports `
  -- C:\placeholder\AI-Open-Cut\apps\agent-bridge\dist\opencut-agent-bridge.exe
```

Equivalent `~/.codex/config.toml` configuration:

```toml
[mcp_servers.opencut]
command = "C:\\placeholder\\AI-Open-Cut\\apps\\agent-bridge\\dist\\opencut-agent-bridge.exe"
cwd = "C:\\placeholder\\AI-Open-Cut"
default_tools_approval_mode = "writes"
startup_timeout_sec = 20
tool_timeout_sec = 600

[mcp_servers.opencut.env]
OPENCUT_PROJECTS_DIR = "C:\\placeholder\\OpenCutData\\projects"
OPENCUT_ALLOWED_MEDIA_DIRS = "C:\\placeholder\\OpenCutMedia"
OPENCUT_EXPORTS_DIR = "C:\\placeholder\\OpenCutExports"
```

## Local Kokoro text-to-speech

The bridge exposes provider-neutral local speech synthesis and currently ships a Kokoro-82M CPU adapter. It generates a 24 kHz mono WAV and atomically inserts it on an OpenCut audio track. V1 includes Kokoro's American and British English voices and defaults to `af_heart`. No API key or hosted inference service is used.

### Supported platforms

- Windows 11 x86-64 supports the Rust core, headless process, bridge, fake provider, and the verified real Kokoro CPU setup.
- The current Ubuntu and macOS CI runners support the Rust core, headless process, bridge, minimal Python worker tests, and fake-provider integration workflows.
- Real Kokoro installation and model execution on Linux, macOS, and WSL are not currently supported or verified. The runtime contains conventional POSIX virtualenv discovery paths for future support, but those paths are not a setup guarantee.

The small cross-platform worker-test environment is separate from the real model environment. Run `bun run apps/agent-bridge/scripts/run-python-tests.ts` from the repository root; it creates or reuses ignored `local-data/kokoro-test-venv`, installs only the pinned NumPy/SoundFile/CFFI/Pycparser lock, and never downloads Kokoro, Torch, or model weights.

From the repository root, create the isolated Python environment, install the CPU PyTorch build and Kokoro 0.9.4, download the model and English voices, and verify a short synthesis:

```powershell
powershell -ExecutionPolicy Bypass -File .\apps\kokoro-tts\setup.ps1
```

The first setup requires internet access and several gigabytes of free disk space for Python, CPU PyTorch, model weights, and language dependencies. By default everything is stored below `local-data/kokoro`, which is ignored by Git. The setup command prints the absolute configuration values to add to the MCP server:

```powershell
$env:OPENCUT_KOKORO_PYTHON = "C:\placeholder\AI-Open-Cut\local-data\kokoro\venv\Scripts\python.exe"
$env:OPENCUT_KOKORO_MODEL_DIR = "C:\placeholder\AI-Open-Cut\local-data\kokoro\model"
$env:OPENCUT_TTS_WORK_DIR = "C:\placeholder\AI-Open-Cut\local-data\kokoro\work"
$env:OPENCUT_SPEECH_PROVIDER = "kokoro"
```

When the MCP server's working directory is the repository root, the bridge automatically discovers this default `local-data/kokoro` installation. Set the variables explicitly when using another data location or working directory.

After setup, the bridge forces Hugging Face offline mode and CPU execution for the worker. `speech_list_voices` is the discoverable voice catalog and includes friendly labels, locale/accent, provider/model IDs, preview support, default status, and current availability. `speech_estimate` accepts either new text intent or an existing speech item with overrides and reports approximate minimum/expected/maximum duration, resolved chunks/model/voice, zero local cost, cache/load and FIFO queue state, and conservative Kokoro requirements of 2/4 logical CPUs and 2/4 GiB RAM minimum/recommended. Estimates are approximate and are not billing data.

Use queued `speech_preview` to synthesize playable audio without editing a project; completed `job_get_status` includes an MCP audio content block and an opaque expiring token. `speech_commit_preview` inserts at a track/start or replaces an existing item without rerunning inference, while `speech_discard_preview` cleans it immediately. A revision conflict retains the token for retry. `speech_regenerate` loads persisted speech intent, applies optional text/language/voice/speed/control overrides, and atomically replaces the existing item's asset while preserving its item ID, placement, transforms, keyframes, and audio controls. Existing `tts_get_status`, `tts_generate_and_insert`, and `tts_commit_generated_artifact` remain supported compatibility aliases/workflows.

Basic text normalization applies Unicode NFC and whitespace normalization. Pronunciations are bounded ordered literal `{term, spoken}` replacements. Sentence chunking is the default and inserts the configured silence between chunks; `chunking: none` disables it. The request limit remains 5,000 characters, and every request produces one physical WAV, one logical asset, and one timeline item.

Call `tts_get_status` to inspect the configured provider, models, languages, device, limits, defaults, readiness, resources, and FIFO queue depth. Poll returned process-local jobs with `job_get_status`; jobs do not survive a bridge restart, expire one hour after reaching a terminal state by default, and can be cancelled with `job_cancel` until an atomic project commit begins.

Speech generation is serialized with one active request and eight queued requests by default. Queue overflow returns retryable `TTS_QUEUE_FULL`. If the project changes after synthesis, the failed job returns `REVISION_CONFLICT` plus a short-lived generated-artifact token. Re-read the project and call `tts_commit_generated_artifact` with the refreshed revision before the token expires; this reuses the completed WAV without rerunning inference.

Committed speech receives a domain-generated portable name such as `Heart - Welcome to the project.wav`. Provider prefixes such as `af_` are removed, whitespace is normalized, the excerpt is capped at 48 Unicode characters, and control/path/reserved filename characters and names are sanitized consistently across Windows, macOS, and Linux.

Successful TTS results include a `warnings` array. If the project commit succeeds but temporary-file deletion does not, the job remains completed and reports `TEMP_FILE_CLEANUP_FAILED`; the bridge retains ownership and retries cleanup during shutdown.

Headless, TTS control, and TTS synthesis requests have configurable deadlines through `OPENCUT_HEADLESS_REQUEST_TIMEOUT_MS`, `OPENCUT_TTS_CONTROL_TIMEOUT_MS`, and `OPENCUT_TTS_SYNTHESIS_TIMEOUT_MS`. Job retention/capacity and speech queue/artifact retention use `OPENCUT_JOB_TTL_MS`, `OPENCUT_JOB_MAX_COUNT`, `OPENCUT_TTS_MAX_QUEUED`, and `OPENCUT_GENERATED_ARTIFACT_TTL_MS`. Timeouts and shutdown remove only temporary files owned by the current bridge process.

If status reports that TTS is unavailable, rerun the setup command and confirm that all three configured paths are absolute. Delete only `local-data/kokoro/work` to clear abandoned temporary output. Delete the whole `local-data/kokoro` directory to remove the local environment and downloaded model; setup can recreate it.

Run `codex mcp list`, then use `/mcp` and `editor_get_status`. Agents should read state first, make one edit at a time using the returned revision, re-read after conflicts, preview before export, poll jobs to completion, and never authorize overwrite implicitly.

## MVP limitations

- The GPUI panels remain placeholders and do not yet visually edit these project files.
- Jobs and their progress are bounded, expire automatically, and do not survive restart.
- TTS jobs are serialized per bridge process and use CPU inference only in V1.
- HTTP is for localhost or authenticated trusted-LAN use, not public internet deployment. There is no remote URL import, scripting tool, OpenAI API, or generative video dependency.
- Export is MP4/H.264/AAC at project, 1080p, or 720p resolution. The renderer supports the MVP transform, fade/crossfade, text, and audio controls; it is not yet a full nonlinear compositor.
- The recommended next milestone is GPUI integration with the shared core and durable render-job recovery.
