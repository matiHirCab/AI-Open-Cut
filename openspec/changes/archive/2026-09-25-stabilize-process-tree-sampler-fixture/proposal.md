## Why

PR #124's Windows correctness job failed because the process-tree sampler test's child freed its 64 MiB allocation after 300 ms before the sampler observed it. The child exited successfully, but the test's memory-delta assertion failed; a timing-dependent fixture must not block required CI.

## What Changes

- Keep the test child alive until the parent observes its allocation or a bounded timeout expires, using an explicit cross-process readiness and release handshake.
- Require the existing memory-delta assertion to remain effective, and always release and reap the child even when observation fails.
- Rerun the focused Windows test and required correctness, render, and OpenSpec gates.

Non-goals: changing production sampler behavior, its 5 ms interval, memory metric, golden references, rendering output, or CI policy. No public or persisted contracts change; there are no breaking changes or migrations.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: make the child-allocation sampler regression fixture deterministic and bounded across Windows, Linux, and macOS.

## Impact

The change affects only the test fixture in `crates/editor-core/src/renderer/golden.rs` and its OpenSpec coverage. It does not add dependencies or change APIs, schemas, contracts, production rendering, or the required CI job structure.
