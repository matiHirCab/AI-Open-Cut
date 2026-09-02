## Why

The renderer-neutral scene evaluator still accepts invalid non-image source ranges and duplicates the merged voiceover activity table into every ducked music layer. These gaps can defer malformed timing to the renderer and make evaluator memory grow with the product of voiceover activity and ducked layers.

## What Changes

- Validate video and audio source ranges after referenced assets are resolved and before complexity checks or output allocation, using checked arithmetic and known asset durations while preserving the image exception.
- Bound positive, pre-merge voiceover activity ranges at 10,000 before allocating the shared scene table.
- Store merged voiceover intervals once on `EvaluatedScene`; keep only gain, attack, and release settings on each `EvaluatedDucking`.
- Extend evaluator, architecture, and documentation coverage for the validation order, boundary cases, linear storage, and unchanged ducking semantics.
- Create, validate, sync, and archive this correction without modifying either existing archived change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-architecture`: Require checked non-image media source ranges, a bounded pre-merge voiceover activity count, and a scene-global merged voiceover interval table.

## Non-goals

- No public Rust, JSON, MCP, or headless contract changes.
- No persisted schema, migration, capability catalog, error catalog, revision, or history changes.
- No change to valid render output or to the renderer-planning migration owned by issue #13.
- No mutation-time source-range validation changes to `TrimItem`.

## Impact

The change is limited to the internal `editor-core` evaluated-scene model and evaluator, its tests and architecture assertions, ADR 0004, the motion-graphics implementation plan, and the living `motion-graphics-architecture` OpenSpec. Existing `SceneResourceBindings` behavior and renderer compatibility traversal remain unchanged. Invalid inputs continue to use `INVALID_ARGUMENT`, with `ASSET_NOT_FOUND` retaining precedence.
