## Why

Issue #73 requires process-local preview cancellation, stale-revision isolation and bounded disposable artifacts. Cancelled entries currently release admission before their task settles, and registry removal leaves encoded preview files behind.

## What Changes

- Keep cancelled work charged to job capacity until its producer settles; wait for all admitted work during close.
- Retain completed PNG/MP4 previews under a process-local inclusive 32-artifact/64MiB budget. Evict oldest settled previews, delete only their confined disposable outputs, and reject an individually oversized preview with existing retryable JOB_REGISTRY_FULL after safe disposal.
- Dispose cancelled late preview results, expired/evicted previews and bridge-owned previews on close, preserving exports, project/media/history and unrelated files.
- Confirm one-shot process termination before temporary cleanup, including overlap with the persistent renderer.
- Preserve immutable requested revisions: edits do not relabel completed output or cancel independent review; stale dispatch remains editor-core REVISION_CONFLICT.

## Capabilities

### New Capabilities

### Modified Capabilities

- `agent-bridge`: settled admission, termination and immutable revision lifecycle.
- `artifact-resources`: bounded encoded preview retention and confined disposal.

## Impact

Bridge jobs, headless overflow execution and confined ephemeral artifact I/O; deterministic unit/native MCP lifecycle evidence and documentation. All public schemas, error catalogs, capability versions, persisted schema44 and renderer semantics remain unchanged. No migration is needed. The process-local preview file budget is distinct from editor-core's immutable encoded cache, and does not bound temporary in-flight render disk usage.

## Non-goals and approval

No durable jobs/restart cleanup, renderer rewrite, export deletion, media garbage collection, fixture deduplication, creative benchmark or GUI expansion. This bounded issue-scoped proposal/design/spec/tasks is approved through the user's explicit delegated specification authority before implementation; no additional permission is inferred.
