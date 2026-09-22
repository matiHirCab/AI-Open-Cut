## Context

The archived issue-36 change stores immutable rasters in a renderer-owned Arc cache. Headless currently constructs services once, reads stdin to EOF, dispatches one request and exits; every bridge call spawns that process. The living agent-bridge specification explicitly requires that lifecycle. This follow-up changes it only for render operations and strengthens the insufficient C2/V1 evidence identified by review.

The user selected one persistent warm worker plus one-shot overflow. Existing one-shot operations, optimistic revisions, atomic edits, job semantics and publication behavior remain compatibility surfaces. No implementation change is authorized until these artifacts receive approval.

## Goals / Non-Goals

**Goals:** Reuse rasters between successive agent frame/range/draft/export calls, preserve overlapping-request concurrency and cancellation isolation, and demonstrate valid-input invalidation and warm-preflight behavior end to end.

**Non-Goals:** Disk or global caches, a persistent worker pool, a render queue, automatic replay, renderer semantic changes, mutations through the worker, public cache statistics, and project-schema changes.

## Decisions

### 1. One reusable worker with one-shot overflow

Each HeadlessClient lazily reserves and starts at most one `--render-worker` process. Reservation occurs synchronously before asynchronous startup. A render arriving while the worker is starting, busy, or being terminated uses the existing one-shot path immediately. Non-render requests always use that path. No render queue or pool is introduced. A successful or normally failed request returns a healthy worker to idle; idle lifetime ends at client shutdown. A worker lost to transport failure is replaced only by a subsequent request. Different projects can share the worker because cache keys already include project identity and revision; no project snapshot survives between calls.

This preserves existing concurrency with one additional retained cache, unlike serial queueing, and bounds persistent memory unlike an expanding pool. Overflow requests need not benefit from the warm cache. Global/disk sharing would add unnecessary security and persistence concerns.

### 2. Explicit additive transport

The legacy CLI and EOF-delimited request behavior remain available unchanged. Worker mode emits exactly `{ "type": "ready", "protocolVersion": 1 }` on startup. It accepts closed JSON-line envelopes `{ "requestId": "...", "request": <existing request> }` and emits `{ "requestId": "...", "event": <existing progress/result/error event> }`. Only the four render operations are accepted. Reuse the existing Rust request type and event semantics; transport routing rejects mutations before dispatch. Worker protocol version is independent of public headless protocol version 1 and requires no new MCP tool or cache capability.

Use UTF-8 line framing with an inclusive 16777216-byte limit excluding the newline, enforced before unbounded accumulation, in both directions. Only one request is outstanding; terminal events release the request after cleanup, not process exit. Unknown envelope fields, malformed framing, incorrect IDs, oversized lines, unexpected readiness or duplicate terminal events make the process unusable. A syntactically valid correlated non-render request returns non-retryable INVALID_ARGUMENT without mutation. Other uncorrelatable malformed input causes worker exit. Sender-side framing failures must not dispatch a partial request.

Startup must complete within min(5000 ms, remaining request deadline). Missing executable or failed/unsupported startup uses DEPENDENCY_UNAVAILABLE; expiration of the request's own deadline uses retryable HEADLESS_TIMEOUT. Malformed output after readiness or unexpected process exit uses INTERNAL_ERROR. No failed request is replayed, even if the bridge cannot tell whether publication occurred. Updated bridge and headless ship together; older standalone clients continue using the compatible one-shot mode, but new bridge/old worker pairing is not silently downgraded.

Add `contracts/render-worker-v1.json` with canonical readiness, requests, events, rejection examples and operation coverage; register it in contract ownership with Rust and TypeScript consumers and parity tests. Do not add worker envelopes to the existing one-shot request union or weaken closed schemas. Obtain designated-owner review per ADR 0002.

### 3. Request identity without environment mutation

Add `Renderer::with_request_id(self, request_id: &str) -> Result<Self, CoreError>` as an additive Rust facade method. The artifact owner validates the existing nonempty ASCII alphanumeric/hyphen identity rule and overrides only ArtifactIo.request_id through a delegating adapter; other injected I/O behavior is preserved. The renderer clone retains shared cache/process ownership. Invalid identity yields INVALID_ARGUMENT. The original renderer and one-shot environment fallback remain unchanged.

The worker clones its renderer and scopes each request before render work. Do not set OPENCUT_REQUEST_ID dynamically: mutating global environment is unsafe and would couple concurrent work. Bridge cleanup derives only that request's existing owned paths; include draft preview paths, which currently are absent from its preview cleanup branch. No domain or path-safety decision moves to the bridge.

### 4. Shared dispatch and lifetime

Extract reusable render dispatch with an event sink so one-shot and worker modes use the same revision checks, fresh project/draft reads, path policy, renderer calls and export-relative-path mapping. Core services and renderer remain alive, but each request obtains a fresh immutable snapshot. Canonical evaluation, font-byte integrity, work limits, readiness, output collision checks and publication continue through existing core code.

Cancellation, deadline, malformed output and unexpected exit retire the worker. Terminate its process tree, wait for termination, then clean owned temporary files before releasing the slot. If termination cannot be confirmed, keep the slot retired and use one-shot overflow rather than claiming cleanup succeeded. Cleanup never removes final published files. Normal core errors allow reuse after their terminal event; subsequent calls repeat preflight. On Windows the native worker installs a private kill-on-close Job Object before readiness, inherited by renderer children, so abrupt worker death also kills descendants. The handle is held for the process lifetime and is not inherited; startup fails before readiness if containment cannot be established. This uses the already-locked windows-sys 0.61.2 OS bindings in headless, not a new editor-core private-owner edge. POSIX retirement kills the worker process group even after leader exit. Client close prevents new reservations, aborts active work and terminates idle workers. Other one-shot requests retain independent cancellation and existing job admission.

### 5. Conformance evidence

Use table-driven valid same-revision project variants, re-evaluation and materialization rather than raw hash mutations as the main C2 evidence. Compare changed rasters with a separately constructed renderer, assert actual misses and subsequent hits, and keep an unrelated retained key warm. Cover text content, runs/spans, paint ordering, layout, valid alternate fonts, vector paints/strokes/geometry, ordered SVG/viewport, output dimensions and composed sampling. Unsupported profiles are negative validation cases; implementation-version partitioning remains a focused key test. Composition-only changes must retain reuse and receive different correct placement/timing plans where appropriate.

Warm V1 tests compare cold/warm error code, retryability and stage, both hit and miss counters, process/artifact events and unchanged authoritative state. Exercise invalid numbers, missing references, missing/corrupt fonts, lexical/canonical path escapes, unsupported SVG, aggregate work limits and failed readiness. Use frame/range/draft/export where applicable, including invalid input against an existing export destination. Preserve normal warm-write failures and cleanup tests.

Add a Cargo feature `raster-cache-test-hooks`, disabled by default, forwarding from headless to core. It exposes counters only to the explicitly instrumented test build. Test-only worker diagnostics go to stderr and never alter canonical production stdout or MCP responses. Native bridge/worker tests prove same-process successive requests avoid real raster calls; process identity alone is insufficient. Build and test default binaries separately to prove the feature is not shipped by default.

Compare cold/warm/fresh results for all render intents with exact local bytes and semantic plans after intent normalization, recomputed warnings/layout diagnostics, independent pixel expectations, SSIM >= 0.99, aligned float-PCM RMS <= 0.0001 and timing within one output frame. No golden recapture or threshold changes are permitted.

## Risks / Trade-offs

- Worker retention increases idle memory: keep one cache bounded to 128 entries/67108864 key-plus-payload bytes; document that fonts, transient surfaces, simultaneous misses and overflow processes are not included in that limit.
- Cancellation can leave FFmpeg descendants or temporary files: test process-tree termination and cleanup ordering on supported platforms, including draft preview and export; do not release the worker while termination is uncertain.
- Late events can contaminate later calls: correlate every event, reject duplicate terminals, and discard a protocol-failed process instead of reusing it.
- Publication can precede connection failure: no automatic replay or deletion of final outputs.
- Test instrumentation can accidentally become a public surface: gate it explicitly, keep diagnostics out of normal wire output, and run default-build contract/smoke tests.
- Prior evidence used a local Rust toolchain different from the pin: establish the pinned Rust 1.97.0 toolchain before final checks or record the unavailable check as blocking; do not silently edit the pin.

## Migration Plan

No project or history migration is required. Ship paired bridge/headless binaries with the additive worker mode and preserve legacy one-shot clients. Reverting the bridge routing restores prior one-shot behavior without converting state; retained cache bytes disappear when workers exit. Keep the previous archive unchanged and synchronize only the approved follow-up deltas after verification.

## Open Questions

No product decision remains open. Explicit approval of these artifacts and designated contract-owner review are required gates, not inferred from planning choices.
