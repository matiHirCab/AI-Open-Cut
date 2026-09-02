## Why

The immutable golden-generation implementation can delete the newly selected generation when the Unix pointer rename succeeds but its later directory sync fails. It also postpones cleanup of recognized interrupted work until another successful update instead of reconciling it on the next harness invocation.

## What Changes

- Classify pointer replacement as uncommitted or committed independently from post-rename durability warnings.
- Preserve both the previous and newly selected generations whenever pointer durability is uncertain, then reconcile safely on a later invocation.
- Reconcile recognized stages, pointer temporaries, and inactive validated generations at the start of every golden harness invocation as well as after successful publication.
- Replace the ineffective pointer failure injection with explicit pre-rename and post-rename failure points and add reopen coverage for both possible persisted pointers.
- Clarify that ordinary conformance never rewrites the selected generation or pointer but may perform bounded orphan cleanup.
- Non-goals: recapturing golden evidence, changing render tolerances or semantics, changing performance schema 2, or modifying public, persisted, headless, MCP, or provider contracts.
- Compatibility: all changed state and result types remain private to editor-core tests; the checked-in Linux generation and digest remain unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: Clarifies the pointer commit boundary, uncertain-durability retention, and next-invocation reconciliation behavior.

## Impact

- Affects only the editor-core golden test harness, its failure-injection tests, golden documentation, and OpenSpec requirements.
- Adds no dependencies and changes no production code or external interface.
