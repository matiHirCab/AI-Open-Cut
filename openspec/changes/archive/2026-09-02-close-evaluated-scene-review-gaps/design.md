## Context

`EvaluatedScene` is an internal, renderer-neutral derivation that is exercised by the existing renderer compatibility traversal but is not yet the render planner's consumed representation. Its current preflight establishes missing-asset precedence and bounds major output collections. Two review gaps remain: non-image media source intervals can overflow or exceed known asset duration, and the merged voiceover activity vector is cloned into every ducked music layer.

The correction must preserve project schema version 6, every public contract, current valid rendering, the resource-binding sidecar boundary, and issue #13's responsibility for switching renderer planning to this model.

## Goals / Non-Goals

**Goals:**

- Fail closed on overflowing or out-of-bounds video/audio source ranges before complexity checks and output allocation.
- Preserve `ASSET_NOT_FOUND` precedence and the current image source-offset exception.
- Bound voiceover activity derivation by 10,000 positive ranges before merging.
- Store the merged voiceover activity table once per scene and keep ducking settings per layer.

**Non-Goals:**

- Changing public or persisted structures, stable errors, capabilities, revisions, or history.
- Changing valid renderer output or routing render planning through `EvaluatedScene`.
- Moving source-range validation into timeline mutation commands.
- Changing path resolution or `SceneResourceBindings`.

## Decisions

### Validate source intervals in a dedicated pass

Evaluation order is canvas validation, asset indexing, missing referenced-asset validation, non-image media source-range validation, complexity preflight, and only then output construction and voiceover interval derivation. Video and audio use checked `source_in_ms + duration_ms`; a known asset duration is an inclusive upper boundary for the source end. Images skip this validation because current rendering does not consume their source offset.

This dedicated pass is preferred over folding validation into output construction because it prevents partial allocations and preserves deterministic error precedence. Mutation-time validation was considered, but `TrimItem` and evaluation of retained state have different responsibilities and that broader contract change is out of scope.

### Count positive voiceover ranges during preflight

For every audible voiceover media item, preflight derives only that item's positive volume ranges and immediately adds their count to a checked accumulator capped at 10,000. Per-item temporary work is already bounded by the 10,000-keyframe-per-channel limit. The full scene candidate table is not allocated until the aggregate count passes.

Bounding merged ranges was rejected because highly overlapping inputs could still cause unbounded evaluator work before merging. Counting audio layers alone was rejected because one keyframed voiceover item can emit many activity ranges.

### Own merged intervals at scene scope

`EvaluatedScene` owns one deterministically sorted and merged `voiceover_intervals` vector. `EvaluatedDucking` retains only gain, attack, and release settings, and is present only when a music track enables ducking and the scene has voiceover activity.

Keeping an `Arc` or another shared pointer inside every ducking record was considered, but a scene-global table expresses the ownership directly, avoids repeated handles, and remains renderer-neutral.

### Preserve the compatibility boundary

The current renderer continues to call evaluation as a correctness check and then uses its existing borrowed planning representation. It does not consume the newly arranged interval table in this correction. Issue #13 remains responsible for that routing change.

## Risks / Trade-offs

- [Per-item activity derivation occurs once in preflight and again during final interval construction] → Both passes are bounded; this avoids retaining potentially large temporary data before validation and keeps allocation after preflight.
- [Previously tolerated malformed non-image source timing now fails] → Return the existing `INVALID_ARGUMENT` code and test boundary/precedence behavior explicitly.
- [Future consumers could reintroduce per-layer copies] → Add structural architecture assertions proving interval ownership remains on `EvaluatedScene` and absent from `EvaluatedDucking`.

## Migration Plan

No persisted or public migration is required. Implement and verify the internal evaluator and documentation atomically. Rollback consists of reverting this corrective change; project files and public clients require no conversion.

## Open Questions

None.
