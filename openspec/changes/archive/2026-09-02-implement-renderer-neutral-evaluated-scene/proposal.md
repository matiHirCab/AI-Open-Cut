## Why

The current render-planning seam still exposes borrowed `Project`, `Track`, and timeline-item data through `SceneEvaluation`, so renderer planning continues to interpret persisted editor state instead of consuming a canonical immutable scene. Issue #12 establishes the renderer-neutral `EvaluatedScene` foundation required before later motion-graphics milestones can add hierarchy and before issue #13 routes every render entry point through it.

## What Changes

- Add an editor-core-owned immutable `EvaluatedScene` model with explicit canvas, timing, stable layer order, managed media-resource requests, visual instructions, and audio instructions.
- Add a pure evaluator for the existing media, text, rectangle, and solid-color items that snapshots all renderer-observable values without changing their current output semantics.
- Enforce finite evaluated values, valid half-open timing, missing asset detection, and explicit scene, resource, visual-layer, and audio-layer complexity limits before renderer planning or I/O.
- Keep renderer-specific expressions, prepared filesystem paths, text scratch files, and backend types outside `EvaluatedScene`.
- Add deterministic unit and regression coverage for success, ordering, invalid input, missing references, limits, and input-project immutability.
- Update architecture documentation to record the concrete model and the boundary retained for issue #13.
- Non-goals: change the persisted project schema, migrations or retained history; add timeline/headless/MCP operations or capability reporting; activate fixture-only future motion-graphics concepts; route all production render entry points through the new representation; or change preview/export output.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `motion-graphics-architecture`: Define the first concrete renderer-neutral `EvaluatedScene` contract, its flat-item compatibility behavior, deterministic ordering, validation, complexity limits, and renderer-boundary exclusions.

## Impact

- Primary code impact is confined to `crates/editor-core`, especially scene evaluation and focused tests, with documentation updates under `docs/`.
- No public request, response, MCP, provider, capability, stable-error catalog, or persisted project contract changes.
- No schema bump or migration is required because the evaluated scene is derived, immutable, process-local state and is never serialized as project data.
- Existing renderer planning and output remain compatible; issue #13 owns switching all render entry points to consume the new model.
