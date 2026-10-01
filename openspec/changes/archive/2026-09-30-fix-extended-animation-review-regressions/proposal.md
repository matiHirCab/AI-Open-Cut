## Why

The complete review of issue #43 reproduced five violations of the approved extended-animation requirements despite passing suites and an archived conformance report. Unsafe scale/crop candidates can commit, identity effects move legacy paths, schema-27 migration publishes invalid retained drafts, and extended sampling loses large-integer timestamp precision.

## What Changes

- F1: Certify continuous local and inherited transform envelopes for extended occurrences independently of rotation-channel presence, rejecting unsafe scale overshoot before publication.
- F2: Preserve legacy path coordinate origins and anchors when an item enters extended rendering, including identity effects and zero rotation.
- F3: Validate retained version-2 draft candidates during schema-27 adoption even when their operations and base contain no extended fields; preserve established valid-stale-draft behavior.
- F4: Account for the interpolated crop extent floor in correlated crop bounds, while preserving exact authored endpoints and holds.
- F5: Preserve root integer clock precision through actual extended scalar and compound consumers, including endpoints, holds, curves, and loops; preserve fractional inherited clocks.
- Add independent regression oracles, rollback/lifecycle coverage, and native intent comparisons. Correct the archived verification claim with a clearly identified superseding record after verification.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `animation-channels`: Clarify rotation-independent continuous bounds, clamp-aware crop correlation, and exact integer-clock sampling with concrete regression scenarios.
- `project-persistence`: Explicitly cover invalid retained legacy-only draft operations before schema-27 publication without changing valid stale-draft behavior.
- `rendering-export`: Explicitly preserve offset legacy path origins through identity extended rendering and all render intents.

## Impact

Implementation belongs in `crates/editor-core`, principally animation sampling, evaluated-scene certification/affines, and retained-draft migration. Rust regression and native render tests change; headless/MCP lifecycle tests may add evidence using their existing typed operations. Documentation and OpenSpec evidence must match final behavior. No new dependency edge, transport-side semantic validator, provider behavior, or warning suppression is authorized.

Compatibility classification: corrections restore the existing approved requirements, not a new public contract meaning. Public operation names, request/response shapes, catalogs, capability identifiers, error codes/retryability, and schema 27 remain unchanged. Previously accepted unsafe or malformed work becomes rejected under existing documented bounds and stable failures; valid legacy/static output and valid stale drafts remain compatible. A correction requiring a new public/persisted declaration, restriction of previously valid work, or contract redefinition requires an amended proposal and approval before implementation.

## Non-goals

No activation of deferred channels, new effect algorithms, new timing or target APIs, schema 28, regenerated goldens, CI policy changes, unrelated cleanup, commits, pushes, or PR creation. Preserve all existing unrelated/uncommitted work.

## Approval status

The user explicitly approved this proposal, design, three delta specs and task list on 2026-09-30 with the response "Approve" after the concrete artifacts were presented and strict validation passed. Implementation of F1-F5 through these tasks is authorized. No new contract or stale-base policy is authorized beyond this approved scope.
