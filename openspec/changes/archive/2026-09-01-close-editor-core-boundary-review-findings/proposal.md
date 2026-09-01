## Why

PR #94 establishes editor-core ownership boundaries, but its persistence and render-artifact adapters still leave required filesystem behavior hard-wired, and its architecture test can still miss legal Rust forms or responsibility duplication. Closing these gaps now prevents the next model and rendering work from depending on seams that are documented but not actually substitutable or enforceable.

## What Changes

- Route project locking, recovery, persisted reads and writes, draft filesystem operations, managed asset storage, and garbage collection through one facade-injectable persistence adapter.
- Expand the renderer artifact adapter to own workspace lifetime, temporary paths, resource and filter I/O, path inspection, publication, cleanup, and metadata.
- Make the architecture analyzer recurse through out-of-line modules, preserve aliases across module scopes, canonicalize alias chains, and inspect structured patterns.
- Replace spelling-based filesystem and garbage-collection checks with structural restrictions that reject direct managed-file I/O and duplicated collection in the wrong owner.
- Preserve every public API, persisted representation, stable error and warning, FFmpeg command, path rule, and dependency-matrix edge.
- Non-goals: changing ADR 0003's ownership graph, introducing public dependency-injection APIs, changing runtime behavior, expanding macros, or adding production dependencies.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `editor-core-architecture`: strengthen the existing persistence and artifact-I/O seams and require architecture enforcement across out-of-line modules, alias scopes, structured patterns, direct filesystem access, and renamed responsibility duplication.

## Impact

- Affected code: private persistence, store, asset, draft, renderer artifact, renderer facade, and architecture-test internals in `crates/editor-core`.
- Affected tests: persistence/recovery/draft/GC fault tests, renderer facade adapter tests, and architecture regression fixtures.
- Compatibility: no public API, serialized schema, cross-language contract, stable error, or user-visible behavior changes; no breaking changes.
- Dependencies: no new production dependency; the existing `syn` dev-dependency remains the parser used by architecture tests.
