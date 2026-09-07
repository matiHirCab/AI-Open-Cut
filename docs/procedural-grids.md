# Procedural grids

Schema 16 adds editable `grid` items. Protocol 1 advertises `grid_items` for editing and `grid_rendering` only when the complete renderer is ready. Use MCP `timeline_add_grid` or headless `edit` with `operation: "add_grid"` and the existing projectId/expectedRevision envelope.

Supply trackId, startMs, durationMs and grid. Optional transform2d defaults to identity; parent follows existing root/component scope rules. Grids start visible with zero zIndex on unlocked overlay tracks. Time uses safe integer milliseconds and half-open intervals. Grids share shape-compatible visual transforms, animation, splitting, duplication, ordering, parenting and history; audio/transition endpoints and grid-specific animation/slot properties are unsupported.

```json
{
  "operation": "add_grid",
  "trackId": "overlay-id",
  "startMs": 0,
  "durationMs": 1000,
  "grid": {
    "width": 320,
    "height": 180,
    "pattern": {
      "type": "dot",
      "spacingX": 20,
      "spacingY": 20,
      "radius": 2,
      "paint": {"type": "solid", "color": {"r": 1, "g": 1, "b": 1, "a": 0.5}}
    }
  }
}
```

`grid` is a closed object containing width, height and pattern. Pattern fields are all required:

| type | Other fields | Geometry |
| --- | --- | --- |
| rectangular | spacingX, spacingY, stroke | Vertical lines first, then horizontal lines |
| diagonal | spacing, stroke | Two perpendicular 45-degree families |
| dot | spacingX, spacingY, radius, paint | Circular dots in row-major order |
| isometric | spacing, stroke | Vertical and two 60-degree families |

Dimensions and spacing must be finite in (0,16384]. Dot radius is in (0,min(spacingX,spacingY)/2]. Paints/strokes use [vector primitives](vector-primitives.md), including linear/radial gradients, dashes, caps and joins. Coordinates start at the top-left of the local viewport; X increases right and Y down. The viewport defines anchor bounds and clips final mark coverage before visual transforms, with a transparent background. Boundary lattice positions are included; slanted lines touching only a corner are excluded. Rotation and placement use standard visual transforms.

Rectangular lines are x=k*spacingX and y=k*spacingY; dots have centers (i*spacingX,j*spacingY). Diagonal unit normals are (1/sqrt(2),1/sqrt(2)) and (1/sqrt(2),-1/sqrt(2)). Isometric normals are (1,0), (1/2,sqrt(3)/2), (1/2,-sqrt(3)/2). Each family draws n dot (x,y)=k*spacing in listed-family and ascending integer k order. Spacing is perpendicular distance. Clipped line endpoints sort by x then y; dash phase starts at the first endpoint. Gradients use grid-local coordinates across all marks, and translucent intersections accumulate source-over alpha in enumeration order.

At most 4096 marks are accepted, including hidden/unused content. Derived geometry is limited to 65536 flattened segments per grid and 1048576 per scene sample, with existing curve tolerance, expanded occurrence, visual-layer, transformed surface and aggregate memory limits. Raster dimensions are at most 16384 and area at most 16777216 pixels; scale-aware density never silently decreases to fit. Excessive or non-finite work returns non-retryable INVALID_ARGUMENT before render preparation.

`timeline_update_item` accepts a complete `grid` replacement for grid items. Omission preserves it; null fails. Replace the descriptor to change nested styles; shape geometry/fill/stroke patches do not target grids. In `timeline_batch_edit`, add_grid accepts resultAlias, earlier track/group aliases resolve in trackId/parent.id and later edits use `@alias`. Batches commit one revision/undo step or roll back completely. Missing tracks/items/parents retain TRACK_NOT_FOUND/ITEM_NOT_FOUND, locked targets TRACK_LOCKED, stale revisions retryable REVISION_CONFLICT, and invalid content INVALID_ARGUMENT. Alias failures retain existing codes.

Frame, range, draft and export consume the same evaluated facts. Equivalent settings yield matching semantic plans, SSIM >=0.99, aligned decoded PCM RMS <=0.0001 and timing within one output frame. Missing complete support returns DEPENDENCY_UNAVAILABLE; no degraded fallback is published. Grid inputs never resolve SVG, expressions, files or network resources.

Fractional viewport edges clip evaluated mark coverage geometrically before antialiasing. Dashes and stroke caps resolve before clipping, so an already partial edge pixel is not multiplied by the viewport fraction again. Stroke outlines, flattening and peak clipping vertices count toward the existing derived-work budgets; those limits can reject a descriptor that fits the mark-count limit before any output is prepared.

Opening supported schemas 1-15 migrates current state and all retained history to 16 atomically under the project lock. The 15-to-16 step changes only schemaVersion. Grids under older source versions are invalid, including hidden definitions; malformed or future current/history content is rejected without publication. IDs, revisions, provenance, media and legacy output remain unchanged. Older binaries reject schema 16; no automatic downgrade exists. Existing requests remain compatible, but grid-bearing project responses require clients that understand the new item variant. Canonical examples live in `contracts/procedural-grids-v1.json`.
