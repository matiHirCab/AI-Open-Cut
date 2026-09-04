## Why

The issue #15 follow-up correctly restricts migration to revision 2 and makes benchmark process failures injectable, but review found two remaining lifecycle gaps. Migration currently treats a report containing only `schemaVersion: 2` as complete and does not enforce the canonical manifest metadata, while a measured-capture panic drops the memory sampler handle without signaling or joining its worker.

## What Changes

- Validate the exact prior revision-2/schema-2 report shape and every canonical manifest invariant before accepting a migration source or recognizing it for cleanup.
- Make process-tree sampler shutdown RAII-safe so explicit completion and unwinding both stop and join the worker exactly once.
- Enforce the exact schema-3 stage-work counts implied by canonical fixture revision 3 for every benchmark intent.
- Require schema-3 reports to contain Git identity explicitly, using null when unavailable or a nonblank string when present.
- Add malformed legacy fixture and sampler-unwind coverage.
- Preserve performance schema 3, stage-definition version 1, deterministic golden references, and all public and persisted contracts.

## Capabilities

### Modified Capabilities

- `render-regression-fixtures`: Completes legacy migration validation and guarantees bounded sampler cleanup on early exit.

## Impact

- Affects only editor-core golden fixture test infrastructure, its documentation/specification, and tests, including strict schema-3 report validation.
- Adds no runtime dependency, public API, transport contract, project migration, or fixture recapture.
- Rollback restores the prior test-private validators and sampler lifecycle; checked-in current fixtures remain unchanged.
