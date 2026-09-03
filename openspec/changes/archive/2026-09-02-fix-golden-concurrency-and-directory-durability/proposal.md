## Why

Golden cleanup and publication are not serialized across processes, so one invocation can delete another invocation's live stage or generation and later expose a dangling `CURRENT`. The Unix durability traversal also synchronizes only each file's immediate parent instead of every accepted ancestor through the generation root.

## What Changes

- Serialize every native golden invocation that reads, reconciles, captures, publishes, or cleans the shared fixture container.
- Retain a stable coordination file and release its exclusive lock through RAII on success, failure, or panic.
- Synchronize every directory ancestor of every retained reference deepest-first before generation installation.
- Never delete a generation after an unconfirmed installation result; leave strictly recognizable residue for locked reconciliation.
- Add cross-process locking, failure-release, same-digest publication, and nested-directory tests.

Non-goals: changing golden media, the selected digest, fixture or performance schemas, render tolerances, renderer semantics, public APIs, project data, or application protocols.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: Require serialized shared-fixture access and complete ancestor-directory durability for atomic golden updates and conformance.

## Impact

The change affects only the test-only golden harness in `editor-core`, its tests, fixture documentation, and the render-regression-fixtures specification. It reuses the existing `fs2` dependency and current subprocess-test pattern. There are no new dependencies, production ownership edges, public contracts, migrations, or fixture-byte changes.
