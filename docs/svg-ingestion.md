# Secure SVG ingestion

Protocol 1 exposes `timeline_add_svg` through MCP and `add_svg` through the existing headless `edit` and `edit_batch` envelopes. `svg_items` advertises editing; `svg_rendering` is present only when the complete renderer is ready.

```json
{"projectId":"project","expectedRevision":0,"trackId":"overlay","startMs":0,"durationMs":1000,"svg":"<svg width=\"40\" height=\"20\"><rect width=\"40\" height=\"20\" fill=\"#f00\"/></svg>"}
```

The item uses an unlocked overlay track. Optional `transform2d` defaults to identity and `parent` follows existing scope rules. New items are visible with zero z-index. Timing uses safe integer milliseconds and half-open intervals. Existing positioning, scaling, opacity, parenting, stacking, trimming, splitting, duplication, deletion and undo/redo apply. Legacy transform/keyframe exclusivity is unchanged. Source or shape-paint replacement, audio edits and transition endpoints are unsupported.

In `timeline_batch_edit`, use `operation: "add_svg"` with `resultAlias: "icon"`; subsequent edits can use `itemId: "@icon"`. Earlier track and parent aliases resolve as usual. A batch commits once or rolls back completely. Stale revisions fail with retryable `REVISION_CONFLICT` before SVG parsing; missing tracks/items/parents and locked tracks retain their existing errors. Invalid SVG returns non-retryable `INVALID_ARGUMENT` with a category rather than a source excerpt.

## Accepted subset

Use one unqualified `svg` root, optionally declaring the default SVG namespace. Supported descendants are `g`, `rect`, `circle`, `ellipse`, `line`, `polygon`, `polyline`, and `path`. Groups inherit presentation attributes. Width/height are required positive finite lengths, unitless or `px`, at most 16384. Geometry uses the existing vector coordinate/stroke limits. Coordinates start at the top left, with X right and Y down. `viewBox` defaults to the viewport; four supplied finite values require positive extents. Only centered `xMidYMid meet` is supported. Artwork is clipped to the viewport, with transparent margins, and painted in document order.

Supported presentations are `fill`, `fill-rule`, `stroke`, `stroke-width`, `stroke-linecap`, `stroke-linejoin`, `stroke-miterlimit`, `stroke-dasharray`, and `stroke-dashoffset`. Colors are `none` or hexadecimal RGB/RGBA in short or long form. Defaults are black fill, no stroke, nonzero winding, width 1, butt cap, miter join, miter limit 4, empty dash and zero offset. Fill precedes stroke. Lines ignore inherited fill and require stroke. Rectangle rx/ry must agree (one omitted value copies the other), with radius clamped to half the smaller dimension.

Paths accept absolute `M L H V Q C Z`, including repeated argument sets, and normalize into typed vector commands. There are no relative commands or arcs. XML declarations, comments and whitespace are inert. Character references are decoded before value validation. Unsupported content is rejected in its entirety; nothing is silently stripped or downloaded.

Scripts, event handlers, href references, URLs (including file/data), DTDs/entities, other processing instructions, text/fonts, CSS/style, embedded images, gradients, patterns, filters, masks, clipping paths, use, nested SVG, SVG transform attributes and animation are rejected. Use the timeline item's Transform2D for transforms. No font fallback or system resource lookup occurs.

## Limits and persistence

The source limit is 1,048,576 UTF-8 bytes, depth 32 including the root, 4,096 elements including groups/root, and 32 attributes per element. Paths have at most 4,096 normalized commands; a document has at most 65,536 commands after primitive lowering. Existing curve subdivision, flattened segment, scene expansion, surface dimensions, pixel-area and aggregate memory limits apply before raster work. Excess work fails instead of reducing quality.

Schema 15 stores a version-1 normalized document with width, height, viewBox and ordered shapes containing geometry, local offset, fill and stroke. It stores no XML, source paths or resource references and creates no asset files. All persisted records, hidden component content, drafts and history validate in core. Supported schemas 1–14 migrate current state and retained undo/redo atomically under the project lock. Existing content is unchanged; older binaries reject schema 15. Restore a pre-migration backup to downgrade.

Frame, audiovisual range, draft preview and export use the same EvaluatedScene and bounded shape raster path. The SVG composes internally before item opacity is applied. Equivalent semantic plans are exact; visual SSIM is at least 0.99, aligned decoded PCM RMS error at most 0.0001, and timing within one output frame. The canonical catalog is `contracts/svg-ingestion-v1.json`; the native suite uses independent integer-coordinate visual oracles with synthetic audio and lifecycle checks.

### Corrected SVG normalization and rasterization

After `Z`, supported `L/H/V/Q/C` commands continue from the closed subpath's initial point. Normalization inserts a `MoveTo` before that continuation; the inserted command counts toward both path and document limits. A following explicit `M` does not add an extra move.

Raster bounds apply to the mapped viewport at the composed sampling density, not an unused child surface in source coordinates. Downscaled large viewBoxes remain renderable within the unchanged 16,384-pixel dimension and 16,777,216-pixel area limits. Mapped stroke/dash precision and all geometry/scene work limits still fail closed, including hidden and unused SVG content.

SVG fills, strokes and ordered siblings accumulate in premultiplied linear-light f64 RGBA and are encoded once. Item and inherited opacity apply once after internal composition. Resource preflight accounts for 44 bytes per viewport pixel with reusable coverage buffers and preserves the existing aggregate memory ceiling. These corrections require no schema or protocol upgrade; existing normalized SVG records benefit on their next render.

Raster representability also includes the backend's outward-rounded integer bounds and conservative stroke envelopes, including caps and miters. Finite floating-point values alone are insufficient: extreme viewport mappings that cannot be represented fail with non-retryable `INVALID_ARGUMENT` before artifact preparation. Dash conversion cannot silently fall back to an undashed stroke. Representable offscreen geometry and degenerate fills remain valid empty coverage.

Component raster limits use the complete occurrence transform. For example, a 100x100 SVG at local scale 100 in an instance at scale 0.01 renders at effective scale 1. Hidden occurrences receive the same checks. Definitions reachable through project instances are not additionally sized in isolation; unreachable definitions are checked as identity-outer virtual roots, including nested instances. Existing graph, work and memory limits remain enforced during traversal.

Each SVG point is checked before conversion to the rasterizer's f32 coordinates. The Euclidean displacement between its f64 raster-space position and the f32 round-trip position must be at most 0.25 raster pixels, inclusive. The check runs after viewport mapping and sampling density and includes offscreen and move-only contours. Excessive or non-finite conversion error returns non-retryable `INVALID_ARGUMENT` before artifact preparation. Exactly representable large coordinates remain accepted within existing backend bounds. This conversion threshold is separate from the unchanged curve-flattening tolerance; it is not a combined 0.25-pixel error guarantee.
