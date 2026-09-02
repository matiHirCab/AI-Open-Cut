## Context

`crates/editor-core/src/render_plan.rs` currently has a useful pure seam, but its `SceneEvaluation` retains maps and references to `Asset`, `Track`, and `TextItem`. `build_render_plan` therefore still traverses persisted timeline items, resolves transitions, evaluates keyframe expressions, and derives audio behavior while constructing FFmpeg-specific filters. ADR 0004 requires editor-core to own those semantics and hand renderers an immutable renderer-neutral `EvaluatedScene` made only of logical resources and typed instructions.

Issue #12 is the representation/evaluator milestone. Issue #13 remains responsible for replacing every production render entry point's current `SceneEvaluation` input with the new representation. The version-1 motion-graphics catalog remains fixture-only; this change uses its agreed coordinate, time, ordering, safety, and limit vocabulary without activating future scene-graph fields in `Project`.

## Goals / Non-Goals

**Goals:**

- Define an owned, immutable-by-interface `EvaluatedScene` in editor-core with no borrowed project records and no renderer/backend syntax.
- Purely evaluate current flat media, text, solid-color, and rectangle visuals into typed logical-resource, visual-layer, and audio-layer instructions.
- Snapshot the current transform, animation, text style, timing, mute/ducking, fade, transition, and stable ordering facts needed to preserve existing rendering behavior.
- Reject missing asset references, non-finite values, invalid evaluated intervals, and excessive evaluation work deterministically before renderer or filesystem I/O.
- Prove evaluation is deterministic and does not mutate the project, revision, or history.

**Non-Goals:**

- Route frame, range, draft, or export production paths through `EvaluatedScene`; issue #13 owns that substitution and output-parity proof.
- Add persisted scene-graph/component types, schema migration, public operations, batch aliases, capabilities, MCP/headless changes, or new stable errors.
- Activate future curves, masks, effects, component instances, SVG, arbitrary resources, or semantic sound events from `motion-graphics-v1.json`.
- Perform media probing, filesystem path authorization, text scratch-file creation, graphics rasterization, FFmpeg planning/execution, or artifact publication during evaluation.

## Decisions

### Add an owned editor-core scene module

Add a crate-private `evaluated_scene` module containing `EvaluatedScene` and closed typed instruction records. The scene owns strings, vectors, numeric values, and logical asset identifiers. It does not borrow `Project`, `Track`, `TimelineItem`, or `Asset`, so callers cannot observe later mutation and future render adapters cannot fall back to traversing persistence records.

The scene header carries canvas width/height, frame rate, and project duration. A stable `EvaluatedLayerOrder` records track-array and item-array indices; current flat items have no explicit z-index, so their observable order remains track index then item index. Time spans are integer-millisecond half-open intervals. Visual instructions are a closed enum for media, text, solid color, and rectangle. Audio instructions are separate from media visuals so mute, volume automation, fades, and resolved ducking intervals are explicit even for audio-only assets.

Alternative considered: rename the existing borrowed `SceneEvaluation`. Rejected because a new name would hide the same architectural leak: renderer planning would still inspect persisted records and reconstruct semantic behavior.

### Snapshot backend-neutral values, not FFmpeg expressions

The evaluator copies typed transforms, keyframes, text styling, colors, media trim/timing, transition fades, and audio automation into evaluated records. It resolves track hidden/muted state, item hidden state, asset media facts, input ordering, and voiceover intervals. It does not format seconds, construct `if(...)` expressions, assign FFmpeg input indices, or embed prepared paths. Those conversions remain downstream and issue #13 will adapt `build_render_plan` to them.

Media resources are referenced by logical asset ID. The evaluated resource table includes only renderer-neutral metadata needed for planning, such as media kind and audio presence; project-relative paths remain outside the scene and are resolved through the existing path-policy/resource preparation boundary after evaluation.

Alternative considered: store current filter fragments in the scene to minimize issue #13. Rejected because raw backend expressions would make the model FFmpeg-specific and violate the architecture and safety contract.

### Make limits explicit at the canonical evaluator

Use named editor-core constants aligned with the accepted fixture vocabulary: at most 4,096 evaluated visual layers, 4,096 logical media resources, 4,096 evaluated audio layers, and 10,000 keyframes per property channel. Count work while traversing and return `INVALID_ARGUMENT` as soon as an inclusive limit would be exceeded. Existing validation remains the first line of defense, while the evaluator defensively checks every copied floating-point value and all derived values for finiteness and verifies non-empty half-open intervals.

The evaluator returns the existing `ASSET_NOT_FOUND` code for a media item whose logical asset reference is missing. All failures happen before path resolution or any render process/artifact adapter call. No new error catalog entry is needed.

Alternative considered: rely only on project validation and vector allocation failure. Rejected because evaluation is a canonical trust boundary and later bounded expansion needs stable, testable work limits before allocation or backend execution.

### Preserve compatibility without changing persistence or transports

`EvaluatedScene` is derived process-local data and is not serialized into project files or exposed through headless/MCP. Therefore schema version 6, migrations, current state, retained undo/redo snapshots, optimistic revisions, batch aliases, public fixtures, and capability reports remain unchanged. Tests compare project serialization and revision before/after evaluation and cover reopen/history as not applicable rather than manufacturing a persistence change.

The existing production render path remains untouched except for shared helper extraction that is behavior-preserving and required by both evaluators. This keeps issue #12 independently reviewable and leaves issue #13 a clear consumer migration.

Alternative considered: switch the renderer in the same change. Rejected because it combines model correctness with output-parity and transport integration work explicitly assigned to issue #13.

## Risks / Trade-offs

- [The new model temporarily exists beside `SceneEvaluation`] → Keep the old type unchanged for production, add architecture tests that forbid persisted/backend types in the new module, and remove the duplicate path in issue #13.
- [Copying owned strings and instruction data increases memory use] → Enforce pre-allocation limits and accept the bounded copy as the cost of immutability and decoupling from persisted state.
- [A flat model could accidentally anticipate future fixture-only contracts] → Keep enums limited to currently rendered item semantics and require later milestones to extend them through their own OpenSpec changes.
- [Limit enforcement can reject unusually large projects that previously reached FFmpeg] → Use named documented limits aligned with the accepted catalog and fail deterministically with `INVALID_ARGUMENT` before expensive work.
- [Planner and evaluator semantics could drift before issue #13] → Add focused equivalence assertions over current flat fixtures and make issue #13 replace the old traversal rather than evolve both paths.

## Migration Plan

1. Introduce the crate-private model, evaluator, limits, and focused tests without changing renderer call sites.
2. Update ADR/implementation documentation to identify the concrete scene module and the temporary compatibility seam.
3. Verify OpenSpec, formatting, strict Clippy, and workspace tests.
4. Issue #13 will migrate all render entry points to the scene and delete the borrowed `SceneEvaluation` traversal after visual/audio parity succeeds.

Rollback removes the new derived module, its tests, and documentation. No persisted or external data rollback is required.

## Open Questions

None. Public exposure and full renderer routing are deliberately deferred to later approved milestones.
