## Why

Issue #41 requests finite, infinite, and ping-pong animation loops. Typed channels and deterministic curves are already active, but editor-core currently rejects loops and holds the final keyframe after its timestamp. Previewing a subrange must not change the phase of an animation.

## What Changes

- Add an optional closed `loop` record to an active typed animation channel. It selects forward repeat or ping-pong playback and a bounded finite count or infinite repetition.
- Define item-local loop boundaries, exact seam samples, finite exhaustion, and range-independent phase mapping in editor-core. Preserve the existing curve sampler and property bounds.
- Keep `set_animation_channels` and `timeline_batch_edit` as the editing surfaces; add no operation. Publish typed, additive headless and MCP input, a support capability, canonical fixtures, and user documentation.
- Advance project schema 24 to 25 and atomically migrate current state and retained undo/redo history. Projects without loops keep identical sampled and rendered output.
- Verify equivalent evaluated visual/audio behavior and decoded output through frame preview, audiovisual range preview, draft preview, and final export.

## Capabilities

### New Capabilities

- `animation-loops`: Typed loop validation, timing, boundary, and sampling behavior for active animation channels.

### Modified Capabilities

- `animation-channels`: Permit the optional loop record on active channels without changing unlooped keyframes or curves.
- `project-persistence`: Schema-25 migration and fail-closed history validation.
- `motion-graphics-contracts`: Canonical loop fixtures, parity, and capability discovery.
- `agent-bridge`: Typed loop input through existing standalone and batch editing.
- `rendering-export`: Shared range-independent loop evaluation across render intents.

## Impact

Editor-core owns loop validation, phase mapping, persistence, and evaluated-scene semantics. Headless and agent-bridge forward typed data without duplicate domain checks. `contracts/` governs the additive public and persisted shape under protocol major 1. No new desktop UI, animation property, external asset source, FFmpeg expression, or legacy `set_keyframes` loop is introduced. Existing operations, fields, stable errors, and retryability remain compatible; older binaries reject schema 25 rather than downgrade it. Issues #38 and #39, the declared dependencies, are closed.
