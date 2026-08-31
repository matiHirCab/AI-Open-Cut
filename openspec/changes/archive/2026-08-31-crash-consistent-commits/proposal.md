## Why

Project mutations currently publish `project.json` and `history.json` through separate replacements, so process termination or an I/O failure can expose documents from different logical revisions. Draft commit also reports cleanup failure after publishing the mutation, leaving clients unable to distinguish rejection from a committed result and making retries unsafe.

## What Changes

- Define one recoverable transaction boundary covering project state, retained undo/redo history, and consumption of a committed draft.
- Persist enough transaction intent to recover deterministically on project open after failure before, between, or after publication phases.
- Reject corrupt or irrecoverable transaction state with a stable, non-retryable diagnostic instead of guessing which generation is authoritative.
- Make draft commit retry-safe so a retry cannot apply the draft operations twice, and return a successful committed result with stable cleanup/recovery warnings when post-commit cleanup is incomplete.
- Remove managed transaction artifacts after successful publication or recovery, while preserving project locking, optimistic revisions, migrations, and existing successful public behavior.
- Add phase-by-phase fault-injection coverage and precise persistence/recovery operational documentation.
- Non-goals: changing the project schema, history retention limit, draft operation semantics, lock implementation, optimistic revision rules, or introducing cross-project transactions.
- No breaking public contract change is intended. New stable diagnostics and warnings are additive compatibility-surface changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-persistence`: Strengthen durable persistence from per-document atomic replacement to recoverable logical transactions for project state and retained history, including deterministic open-time recovery and stable irrecoverable-state diagnostics.
- `edit-drafts`: Include committed draft consumption in the transaction protocol and define retry-safe results when publication succeeds but cleanup requires recovery.

## Impact

- Primary implementation: `crates/editor-core/src/store.rs`, persistence models/helpers, and targeted fault-injection tests.
- Compatibility surfaces: `crates/editor-core/src/error.rs`, `contracts/error-codes-v1.json`, headless error/warning serialization, contract fixtures, and any cross-language catalog assertions.
- Persisted filesystem layout: a versioned, managed transaction artifact may temporarily coexist with `project.json`, `history.json`, and `drafts/*.json`; project open becomes the deterministic recovery boundary.
- Documentation and requirements: `openspec/specs/project-persistence`, `openspec/specs/edit-drafts`, and operational persistence documentation.
- Dependencies: no new external runtime dependency is expected.
