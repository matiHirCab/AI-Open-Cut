## Why

Issue #72 is open and its dependencies #71 (raster reuse) and #15 (bounded execution) are closed. Existing raster caching avoids repeated text/vector rasterization but every preview still executes the final audiovisual encoder. Equal immutable previews should reuse successful encoded media while preserving canonical preflight and publication behavior.

## What Changes

- Add an internal bounded process-local preview artifact cache shared by renderer clones and the existing persistent render worker.
- Key frame/range previews by project identity, revision, exact requested time span, dimensions, FPS, audio choice, complete effective snapshot/draft identity and verified rendering dependencies.
- Retain at most32 entries and67108864 accounted bytes with LRU eviction; oversized or unsupported caching remains uncached.
- Reuse successful encoded bytes through fresh owned preview publication, preserving current result/warnings/layout/progress semantics and canonical validation.
- Prove exact native cold/warm media identity, invalidation, warm validation failures, concurrency, atomic lifecycle and actual cross-request agent reuse.

## Capabilities

### New Capabilities

- `preview-artifact-caching`: revision-keyed disposable encoded frame/range preview reuse with complete identity, bounded retention and preflight-safe publication.

### Modified Capabilities

None. Existing raster caching and rendering/export requirements continue to apply. Export consumes the unchanged evaluated semantics and does not reuse a preview artifact.

## Impact

Editor-core renderer/artifact orchestration and tests; existing headless worker instrumentation only behind explicit test gates; source/packaged bridge integration tests and docs. No public operation, error, response, capability, schema, provider or speech-worker contract changes. No persisted cache or migration. No merge or deployment.

## Approval

The user explicitly approved proposal, design, delta spec and tasks on 2026-10-09 with “yes” in response to the issue #72 scope question.
