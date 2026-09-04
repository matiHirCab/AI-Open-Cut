## Context

Issue #18 follows common visual-property ownership from #17, present in the checkout at schema 7. ADRs 0002–0004 and the existing flat EvaluatedScene remain the governing boundaries. The current Transform contains positionX, positionY, scale, and opacity. Transform fixtures use richer lower-camel-case fields but are not runtime contracts.

## Goals / Non-Goals

Deliver a typed static affine transform for media visuals, text, solids, rectangles, and captions, through existing update operations and shared scene evaluation. Preserve legacy output and all transaction guarantees. Non-goals are listed in proposal.md; transition records and audio-only items cannot receive Transform2D because they have no independent visual source.

## Decisions

1. Preserve legacy Transform and add optional transform2d to common properties. A complete Transform2D contains position {x,y,unit}, anchor {x,y}, scaleX, scaleY, rotationDeg, skewXDeg, skewYDeg, opacity. Runtime values do not carry fixture definition id/scope. Absence or null selects legacy behavior; a legacy transform update clears transform2d; a transform2d update retains dormant legacy values. Supplying both in one update is invalid. Partial Transform2D objects are invalid. This avoids replacing a strict public legacy shape or multiplying two transforms unexpectedly. Extend update_item and its existing MCP/batch representation rather than introducing redundant operations; new content can be created and transformed through aliases in one batch.

2. Core owns validation and affine math. Coordinates are top-left, X right, Y down. Resolve normalized position against canvas width/height. Anchor uses fractions of the post-crop source box, before scaling; text uses its measured raster box and captions use the isolated measured box specified in amendment.md, solids/rectangles their declared box, media its existing fitted/cropped box. Compose column vectors with M = T(position) R(rotation) Ky(skewY) Kx(skewX) S(scaleX,scaleY) T(-anchorX*width,-anchorY*height). Positive rotation is clockwise in this coordinate system. Skews use tangent of degrees. Independent shears are explicitly X then Y, not a simultaneous shear with an ambiguous singular determinant.

3. All numbers must be finite. Anchor is in [0,1]; scale is (0,100]; opacity is [0,1]; rotation is [-36000,36000] degrees; each skew is [-80,80] degrees. New pixel position magnitude is at most 1,000,000 and normalized magnitude at most 100. Legacy position validation is unchanged. Derived coordinates and matrix entries must be finite; transformed bounds must have each dimension <= 16384 and area <= 16,777,216 pixels before allocation, including the un-clipped bounds. Existing scene collection limits remain. Clip against the composition only after canonical bounds checks. Invalid or excessive work is INVALID_ARGUMENT, never silent clamping. Legacy animation of position, scale, or opacity with active transform2d is rejected on either mutation path; other existing channels retain their behavior.

4. EvaluatedScene carries an owned typed affine instruction and source-box facts, with no paths/backend strings. Renderer lowering uses one affine mapping of prepared RGBA sources with premultiplied-alpha interpolation, bounded transparent output, and the documented offset. Existing decode/raster and audio paths are reused. Preview, range, draft, and export lower the same instruction. Backend readiness must cover the full instruction set; unsupported affine execution is DEPENDENCY_UNAVAILABLE before rendering/artifact publication. Backend expression generation, if required, accepts only validated typed values, never client expression strings. No new dependency edge is intended; any necessary edge requires an approved ADR 0003/test update before code.

5. Keep motion-graphics-v1 fixture-only for other roadmap concepts and introduce a dedicated transform2d-v1 runtime catalog linked from contract ownership. Reuse vocabulary and explicitly document runtime bounds and source-box semantics. Update headless-protocol-v1, mcp-surface-v1, native consumers, persisted fixtures, and capability reporting together. Advertise transform2d only when its complete runtime/render support is ready. No new provider or error code is needed. Replacing the entire fixture catalog status would incorrectly activate deferred features.

## Migration Plan

Add one schema bump to 8. Chain older migrations to 7, then retain legacy transforms exactly and default transform2d to absent. Validate current state and every retained undo/redo snapshot before publishing through the existing locked crash-consistent generation transaction. Invalid retained states, version 0, and versions above 8 fail closed. Preserve assets, provenance, revisions, and timestamps. Test every persistence fault injection phase, reopen, undo, and redo. Rollback uses a compatible schema-8 reader or restores a complete pre-upgrade backup; never silently downgrade schema-8 projects.

## Risks / Trade-offs

- Affine bounds and source-box offsets can cause clipping drift: use asymmetric corner fixtures, noncentral anchors, both skews, rotated text/media, and independent coordinate oracles.
- Keeping two serialized fields creates ambiguity: exactly one is active, with explicit switching and round-trip tests.
- Static-only Transform2D restricts animation: reject incompatible combinations, preserve legacy channels, and defer new channels to their own change.
- Migration failures can mix generations: reuse existing transactions and assert unchanged authoritative state/assets on each failure.
- Public schema updates need review: require @matiHirCab's governed contract review and parity tests.

## Verification

Each scenario in the five delta specs maps to a named test or table-driven case in tasks.md. No automation exemption is proposed. Compare exact evaluated affine facts and asymmetric rendered fixtures across every intent, with SSIM >= 0.99, aligned float-PCM RMS error <= 0.0001, and timing <= one frame. Existing legacy fixtures must retain prior behavior. Record actual commands/results during implementation.

## Open Questions

The user approved the complete static-transform scope, explicit bounds, and compatibility rules on 2026-09-04. If implementation reveals a need to alter those decisions, update these artifacts and obtain approval before expanding scope.

Implementation preflight found that text dimensions require font I/O and captions have no existing isolated raster box. `amendment.md` contains the concrete proposed correction and additional verification scenarios. The user approved that correction; it supersedes the original source-box/preparation assumptions. Evaluation uses pure preflight, read-only measurement, then pure affine finalization before resource writes or raster allocation.
