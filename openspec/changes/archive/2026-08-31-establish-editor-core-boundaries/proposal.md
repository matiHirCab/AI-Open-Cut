## Why

`editor-core` currently concentrates persistence, migrations, asset lifecycle, timeline operations, and drafts in one store module, while renderer planning, scene evaluation, FFmpeg execution, and artifact I/O share another large module. Establishing explicit ownership seams now prevents upcoming scene-graph, animation, compositing, and audio work from increasing coupling or creating parallel domain rules.

## What Changes

- Record an architecture decision and ownership map for the editor model, canonical validation, persistence, migrations, asset lifecycle, timeline operations, drafts, scene evaluation/render planning, process execution, and render artifact I/O.
- Split store responsibilities into cohesive internal modules while preserving the `EditorCore` facade, serialized project/draft/history shapes, stable errors and warnings, revision behavior, and existing headless/bridge callers.
- Separate deterministic renderer planning and scene evaluation from FFmpeg process execution and artifact publication behind narrow, injectable interfaces suitable for fault and deterministic tests.
- Add architecture checks and review rules that enforce inward dependencies and keep canonical validation in `editor-core` rather than bridge or desktop code.
- Coordinate the renderer boundary with the future `EvaluatedScene` work tracked by issues #12 and #13 without implementing that roadmap model in this change.
- Keep focused unit and integration coverage beside each extracted owner and retain compatibility coverage for reopen, preview, and export behavior.

### Non-goals

- No persisted schema, public request/response, MCP, stable error, or provider-contract change.
- No new timeline, scene-graph, animation, compositing, audio, or rendering behavior.
- No replacement of the existing `EditorCore` public facade or renderer entry points.
- No introduction of validation in headless, bridge, desktop, or renderer execution layers.

## Capabilities

### New Capabilities

- `editor-core-architecture`: Defines canonical responsibility owners, allowed dependency direction, narrow infrastructure seams, compatibility-preserving extraction rules, and automated architecture enforcement for `editor-core`.

### Modified Capabilities

None. Existing persistence, media, timeline, draft, preview, and export requirements remain behaviorally unchanged.

## Impact

- Primary code: `crates/editor-core/src/store.rs`, `crates/editor-core/src/renderer.rs`, new cohesive internal modules, and `crates/editor-core/src/lib.rs` exports.
- Architecture evidence: a new ADR/ownership map, repository contributor rules, and dependency-boundary tests.
- Verification: focused module tests plus existing editor-core, headless protocol, bridge integration/smoke, formatting, Clippy, and workspace suites.
- Compatibility surfaces: persisted project/history/draft JSON, `EditorCore` and renderer public Rust APIs, headless protocol, stable errors/warnings, preview frames, and exported artifacts are preserved. This change is non-breaking.
