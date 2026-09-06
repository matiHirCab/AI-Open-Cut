## Context

In evaluated_scene/shapes.rs, affine replaces the supplied output canvas with the instance's local canvas. That local canvas is appropriate for normalized position resolution, but later root-space animation bounds are clamped against it. Review reproduced a visible red rectangle becoming entirely black after adding constant scale keyframes. Existing tests covered component transforms and animation on separate shapes, missing their combination.

## Goals / Non-Goals

Goals: preserve animated shape visibility and placement under composed component transforms; retain local normalized-coordinate resolution, density, timing, opacity and pre-clipping geometry validation; prove consistent frame/range/export behavior with independent oracles.

Non-goals: changes to the non-shape evaluator, public APIs, persisted schema, component masking, animation interpolation, rasterization, density selection, vector decoding or golden-reference policy. Previous fixes and archives remain intact.

## Decisions

1. Rename the shape affine input to output_canvas and derive a separate local_canvas from layer.instance.canvas or output_canvas for root layers. Use local_canvas only for local transform resolution. Use output_canvas for the animation envelope's viewport intersection. This directly corrects coordinate ownership without introducing another clipping pass or changing component semantics.
2. Pass output_canvas unchanged to recursive affine samples. Each sample resolves its local coordinate dimensions from the occurrence as before, composes its ancestors and density compensation, and returns root-space bounds. Keep affine_from_matrices validation before viewport intersection; clipping must never conceal oversized or non-finite geometry. Preserve zero-size sampling results for entirely invisible animated occurrences so finalization can omit them.
3. Keep the shared EvaluatedScene pipeline: no renderer-specific workaround, FFmpeg expression rewrite, or headless validation. Refine/finalize must produce the same corrected bounds repeatedly and use the requested output dimensions rather than persisted project settings.
4. Add a dedicated component-animation fixture builder in the existing core shape test module, independent of the stored golden recipe. Run its native assertions through the required golden conformance entrypoint. Do not recapture or weaken existing references. Supplement cross-intent comparisons with independently expected pixels/bounds so shared evaluator bugs cannot make every intent agree incorrectly.

## Verification and traceability

All cases map to the delta's Canonical shape evaluation and bounded raster work requirement:

- Constant animation: 240x120 output, 40x40 component, red 10x10 shape at local (0,0), instance translation (100,20). Compare the static shape against scale keys of 1 at 0/1000ms. At 500ms decoded images must match exactly, the pixel (105,25) must be red, and background pixels must remain black. Assert the evaluated sampling bounds contain the transformed rectangle.
- Animated, nested and retimed instances: animate local position (0,0) to (10,0) and scale 1 to 2 over 1000ms using linear interpolation. Cover direct instances and two nested levels, with explicit instance translations/scales, trim offsets and timeScale=2. Compute expected local time and transformed rectangle corners arithmetically in the test, independently of the production evaluator. Assert bounds enclose those corners and native interior/exterior pixels match at multiple valid root timestamps.
- Output clipping: use translations near both viewport edges and entirely beyond the output. Assert the visible part is preserved without moving its origin; fully offscreen animated occurrences have no visible pixels. Include a component larger than the output and one smaller than the output.
- Coordinate dimensions: a static shape with normalized position (0.5,0.5) in a 40x40 component resolves locally to (20,20), independent of requested output size. Pair this with an animated shape in that component and compare evaluation/range/export at an output size different from project settings. For native frame parity at that size, use a clone whose project settings match the requested size; normalized component coordinates must remain local.
- Validation/compatibility: retain the existing finite geometry, density surface and no-artifact-failure tests; include an oversized offscreen transformed shape to ensure clipping does not hide failure. Existing state/history, stale revision, root shapes, density, fractional anchor and raw duplicate regressions stay green.

Native previews/ranges/exports use the same FFmpeg toolchain and existing SSIM >= 0.99 cross-codec threshold. Constant-keyframe same-intent images use exact equality. Exact affine assertions use the established floating-point tolerance, and pixel probes stay safely inside/outside edges to avoid antialias ambiguity. For the new H.264 interior probes, red must exceed 220 and green/blue remain below 35 (the observed small-rectangle encoded center is [239,0,19]); preview-only reproduction probes retain red > 240 and green < 10. These new probe thresholds do not change existing golden references, exact-equality checks or SSIM tolerances. Retain project/history byte equality across read-only rendering.

## Risks / Trade-offs

- Confusing local normalized units with viewport units could move static shapes: explicit unequal-canvas and normalized-coordinate oracles cover this.
- Fixing only the outer call could leave nested samples incorrect: keep output_canvas across every recursive call and cover nested/retimed occurrences.
- Correct clipping could accidentally bypass geometry limits: validate composed geometry before clipping and retain side-effect-free failure checks.
- Cross-intent agreement alone could hide a shared defect: include independent bounds and red/background pixel probes.

## Migration Plan

No data migration or publication change. Obtain approval of these concrete artifacts, add failing regressions, implement through tasks, run required checks, verify with openspec-verify-change, synchronize and archive with openspec-archive-change, then run the post-archive Moon gate. The active-change CLI validator runs before archival; protected Moon policy runs after archival because it rejects active changes. Rollback would revert this follow-up only; it does not rewrite saved projects or prior fixes.

## Open Questions

No implementation decisions remain open. The user approved the concrete artifacts on 2026-09-06.
