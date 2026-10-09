## Context

The bridge owns process-local jobs and disposable request outputs. Editor-core owns authoritative validation, revisions, project media and rendering. Its immutable encoded preview cache already has an independent32-entry/64MiB budget. Bridge artifact descriptors are currently retained only by job TTL/count; backing previews leak and cancellation makes unsettled producers eligible for eviction.

## Goals / Non-Goals

Fulfil #73 without public/schema/renderer changes. Do not garbage-collect project media, delete exports, persist retention state, clean predecessor-process files or replace required native scene fixtures.

## Decisions

1. Track task settlement separately from visible terminal status. Admission and expiration cannot reclaim an unsettled task; close awaits every producer. Cancellation stays immediate/idempotent and progress stays monotonic. This avoids a new public cancelling state.
2. Serialize preview retention/publication/disposal around task completion. Include only PNG/MP4 preview jobs. Default inclusive32 outputs/67108864 bytes; internal constructor limits allow deterministic boundary tests. Evict oldest settled preview jobs before admitting bytes. An individually oversized output is safely discarded and fails with existing retryable JOB_REGISTRY_FULL. Retain accounting on failed disposal and reject admission rather than silently exceeding the budget. TTL removal schedules disposal; get reports expired jobs immediately, admission can reject until disposal settles. No timer/background scanning is added.
3. Inject confined ephemeral deletion from runtime config into JobRegistry. It accepts only canonical project UUID and renderer-generated previews/preview[-range]-UUID.png|mp4, checks non-symlink root/project/previews/file and canonical ancestry, and never recurses. Missing files count as disposed; unsafe/I/O failures log safe diagnostics and retain accounting. This is job output lifecycle I/O, not media ownership/GC or duplicate timeline validation. Export/speech/analysis lifetime policies remain unchanged.
4. Dispose a cancelled late returned preview, including the render-result/cancel race. Do not publish its descriptor. Requested project/revision remain immutable; old successful output remains reviewable until retention and never appears current after unrelated edits. Canonical preflight retains REVISION_CONFLICT.
5. One-shot termination waits for observed close (and Windows taskkill completion) before cleanup/rejection; kill the POSIX process group forcibly as the existing persistent worker does. A five-second wait bound permits typed failure while leaving unconfirmed-process temporary files untouched until close. This prevents cleanup racing an overflow renderer's writes.

## Risks / Trade-offs

Disk retention is bounded for settled bridge previews, not in-flight temporary renders, exports, analysis or orphaned files from crashed prior processes. Filesystem failures can block capacity; preserve safety and report this limitation. Discard races are serialized, and late cancellation rechecks after admission. Existing confined path checks cannot provide a fully atomic directory replacement guarantee on all operating systems; use canonical directories and refuse symbolic links. Native Linux evidence is local; Windows/macOS are remote obligations.

## Compatibility, migration and rollback

No public fixtures, capability versions, error catalog or schema44 changes; no migration. Existing JOB_NOT_FOUND follows expiration/eviction and JOB_REGISTRY_FULL reports retryable admission/retention exhaustion. Revert the scoped bridge changes to roll back; no project/history writes occur.

## Verification

New deterministic tests cover cancelled unsettled admission and close, late artifacts, inclusive count/byte limits, TTL/eviction disposal, oversized/failed deletion and malicious paths. Actual source/package MCP and native cached preview workflows cover stale dispatch, edit/undo/redo/reopen, cancellation and independent requests. Run all required Rust/bridge/OpenSpec gates with full logs. No passing compilation substitutes for execution; archive only after conformance and implementation checks pass.
