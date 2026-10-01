## Why

Issue [#43](https://github.com/matiHirCab/AI-Open-Cut/issues/43) requires rotation, crop, path, gradient, and effect animation to reach every production render intent. The current catalog declares these channels but rejects them; renderer-neutral evaluation must implement their semantics before they become writable.

## What Changes

- Activate `transform.rotation_deg`, the four `media.crop_*` channels, `graphic.path_points`, `graphic.path_trim`, `graphic.gradient_stops`, and blur/glow/tint/vignette effect channels with explicit compatible targets, finite bounds, and deterministic sampling.
- Add bounded static media crop and ordered per-item effects needed as channel targets and fallback values. Extend typed scoped channel references to identify authored shape graphics and effects without arbitrary property paths.
- Share sampled geometry, paint, crop, and effects through EvaluatedScene for frame, range, draft preview, and export, preserving inherited clocks, loops, stacking, and static fallback.
- Advance persisted schema 26 to 27 and migrate current state, undo/redo snapshots, and retained draft state atomically with identity defaults.
- Update canonical catalogs, consumer parity, capability reporting, documentation, and deterministic render regression evidence before advertising support.

## Capabilities

### New Capabilities

- `visual-effects`: Typed bounded static blur, glow, tint, and vignette stacks supporting effect-channel targets.

### Modified Capabilities

- `animation-channels`: Activate the selected channels, scoped targets, compound interpolation, and transactional validation.
- `rendering-export`: Share complete sampled transforms, source crops, vector geometry, gradient paint, and effect order across output intents.
- `project-persistence`: Atomic schema-27 migration with unchanged legacy output and retained history/draft validation.
- `motion-graphics-contracts`: Govern additive activation metadata, target shapes, defaults, capability reporting, and consumer fixtures.

## Impact

Core model, validation, animation, migration, evaluated scene, render planning/artifacts, and their tests own the behavior. Headless and bridge only deserialize typed edits and translate core results. Reuse existing visual-property and animation-channel standalone/batch edits; no new top-level operation or provider dependency is needed. Shape primitive limits and the ADR 0003 dependency graph remain authoritative.

Public contracts are additive: old accepted requests retain their meaning, existing errors/retryability and identifiers remain stable, and newly supported channels receive a separate `extended_visual_animation_v1` capability. Schema 27 is an explicit persisted version transition; older binaries reject it. Canonical artifacts and governed consumers require the existing CODEOWNER review.

## Non-goals

Do not activate source-position/playback-rate, skew/anchor, fill/stroke/stroke-width, audio-pan, or particle channels. Particle simulation and the other roadmap effects remain inactive. Do not add masks, new SVG execution, network resources, raw renderer expressions, desktop inspectors, provider behavior, or parallel transport-side validation.

## Approval

Approved by the user on 2026-09-30 in this chat (response: "Approve"). Implementation is authorized through tasks.md. The channel subset and semantics in the accompanying delta specifications describe the approved outcome, not claims about current support. Designated CODEOWNER review of the resulting contract changes remains a final review requirement.

Final implemented contract review approved by the user acting as designated CODEOWNER @matiHirCab on 2026-09-30 (response: Approve implemented contracts). Approval covers schema 27 migration, scoped targets, crop/effects, activated channels/capability, and synchronized headless/MCP catalogs, and authorizes completing specification synchronization, archival, and the final protected gate.
