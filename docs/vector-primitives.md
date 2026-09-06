# Vector primitives (version 1)

Issue #27 provides reusable production types and pure validation in editor-core. The canonical [catalog](../contracts/vector-primitives-v1.json) has activation status `core_primitives_only`. ShapeItem rendering and `timeline_add_shape` are issue #28. No new headless/MCP operation, capability, project field, or renderer instruction is activated here.

## Wire vocabulary

All objects are closed: every listed field is required, unknown fields and tags are rejected, and numbers must be finite without string coercion. Rust callers must call `validate()` on decoded or directly constructed values before use. Structural Serde decoding errors and semantic core errors are distinct; semantic rejection uses existing non-retryable `INVALID_ARGUMENT`. Validation performs no I/O or mutation.

Records accept JSON objects only, including nested colors, points, stops, strokes, radii, and paths. Positional arrays such as `[1,0,0,1]` are not colors. Paints and path commands also require objects with their existing `type` tag; sequences such as `["solid",{"r":1,"g":0,"b":0,"a":1}]` are invalid. Caps, joins, and fill rules accept exact JSON strings only; objects such as `{"butt":null}` are invalid. Both raw-string and parsed-value Rust decoders enforce these shapes. Streaming record decoding also rejects duplicate fields in raw JSON without first normalizing them through a JSON value. TypeScript schemas validate parsed objects; duplicate-key evidence therefore belongs to the raw Rust decoder tests.

| Primitive | Fields / variants |
| --- | --- |
| Color | `{r,g,b,a}`, each in [0,1] |
| Point | `{x,y}`, each in [-1000000,1000000] local pixels |
| Paint | `{type:"solid",color}`; `{type:"linearGradient",start,end,stops}`; `{type:"radialGradient",center,radius,stops}` |
| Gradient stop | `{offset,color}`; 2–64 stops, strictly increasing from exactly 0 to exactly 1 |
| Stroke | `{paint,width,dash,dashOffset,lineCap,lineJoin,miterLimit}` |
| Corner radii | `{topLeft,topRight,bottomRight,bottomLeft}`; each in [0,16384] |
| Path | `{fillRule,commands}`; fill rule `nonzero` or `evenodd`, 1–4096 commands |
| Commands | `{type:"moveTo",to}`, `{type:"lineTo",to}`, `{type:"quadraticTo",control,to}`, `{type:"cubicTo",control1,control2,to}`, `{type:"close"}` |

Example linear paint:

```json
{
  "type": "linearGradient",
  "start": {"x": 0, "y": 0},
  "end": {"x": 100, "y": 0},
  "stops": [
    {"offset": 0, "color": {"r": 1, "g": 0, "b": 0, "a": 1}},
    {"offset": 1, "color": {"r": 0, "g": 0, "b": 1, "a": 0}}
  ]
}
```

## Geometry and color semantics

Points use a top-left origin with positive X right and Y down. All path commands are absolute. Linear gradient endpoints must differ; radial gradients are circular, centered on their focal point, with radius in (0,1000000]. Gradients pad beyond their endpoints. RGB values represent unassociated sRGB and alpha is linear. Interpolation converts RGB to linear light, premultiplies by alpha, interpolates, and uses zero RGB if unpremultiplying a fully transparent result. Raster sampling is implemented by the later renderer activation; these semantics do not imply a rasterizer exists now. Stops are never sorted, clamped, or repaired.

Strokes are centered. Width is in (0,16384]. Caps are `butt|round|square`; joins are `miter|round|bevel`. Miter limit is in [1,1000], measured as miter length divided by half-width, with bevel fallback when exceeded. A dash array is empty (solid) or has an even number of at most 64 entries, each in (0,1000000] local pixels. Odd arrays are rejected. Dash offset is in [-1000000,1000000]; positive values advance into the pattern and phase restarts per subpath.

For a rectangle of width/height in (0,16384], `CornerRadii::resolve` scales all four corners by one common factor: the minimum of 1, width/top sum, width/bottom sum, height/left sum, and height/right sum. Zero denominators are ignored. The original radii are unchanged. For radii (80,20,60,40), width 100 and height 60, the factor is 0.5 and the result is (40,10,30,20).

Every subpath starts with moveTo. A move-only path is legal empty geometry. Drawing before moveTo, close without a drawing segment, repeated close, and drawing after close without a new moveTo are rejected. Close connects to the subpath start. Filling implicitly closes open drawable subpaths; stroking only closes explicit close commands. Multiple subpaths are legal, and command order is preserved. There is no SVG/path-string parser, arc or relative command, resource reference, tessellation, or renderer expression surface.

## Compatibility and verification

Legacy color strings, schema 13, current state and retained history, optimistic revisions, atomic batches, undo/redo, reopening, and existing EvaluatedScene/render output remain unchanged. Therefore no migration or new missing-reference/alias behavior applies. Future activation must add typed consumers, capability evidence, deterministic history migration for persisted additions, and preview/export parity through the shared scene; unsupported rendering must follow ADR 0004's fail-closed policy.

Rust tests consume the catalog using production types; TypeScript tests use reusable strict Zod schemas in `src/vector-primitives.ts`. Both test canonical acceptance, malformed data, limits, and immutable round trips. Rust supplements JSON with directly constructed non-finite values and an independent radius-resolution oracle. The standalone bridge `bun run contracts:check` runs both vector suites.

Fixture entries marked `structuralInvalid: true` must fail decoding before semantic validation. The catalog includes standalone gradient stops, path commands, and string enums as well as their nested uses. Catalog and fixture envelopes themselves are object-only, and fixture kinds are strings. Raw Rust tests cover reordered object keys and duplicate fields at every nested record site.
