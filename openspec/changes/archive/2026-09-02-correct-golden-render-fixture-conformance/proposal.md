## Why

The issue-14 golden fixture infrastructure currently has correctness gaps in interrupted publication, URI rejection, decoded-audio alignment, and performance sampling. These gaps can leave the canonical fixture unavailable after interruption, accept forbidden reference syntax, reject valid codec-delayed audio, and publish timing or memory observations that do not represent the complete render process tree.

## What Changes

- Store reviewed golden sets as immutable content-addressed generations selected by one atomically replaced `CURRENT` pointer, with bounded cleanup of recognized staging and orphan-generation data.
- Align decoded PCM in both directions within the existing one-frame limit before calculating RMS, while retaining the existing RMS and timing tolerances.
- Reject every RFC 3986-style URI scheme in golden reference paths in addition to existing absolute-path, traversal, and fixture-root escape checks.
- Separate ordinary conformance from report capture; performance capture performs one discarded warm-up and three measured runs, verifies deterministic evidence across samples, and reports median timings plus maximum process-tree resident memory.
- Version the internal performance report and fixture revision, regenerate the reviewed Linux-compatible generation, and update documentation and CI artifact evidence.
- Non-goals: changing production render semantics, public/headless/MCP/provider contracts, project schema, revisions, undo/redo, migrations, tolerance values, or introducing release performance budgets.
- Compatibility: the fixture directory layout and internal performance-report schema change additively to test infrastructure. No public or persisted compatibility surface changes, and no breaking runtime change is introduced.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: Clarifies atomic generation publication, URI syntax rejection, bidirectional PCM alignment, sampling aggregation, and process-tree memory scope.

## Impact

- Affects editor-core test-only golden capture and validation support, its development dependencies, checked-in render fixtures, golden documentation, and Linux CI evidence.
- Adds development-only cross-platform process inspection and Windows atomic file-replacement support.
- Does not change production dependencies, runtime APIs, serialized projects, stable errors, or cross-language contracts.
