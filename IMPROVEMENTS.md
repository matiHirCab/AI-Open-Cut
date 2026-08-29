# OpenCut improvement checklist

Reviewed 2026-08-26. The first milestone should make speech synthesis a provider-neutral application capability whose semantics are represented by the canonical editor model. The editor model should own speech intent, validation, and persisted provenance; an application service should own orchestration; provider adapters such as Kokoro should own inference details; MCP should only translate transport requests and responses.

## P0 — TTS model and boundary

- [x] Add provider-neutral model types in `crates/editor-core`, such as `SpeechSynthesisRequest`, `SpeechVoiceId`, `SpeechGeneration`, and `GeneratedAssetOrigin`.
  - Keep stable fields in the model: text, language, voice ID, speed, provider ID, model ID/version, sample rate, and generation timestamp.
  - Do not encode Kokoro's current voice list or CPU-only implementation as canonical editor enums.
  - Add schema-version migration/default behavior before persisting new fields in `project.json`.
- [x] Persist generation provenance on the resulting asset instead of reducing generated speech to an indistinguishable imported WAV. This enables inspect, regenerate, change-voice, and audit workflows.
- [x] Introduce a `SpeechSynthesizer`/`GeneratedMediaProvider` interface with `status`, `list_voices`, `synthesize`, `cancel`, and `close` behavior. Put Kokoro behind an adapter implementing that contract.
- [x] Move the generate → validate → import → insert workflow out of `apps/agent-bridge/src/server.ts` into a provider-neutral application service or headless command. The MCP handler should parse input, invoke one use case, and map the result.
- [x] Replace the TTS-specific `CommitGeneratedAudio`/`import_generated_audio_and_add` API with a typed `CommitGeneratedAsset` command carrying a request struct. Keep atomic asset-plus-timeline insertion in the Rust core.
- [x] Make placement validation canonical in the core/application service. Remove the duplicate audio-track and revision pre-checks from the MCP handler once the single command reports typed domain errors.
- [x] Return a typed result containing named `asset_id` and `item_id` fields. Do not depend on `changedIds[0]` and `changedIds[1]` ordering in `server.ts`.
- [x] Expose provider capabilities dynamically (`providerId`, models, languages, voices, devices, limits) rather than hard-coding Kokoro voices in both `schemas.ts` and `worker.py`.
- [x] Choose one source of truth for cross-language contracts and generate or contract-test the Rust, TypeScript, and Python representations. At minimum, add a test that detects voice/model/sample-rate drift.

### P0 acceptance criteria

- [x] A fake synthesizer and Kokoro can be swapped without changing MCP tool registration or editor-core business rules.
- [x] A project reopened from disk retains enough speech metadata to explain and regenerate the asset.
- [x] Adding a second provider requires an adapter and configuration, not edits to the core project schema or MCP orchestration.
- [x] Unit tests cover model serialization/migration, speech validation, provider failure, revision conflict, atomic commit, and provider substitution.

## P1 — Reliability and job lifecycle

- [x] Add configurable deadlines to headless requests and TTS worker requests. On timeout, terminate the child, reject pending work with a stable retryable error, and clean temporary files.
- [x] Thread cancellation from the job registry into the provider process and add a `job_cancel` operation. Define cancellation behavior during synthesis and during commit.
- [x] Bound the in-memory job registry with TTL/max-count cleanup. Prefer durable jobs if bridge restarts are expected; otherwise document restart semantics in the API response as well as the guide.
- [x] Return a stable `JOB_NOT_FOUND` bridge error from `JobRegistry.get`; the current generic `Error("JOB_NOT_FOUND")` becomes `INTERNAL_ERROR` at the MCP boundary.
- [x] Clamp and validate provider progress before storing it, and make progress monotonic.
- [x] Define shutdown handling that closes the TTS worker, rejects queued requests, and removes only files owned by the current bridge process.
- [x] Avoid throwing away an expensive completed synthesis on `REVISION_CONFLICT`. Consider a short-lived generated-artifact token that can be committed against a refreshed revision without rerunning inference.
- [x] Make queue policy explicit per provider (concurrency, maximum queued requests, overload error, and fairness). A single promise chain currently permits an unbounded queue.
- [x] Validate the Kokoro readiness marker contents against provider ID, model/version, voices, and dependency versions. A marker file's existence alone can report a stale or incomplete installation as ready.
- [x] Either honor `OPENCUT_KOKORO_DEVICE` end to end or remove it. It is currently documented and emitted by setup, while TypeScript and Python force CPU and the response schema only accepts `cpu`.
- [x] Stop mutating global `process.env` in `TtsWorker` construction. Resolve immutable typed configuration once at startup and inject it into the provider, headless client, and health reporting.

## P1 — Correctness, contracts, and tests

- [x] Replace the eight-argument `import_generated_audio_and_add` method with a request object. This clarifies the contract and fixes the current strict-Clippy failure (`clippy::too_many_arguments`).
- [x] Make `bun run test` hermetic: build the bridge and release headless binary first, or have smoke tests launch source code with an explicitly built sidecar. It currently reads `dist`/`target/release`, so it can fail or exercise stale binaries.
- [x] Split unit, integration, smoke, and real-model tests into explicit commands. Keep the default unit suite independent of FFmpeg, a release build, network access, and installed Kokoro weights.
- [x] Run Python worker tests in the managed Kokoro environment, or provide a small test dependency environment. `python -m unittest test_worker.py` currently relies on ambient `soundfile` and fails when it is absent.
- [x] Add an MCP integration test for the full fake-TTS path that asserts persisted provenance, typed IDs, cleanup, and undo/redo—not only WAV insertion.
- [x] Add failure-path tests for worker startup error, malformed JSON, partial stdout lines, process exit, request timeout, cancellation, commit conflict, cleanup failure, and queue continuation after a failed job.
- [x] Add headless request/response contract tests. `apps/headless` currently has no direct tests even though it is the trust boundary between TypeScript and Rust.
- [x] Add CI gates for `cargo fmt --check --all`, strict Clippy, Rust tests, TypeScript typecheck/lint/unit tests, Python unit tests, and one packaged fake-provider smoke test.
- [x] Test and document supported platforms. `setup.ps1`, the verified lock comment, and virtualenv path are Windows-specific, while runtime code contains macOS/Linux paths.

## P2 — Architecture and maintainability

- [x] Break up `apps/agent-bridge/src/server.ts` by capability (`projects`, `timeline`, `render`, `speech`, `jobs`) and inject use-case services into registration functions.
- [x] Replace `Record<string, unknown>` headless requests with a discriminated TypeScript request union matching the Rust `Request` enum.
- [x] Centralize structured error codes across Rust, TypeScript, and Python. Document retryability and map provider errors to stable application errors without leaking provider internals or paths.
- [x] Separate editor readiness from optional subsystem readiness. Return structured subsystem health so missing FFmpeg and unavailable speech are independently diagnosable.
- [x] Generalize `generated_media_root` and speech-specific error strings in `PathPolicy` so image, music, captions, and future generated media use the same safe ingestion path.
- [x] Store content hashes and media probe facts on assets. Use hashes for deduplication/integrity checks and make cached/generated artifacts addressable.
- [x] Define ownership for generated and copied files during undo/delete. Project history can remove model references while asset files remain; add garbage collection that is lock-safe and history-aware.
- [x] Consider a long-lived headless service or library binding if process-per-request startup becomes material. Measure first and keep STDIO isolation if its simplicity wins.
- [x] Generate human-readable display names independently from provider IDs (for example, voice label and text excerpt) and sanitize them in the domain layer.

## P2 — Product and developer experience

- [x] Add `speech_list_voices` with labels, locale/accent, provider, model, preview support, and availability so clients do not need baked-in IDs.
- [x] Support preview-before-insert and regenerate/change-voice flows using persisted speech intent.
- [x] Surface estimated duration/cost/resource use before queueing. For local Kokoro, report model load state, queue depth, and approximate CPU/RAM requirements.
- [x] Add text normalization and pronunciation controls (language, phonemes/lexicon, pauses, sentence chunking) behind provider-neutral options.
- [x] Decide how long text maps to timeline content: one asset, sentence-level clips, or a grouped speech object. Make this explicit before expanding beyond the current 5,000-character limit.
- [x] Add structured logs to stderr with request/job/provider IDs, timings, queue wait, synthesis duration, and cleanup outcome; never log full user text by default.
- [x] Add a cross-platform setup/status command that checks Python version, disk space, FFmpeg, model cache integrity, write permissions, and an actual short synthesis.
- [x] Package only runtime sidecar files. The current recursive copy includes setup and dependency-installation material; use an explicit allowlist and produce a manifest/checksum.
- [x] Keep `.env.example` consistent with runtime validation: either accept documented relative paths by resolving them from a defined root, or show absolute placeholders everywhere.

## P3 — Existing repository quality

- [x] Resolve or intentionally gate the desktop dead-code/unused-import warnings so workspace diagnostics remain signal-rich.
- [x] Add an `AGENTS.md` or contributor architecture note defining ownership boundaries: model/domain, application services, providers, transports, and UI.
- [x] Replace the broad `docs/*` ignore rule with explicit generated-doc exclusions so new architecture and operational documentation is not silently ignored.
- [x] Record an ADR for the TTS abstraction, provider contract, persisted provenance, job durability choice, and schema migration policy.

## Verification snapshot

- [x] `cargo test --workspace`: passed (33 tests total; no desktop warning noise).
- [x] `cargo fmt --check --all`: passed.
- [x] `bun run typecheck`: passed.
- [x] `bun run lint`: passed (34 files checked).
- [x] `cargo clippy --workspace --all-targets -- -D warnings`: passed; the documented dormant-component module is the only warning exemption.
- [x] `bun run test`: passed (54 unit/subprocess-contract tests) with nonexistent headless/FFmpeg paths and without using packaged artifacts.
- [x] `bun run test:integration`: passed (4 fake-provider MCP workflows).
- [x] `bun run test:smoke`: passed (isolated release headless and compiled bridge).
- [x] `bun run apps/agent-bridge/scripts/run-python-tests.ts`: passed (10 tests in the pinned minimal environment).
- [x] `bun run bench:headless`: passed (30 release status samples; 41.16 ms median, 43.17 ms p95; process-per-request STDIO retained).
