## Context

The archived add-secure-svg-ingestion change introduced schema-15 SVG documents. Review reproduced: 500 overlapping #ff000001 rectangles produce preview red 127 instead of approximately 219 over black; a 100x100 viewport with a 10000x10000 viewBox and matching rectangle imports but fails raster bounds; and M10 10 L20 10 L20 20 Z H40 V40 fails with drawing requires moveTo. These are corrections to existing SVG requirements, not additional SVG syntax categories.

## Goals / Non-Goals

Goals: preserve accumulated opacity, enforce limits on actual mapped surfaces, normalize legal supported path continuation, and prove these corrections across immutable render intents and lifecycle operations.

Non-goals: new public types, schema changes, dependencies, SVG commands or resources, warning exceptions, relaxed budgets, changed standalone shape rendering or regenerated unrelated golden references.

## Decisions

### Floating-point document composition

Use one viewport-sized Vec<[f64; 4]> of premultiplied linear-light color. Obtain child fill/stroke coverage using reusable tiny-skia masks; combine fill then stroke and then siblings directly into the accumulator. Encode straight-alpha sRGB PAM once at the end. Keep item/ancestor opacity in the existing layer pipeline, applied once. Do not route SVG children through the current byte-encoded standalone raster result: that retains the repeated quantization defect, including fill/stroke interaction. Keep standalone shape output unchanged, sharing coverage helpers only when byte-equivalence is established.

Conservatively preflight 44 bytes per pixel: accumulator 32, two coverage masks 8, encoded output 4, plus checked header arithmetic. Avoid retaining per-child surfaces. Keep the existing aggregate ceiling, currently 16777216 * 12 * MAX_EVALUATED_VISUAL_LAYERS, unchanged; use checked arithmetic and fallible large allocations. Memory accounting belongs in scene preflight; artifact preparation consumes the checked evaluated surface.

### Compile geometry separately from surface allocation

Extract bounded contour compilation from standalone surface sizing. Preserve standalone geometry, padding, precision checks and output exactly. For SVG, validate the normalized source, compute density from composed transforms (at least one viewport sample per unit), and flatten source curves with tolerance derived from density times viewport mapping. Map contours, offsets and stroke metrics into viewport space before checking their raster representation. Dash splitting and contour work remain bounded even when offscreen.

Allocate only the viewport surface: origin zero and dimensions ceil(viewport * density), with no dummy-rectangle padding. Check dimensions <=16384 and pixel area <=16777216, inclusive, before casting or allocating. Do not run source-space standalone surface checks on children whose only surface is the clipped viewport. Merely increasing global limits or removing all child validation would hide the error and weaken safety.

Validate finite mapping intermediates and representability of mapped raster coordinates, stroke widths and dash metrics. Reject positive stroke/dash lengths that become zero or non-finite in the raster representation. Keep source-coordinate bounds, subdivision depth and per-shape work limits. Thread the remaining scene-segment budget through SVG compilation, including preflight and refinement, checking before each expansion. Hidden content and component expansion use the same safeguards.

### Normalize closepath continuation in the source parser

Track the initial point of the current subpath and whether it is closed. Z resets the current point to that initial point. Before a following L/H/V/Q/C, emit a MoveTo to the saved initial point and then the lowered drawing command. Explicit M starts its own subpath without an inserted move. Preserve repeated argument-set handling; repeated Z and unsupported commands retain existing rejection behavior. The existing structured VectorPath validator remains unchanged.

Account for synthetic MoveTo commands in the 4096-command path budget and 65536-command document budget before insertion. Pass remaining document capacity into path/primitive normalization so lowering cannot grow beyond that capacity before rejecting. Retain the stricter existing normalized grammar; accepting raw SVG grammar in the persisted validator would unnecessarily expand the contract.

### Fixtures, ownership and compatibility

Update the canonical SVG catalog manually with small post-close and downscaled fixtures and counterexamples; large opacity/budget fixtures are generated deterministically in tests. Consume them in core, headless and bridge parity/MCP workflows. Wire records remain schema 15/document 1/protocol 1. Existing SVG items render correctly without rewriting them; new path continuations persist ordinary MoveTo/LineTo/curve/Close records. Obtain designated contract-owner review for changed canonical fixtures and consumers, without equating passing tests with review.

Use existing inward edges: validation normalizes, evaluated_scene owns mapping and budgets, render_artifact consumes evaluated contours. No transport semantic validator or resource access is added. Errors retain existing non-retryable INVALID_ARGUMENT handling, category-only diagnostics and atomic rollback; stale revisions still precede semantic parsing.

## Risks / Trade-offs

- Floating-point buffers increase peak memory -> account for 44 bytes/pixel before allocation and reuse masks; do not raise the aggregate ceiling.
- Shared compiler refactoring affects standalone shapes -> retain exact geometry/debug plans and run the complete legacy native suite without updating its references.
- Downscale handling can silently underflow strokes or over-expand curves -> test mapped precision, magnification and remaining-work limits explicitly.
- Newly inserted commands exceed source-command expectations -> normalized rather than textual command counts govern inclusive limits and are documented and tested.
- Native intent agreement alone can preserve a shared bug -> retain independent analytic alpha and geometric/pixel oracles.

## Verification plan

Map each delta scenario to named tests in the verification report. Add regression tests before implementation: analytical opacity (including mixed colors, fill/stroke and order); mapped viewport bounds/precision and clipping; post-close L/H/V/Q/C and normalized budgets. Native tests include all fixes plus item opacity, transformed/retimed components, synthetic audio and frame/range/draft/export before and after undo/redo/reopen. Preserve plan equality, SSIM >=0.99, PCM RMS <=0.0001 and one-frame timing tolerance. Run all repository-required gates listed in tasks.md.

## Migration Plan

Create and approve these artifacts, implement tracked tasks, verify, synchronize and archive, then run the protected Moon gate. No migration or new version is required. Reverting this follow-up restores the defective rendering but leaves existing persisted records readable by schema-15 binaries; do not rewrite projects or history as a rollback mechanism.

## Open Questions

No technical scope decision is outstanding. The user explicitly approved these artifacts on 2026-09-07.
