# Shape items

Schema 14 activates seven vector shapes. Protocol 1 advertises `shape_items` for editing and `shape_rendering` when the complete renderer is ready. The reference-free vector catalog keeps its `core_primitives_only` status; activation is owned by `contracts/shape-items-v1.json`.

Use MCP `timeline_add_shape` or headless `edit` with `edit.operation: "add_shape"`. Supply `projectId`, `expectedRevision`, `trackId`, `startMs`, `durationMs`, `geometry`, and explicit nullable `fill` and `stroke`. Shapes require an unlocked overlay track. Optional `transform2d` defaults to identity; optional `parent` uses existing composition scopes. New shapes are visible with zero z-index. Timing uses safe integer milliseconds and half-open intervals.

```json
{
  "projectId": "project-id",
  "expectedRevision": 0,
  "trackId": "overlay-id",
  "startMs": 0,
  "durationMs": 1000,
  "geometry": {"type": "ellipse", "width": 120, "height": 60},
  "fill": {"type": "solid", "color": {"r": 1, "g": 0, "b": 0, "a": 1}},
  "stroke": null
}
```

Geometry uses closed objects and local pixels, with positive X right and Y down:

| Type | Required geometry fields |
| --- | --- |
| `rectangle` | `width`, `height` |
| `roundedRectangle` | `width`, `height`, `radii` with all four named corners |
| `ellipse` | `width`, `height` |
| `line` | distinct `start` and `end` points |
| `polygon` | 3–4096 `points`, implicitly closed with nonzero fill |
| `star` | `center`, `outerRadius`, `innerRadius`, `pointCount`, `rotationDeg` |
| `path` | structured `path` containing `fillRule` and `commands` |

Dimensions and star radii are in (0,16384]. Stars require inner radius below outer radius, 3–2048 tips, and rotation in [-36000,36000] degrees. The first outer tip points up at zero rotation; vertices proceed clockwise, alternating radii. Rectangle/ellipse origins are (0,0). Corner radii retain the existing zero-inclusive vector contract and resolve with a common proportional scale.

See [vector primitives](vector-primitives.md) for colors, gradient interpolation, stops, points, stroke widths, dash phase, caps, joins, corner radii and path grammar. At least one paint must be present; lines require stroke and null fill. Fill draws before stroke. Anchors use analytic unstroked bounds, with a one-pixel minimum for degenerate dimensions. Stroke padding does not move the anchor. Offscreen geometry clips without relocation.

`timeline_update_item` accepts complete `geometry` replacement and nullable `fill`/`stroke` patches; omitted fields retain their values. Existing move, trim, split, duplicate, visibility, stacking, parenting and history operations apply. Legacy transform updates switch to legacy mode; legacy position/scale/opacity animation retains its existing incompatibility with active Transform2D. Shapes cannot carry audio or be transition endpoints. Component create/update accepts local shapes; existing instance clocks, parent scopes and ordering apply. Shape-specific slot bindings and animation channels are not introduced.

Within `timeline_batch_edit`, `add_shape` accepts `resultAlias`; later operations reference `@alias`. Earlier track/group aliases resolve in `trackId` and `parent.id`. The ordered batch commits once or rolls back entirely. Missing tracks/items/parents retain `TRACK_NOT_FOUND`/`ITEM_NOT_FOUND`, locked targets return `TRACK_LOCKED`, stale revisions return retryable `REVISION_CONFLICT`, and invalid shape semantics return non-retryable `INVALID_ARGUMENT`. Malformed structures fail typed decoding.

Core bounds authored paths at 4096 commands, compiled geometry at 8192 commands, flattened geometry at 65536 segments per shape and 1048576 per scene sample, and subdivision depth at 16. Curves target at most 0.25 output-pixel deviation. Existing transformed-surface and scene limits still apply. Excess work fails before rasterization and publication. No SVG, raw expressions, arbitrary paths or network resources are accepted. tiny-skia 0.11.4 computes coverage from core-compiled geometry; core explicitly evaluates linear-light premultiplied paints.

Frame, audiovisual range, materialized draft and final export share evaluated geometry and rendering. Equivalent output has equal normalized semantic plans, visual SSIM at least 0.99, aligned decoded PCM RMS error at most 0.0001 and timing within one output frame. Missing complete backend support returns `DEPENDENCY_UNAVAILABLE`; no degraded fallback is published.

Opening supported schemas 1–13 migrates current state and all retained undo/redo snapshots to schema 14 under the project lock in one recoverable generation. The 13-to-14 step changes only the version. Older schemas containing shapes are rejected before relabeling; invalid or future current/history state is never partially published. Older binaries reject schema 14. There is no automatic downgrade, and existing rectangle operations and legacy color strings retain their behavior.

## Raster density and strict decoding

Shapes rasterize at a density of at least one raster pixel per local pixel and at least the maximum effective magnification over the render interval. Density includes composed parent/component transforms and animated scale. Core compensates source sampling so local geometry, gradient coordinates, stroke metrics and placement are unchanged. Density-adjusted surfaces retain the existing 16,384-pixel dimension and 16,777,216-pixel area limits; excessive work fails before allocation instead of lowering quality. Aggregate accounting derives from those surface limits and the existing 4,096 visual-layer limit, with three RGBA raster buffers per shape.

Anchors use positive geometric extents exactly, even below one pixel. Only a zero-extent axis of path geometry uses the one-pixel anchor fallback. Horizontal and vertical line axes remain zero, and stroke padding does not change anchors.

Raw shape-bearing JSON retains duplicate object entries through migration preprocessing and typed decoding. Duplicate vector fields fail in edits, batches, drafts, components, current project data and retained history; failures do not mutate state. An already-parsed JSON object cannot recover duplicate keys discarded by its caller. Valid serialized forms, schema 14, missing/null semantics and stable errors remain unchanged.
