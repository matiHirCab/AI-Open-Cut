## Context

Issue #43 builds on typed channels (#38), curves (#39), and static transform/shape/vector milestones (#27, #28, #34). Current schema is 26. `AnimationChannelProperty::active` permits five legacy-transform visual channels and gain; compound DTOs exist but are inactive. `AnimationChannel.target` currently uses `{scope,id}` and accepted channels reject any target. Shape items have one geometry and fill/stroke paints. `VisualProperties` has neither crop nor effects. EvaluatedScene is process-local and renderer-neutral.

## Goals / Non-Goals

Implement the approved rotation/crop/path/gradient/effect subset for all output intents with stable transactions and bounded canonical evaluation. Non-goals are listed in proposal.md, including particle simulation, additional deferred channels, masks, and new desktop UI.

## Decisions

1. **Core owns activation and target resolution.** Reuse `set_animation_channels` and visual-property edits, extending their typed inputs. Use strict additive target records `{kind,scope,id}` for shape geometry/fill/stroke and authored effect IDs. Existing targetless inputs stay unchanged; previously rejected targeted inputs acquire only the defined semantics. Do not change the hierarchy `ParentReference` contract. Extend the channel target DTO separately. Alternative: opaque property strings or transport resolution would duplicate validation and violate ADR 0003.

2. **Sample structured values before renderer work.** Canonical animation supplies bounded scalar and compound samples after existing inherited clocks, curves, and loops. Rotation interpolates authored degrees directly, permitting multiple turns. Crops use normalized source coordinates. Path point arrays follow authored command point order and require fixed topology. Gradient stops require matching stop counts and stable endpoint offsets. Resolve shape targets within the item's owning composition; effects within its own stack. Alternatives: raw FFmpeg interpolation or normalized polygons would create intent divergence or discard authored topology.

3. **Preserve the transform compatibility rule.** Existing legacy visual channels continue rejecting transform2d. Rotation is the only exception: it replaces static transform2d rotation while preserving anchor, independent scale, skew, and position. Without transform2d it rotates around the existing legacy anchor using identity skew. Adding/removing transform2d with existing five channels remains invalid. Alternative: enabling all transform channels on transform2d expands a separate compatibility promise outside #43.

4. **Introduce only required static fallbacks.** Media crop defaults to the whole source. Eligible leaf visuals get empty effect stacks. Groups, instances, repeaters, and captions reject effects in this milestone because group-level intermediate compositing is a separate behavior. Effects are ordered and ID-addressable, limited to Gaussian blur, glow, color tint, and vignette. No new module edge is needed: model defines DTOs, validation checks them, evaluated_scene resolves them, render_artifact performs bounded raster work, and render_plan consumes the evaluated result. Alternatives: invisible channel-only effects provide no stable target or static fallback; general group effects require new isolation semantics.

5. **Use a common bounded sampled visual path.** Extend the current shape/text rasterization and affine rendering seams for animated structured/effect properties. For range/export, prepare each needed sample through the same evaluator used by frame preview; backend expressions may only translate already defined scalar facts. Cache keys include every sampled visual property and time/occurrence dependency. Keep legacy output routing for unaffected projects. Alternatives: static first-frame raster caching freezes compound animation; separate export-only filters change effect semantics.

6. **Atomic schema and contract adoption.** Schema 27 adds identity crop/empty effects. Validate all retained snapshots and draft candidate state before publication under the project lock; use existing crash-consistent transaction primitives. Add `extended_visual_animation_v1` only when complete support is available. Update checked-in catalogs manually and obtain required CODEOWNER review; do not derive fixtures from registrations. Alternative: changing schema-26 acceptance silently gives older readers inconsistent semantics.

7. **Bound candidate certification as approved in approval-amendment.md.** Final candidates share at most 65536 interval-certification nodes across canonical expanded occurrence order and retained definitions, subdividing left before right. Conservative continuous envelopes cover crop coupling, gradient ordering, geometry, rotating raster support and cumulative effect work; hidden content and retained draft candidates are included. Proven unsafe candidates and unresolved safety at quota exhaustion fail with non-retryable INVALID_ARGUMENT before publication, preserving generations, revisions, aliases and resources. New work may be conservatively rejected; legacy candidates retain their existing behavior. Exact integer segment selection is preserved for legacy u64 clocks, while inherited fractional clocks use the shared continuous sampler.

Range/export sampled resources stream bounded PAM frames into a lossless FFV1 intermediate instead of retaining a per-frame image sequence. Frame, range, draft and export consume the same samples. Shared pure text measurement lets font-aware edit certification and artifact preparation agree without moving font ownership or adding an ADR dependency edge.

## Risks / Trade-offs

- Per-frame geometry/effect work can be expensive: enforce channel, geometry, expanded-layer, raster, and effect budgets before allocation/publication; retain deterministic cache keys and fail closed.
- Crop and rotation may shift anchors: fixtures use asymmetric sources, noncentral anchors, skew, parent transforms, and component clocks.
- Effect/color handling can diverge across codecs: evaluate in linear premultiplied color and use the established SSIM/PCM tolerance for encoded outputs, with exact semantic sample assertions.
- Compound overshoot can make invalid gradients/crops: clamp individual components to bounds, then reject invalid coupled geometry or stop ordering without repair.
- Scope deliberately leaves particle and group effects unavailable: catalog and diagnostics must accurately report this subset.

## Migration Plan

Write canonical fixtures and source-snapshot tests first; add schema-27 migration/defaults and validators; update every governed consumer; implement shared sampling/rendering; validate legacy and new outputs and all required suites; verify, synchronize, and archive only after approval and successful checks. Rollback before publication preserves the old generation; after publication restore a verified backup with the matching binary. Never downgrade schema 27 or weaken unknown-future rejection.

## Approval and tooling

The original plan and bounded candidate certification amendment were explicitly approved on 2026-09-30 (user: Approve / Aprove). Implementation proceeds through tasks.md. Pinned Moon 2.3.3 is available through bunx; the pre-archive protected run rejected only this active change, as expected. No new external library is introduced.
