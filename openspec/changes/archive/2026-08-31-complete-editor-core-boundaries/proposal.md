## Why

Review of the issue #83 extraction found that several declared boundaries are present only structurally: renderer input planning still occurs in the facade, process and artifact adapters cannot be injected through that facade, asset garbage collection still performs filesystem work in `store`, and architecture checks do not enforce the complete dependency graph. These gaps must be closed before later `EvaluatedScene` work can rely on the boundary.

## What Changes

- Make render evaluation produce ordered logical inputs, resource requests, timing, dimensions, and output intent before artifact preparation or process execution.
- Route renderer readiness, probing, execution, publication, metadata, and cleanup through private injectable production adapters while preserving the public `Renderer` API.
- Move asset garbage-collection decisions and operations out of `store`, perform managed-file I/O through the persistence storage port, and remove the reverse `assets -> drafts` dependency.
- Replace partial source checks with an explicit allowed-dependency matrix and targeted ownership checks for every editor-core owner.
- Add deterministic facade, plan, garbage-collection, and architecture regression tests.

### Non-goals

- No public Rust API, headless protocol, persisted schema, stable error/warning, preview, or export behavior change.
- No implementation of the roadmap `EvaluatedScene` model from issues #12 or #13.
- No complete rewrite of draft lifecycle or persistence construction beyond the four reviewed boundary gaps.
- No pull request creation or unrelated documentation changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `editor-core-architecture`: Clarify the required contents of deterministic render plans, facade-level adapter replaceability, asset-GC ownership, and exhaustive enforcement of the allowed internal dependency graph.

## Impact

- Primary code: private renderer planning/process/artifact modules, asset and persistence boundaries, `store` orchestration, and editor-core architecture tests.
- Compatibility surfaces: `Renderer`, `EditorCore`, public DTOs, serialized project/history/draft data, stable errors/warnings, and native output behavior remain unchanged.
- Verification: focused editor-core tests plus existing workspace, headless, bridge integration, packaged smoke, native render, and OpenSpec gates.
