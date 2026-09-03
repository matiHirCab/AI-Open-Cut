## Why

The issue #15 implementation added the intended stage observations, but review found four correctness gaps: ordinary golden conformance still executes benchmark-only probes, the uploaded report receives only partial CI validation, update mode applies a permissive migration validator to current fixtures, and decode/composite failure behavior lacks injectable coverage. These gaps weaken isolation and fail-closed fixture guarantees.

## What Changes

- Run stage probes only for explicit report, recapture, or update modes; ordinary conformance retains its prior render-only behavior.
- Strictly validate every aggregated and serialized schema-3 performance report before installation or upload, using the editor-core-owned validator in CI.
- Restrict migration acceptance to the reviewed revision-2/schema-2 predecessor and strictly validate current revision-3 generations.
- Add test-private stage-aware probe execution so decode, composition, and encoding failures are covered independently, stop subsequent work, and leave no artifacts or state changes.
- Keep performance schema 3, stage-definition version 1, and all public and persisted contracts unchanged.

## Capabilities

### Modified Capabilities

- `render-regression-fixtures`: Tightens benchmark opt-in behavior, report validation, fixture migration, and stage-failure evidence.

## Impact

- Affects only editor-core golden benchmark internals, their tests, required Linux CI validation, and renderer fixture documentation/specification.
- No public API, MCP/headless contract, project schema, runtime dependency, or user-data migration changes.
- Rollback restores the prior test-only capture and validation paths; checked-in deterministic references remain usable.
