## Context

A white width-1 rectangular grid with viewport 10.5 by 10 and spacing 10 by 20 produces RGB 64 at (10,5), versus 128 when viewport width is 11. The same half-pixel stroke area is visible in both; multiplying the already antialiased coverage by 0.5 clips twice. The MCP test uses nonexistent remove_item and therefore never reaches core batch execution.

## Decisions

Keep clipping in evaluated_scene/shapes/grids. Resolve stroke coverage before clipping: grid strokes are straight lines, so use the existing tiny-skia dash resolution and stroke outline conventions, then flatten outline curves with the shared compiler tolerance 0.25/density. Dots use existing evaluated fill contours. Clip closed coverage polygons against x=0,width and y=0,height using Sutherland-Hodgman, preserving contour orientation and mark order. Store only resolved fill coverage for clipped marks and the original local paint; do not reconstruct persisted semantics in render_artifact. Preserve original drawing facts where helpful to existing geometry oracle tests, with bounded process-local coverage facts as necessary.

Account for source compilation, stroke/dash outline generation, flattening and clipped output within the existing 65536 per-grid and 1048576 per-scene segment budgets. Check finite values and output capacity before pushing derived vertices, reject excess as INVALID_ARGUMENT during preflight, and preserve memory/surface bounds. No tolerance reduction or approximate raster mask fallback. Shapes/SVG retain their current code paths. The final grid coverage multiplier is removed.

Change the MCP trailing operation to delete_item for a missing ID. Assert structured error code/retryability for ITEM_NOT_FOUND and REVISION_CONFLICT, and equal project content/revision before and after each failure. Reuse the workflow in both integration and packaged suites.

## Verification

Start with failing right/bottom/corner edge regressions; extend coverage to dots, diagonal/isometric lines, dashes/caps, alpha/gradients and density. Verify clipped contour bounds and exact budget acceptance/rejection. Include a fractional grid in native frame/range/export comparisons and root/draft equivalence with existing >=0.99 SSIM, <=0.0001 PCM RMS and one-frame timing tolerance. Run all repository-required checks, verify the change, sync/archive and rerun Moon.
