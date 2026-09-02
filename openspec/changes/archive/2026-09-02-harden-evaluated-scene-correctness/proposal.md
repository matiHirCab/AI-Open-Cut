## Why

The initial renderer-neutral `EvaluatedScene` implementation validates some complexity only while constructing output and repeatedly scans transitions, allowing oversized persisted collections to drive allocations or superlinear work before rejection. It also lacks a path-free binding seam that preserves custom font selection for the later render-routing milestone.

## What Changes

- Preflight all referenced assets and bounded scene work before scene allocation or voiceover interval derivation, while preserving missing-asset error precedence.
- Index transition endpoint facts once in stable declaration order, cap emitted transition facts at 4,096, and preserve both directions for self-referential endpoints.
- Return an internal evaluation result containing the path-free `EvaluatedScene` and a separate resource-binding sidecar for media requests and requested font path/family selections.
- Extend evaluator, boundary, limit, ordering, and resource-binding tests and synchronize the architecture documentation.
- Keep production render routing on the existing borrowed plan until the separately approved routing milestone.

### Non-goals

- No public or persisted contract, project-schema, capability, MCP/headless operation, provider protocol, migration, revision, history, or stable-error-catalog change.
- No renderer/backend consumption of `EvaluatedScene` or `SceneResourceBindings` in this change.
- No arbitrary path, resolved filesystem path, URL, backend type, command, or prepared artifact inside `EvaluatedScene`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-architecture`: Clarify fail-closed preflight ordering, add the inclusive transition-fact limit, and define the process-local resource-binding sidecar that keeps the canonical scene path-free.

## Impact

- Affects the private evaluator in `crates/editor-core`, its architecture and unit tests, ADR 0004, and the motion-graphics implementation plan.
- Adds no dependency and changes no public compatibility surface.
- Uses existing `ASSET_NOT_FOUND` and `INVALID_ARGUMENT` behavior without changing their catalog definitions.
