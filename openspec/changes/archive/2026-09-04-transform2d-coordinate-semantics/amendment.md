## Proposed correction: measure text before resolving affine bounds

Status: approved by the user with a second "Approve" message on 2026-09-04. This correction supplements the originally approved plan.

## Evidence and reason

The original design assumed source dimensions were available during pure scene evaluation. In `crates/editor-core/src/render_artifact.rs`, `prepare_evaluated_text_layers` resolves a font, wraps text, writes a text file, and only then calls `measure_text_block` to derive its raster dimensions. Those dimensions depend on the actual font bytes. In `crates/editor-core/src/render_plan.rs`, captions use `drawtext` directly on the composition; there is no existing caption raster box. Consequently, the original requirement to resolve text/caption anchors and bounds before resource preparation cannot be implemented literally while keeping evaluation free of filesystem access.

## Proposed design replacement

For Transform2D scenes, use three ordered stages through the existing ownership boundaries:

1. Pure core preflight validates item references, values, timing, and collection limits and produces owned typed source/measurement requests.
2. Read-only resource preflight resolves fonts using existing path-safe selection and computes text layout metrics. It returns a typed measurement table keyed by item ID, with only dimensions and layout facts crossing into evaluation. Font paths and bytes remain in the separate resource sidecar. Reuse the same selected font and measured layout when preparing rendering; do not independently resolve them again. No workspace, text file, raster buffer, FFmpeg process, or artifact is created in this stage.
3. Pure core finalization uses the typed measurements to resolve all affine matrices and enforce the approved transformed dimension/area limits. Only the complete, validated EvaluatedScene reaches resource writes, rasterization, graph lowering, and execution.

This uses existing `render_artifact -> evaluated_scene` and renderer orchestration dependencies. The evaluator does not call the artifact layer or read fonts, and no new dependency edge is required. The term "before output allocation or backend preparation" in the original delta is replaced with "after bounded read-only measurement, before raster/output allocation, resource writes, or backend execution" for text geometry. Collection limits remain checked before measurement.

For active Transform2D captions, define a new isolated source box using the shared text measurement policy: width = max(1, ceil(measured text width)) + 24, height = max(1, ceil(measured text height)) + 24, with text inset 12 pixels on each axis, existing font size/colors, and background alpha 0.75. The Transform2D anchor is relative to that complete box; its position replaces legacy bottom-center placement. Captions without Transform2D retain their existing direct-render behavior exactly. This box is a new contract, not an assertion that the legacy caption renderer already has such a box.

## Verification additions

- With the same text and two configured fonts of different metrics, assert the measured source box and finalized affine anchor follow the selected font and that preview/export use the identical measurement.
- Inject a geometry overflow discovered after measurement and assert zero workspace creation, text writes, raster allocation, process calls, and artifact publication.
- Verify pure evaluation/finalization receives no filesystem paths or font bytes and add structural architecture coverage for the staged API.
- Test caption source dimensions, 12-pixel inset, background alpha, noncentral anchor, explicit position, and unchanged legacy caption placement.
- Preserve missing-asset precedence over measurement and geometry failures, and all approved migration, mutation, bounds, and parity requirements.

## Approval recorded

The user approved this measurement-stage and caption-box correction. This is required by AGENTS.md's instruction to update inconsistent artifacts and obtain explicit approval, and by the openspec-apply-change skill's instruction to pause when implementation reveals a design issue.
