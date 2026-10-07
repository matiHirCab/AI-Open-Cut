# Mask authoring models

Project schema32 introduced ordered painted path masks; schema33 activated static/animated mask execution and schema34 introduced scoped track mattes. Current schema37 preserves those contracts and adds [controlled group composition](group-compositing.md). Existing stored32 models retain their fields and deliberately acquire authored mask appearance on adoption. Absent/empty stacks preserve previous output. See [mask rendering](mask-rendering.md) for the active algorithm, animation, readiness and certified limits.

`contracts/mask-models-v1.json` is the canonical cross-language contract. Each record requires `id`, `source`, `channel`, `operation`, `inverted`, `transform`, `featherPx` and `expansionPx`. The only source is `{type: "path", path, paint}` using the existing VectorPath, Paint and Transform2D contracts. Paint supports solid, linear and radial forms. Channel is `alpha` or `luma`; operation is `add`, `subtract`, `intersect` or `exclude`. The authoring authority preserves these choices; the linked rendering contract evaluates coverage. Layer references, positional shorthand, unknown fields and null mask arrays are rejected.

Mask IDs are unique within an item and contain 1–128 UTF-8 bytes. The maximum stack is 16 records per item, with 4,096 records per composition and 16,384 per project. Paths allow 4,096 commands each; aggregate command budgets are 65,536 per composition and 262,144 per project. Coordinates are bounded to ±1,000,000 and gradients allow 64 stops. Feather is finite and between 0 and 128 pixels; expansion is finite and between −128 and 128 pixels. Validation counts hidden items, unused component definitions and all stored stacks, with each component definition counted once.

Nonempty stacks are allowed on image/video, text, solid color, rectangle, shape, SVG and grid leaves. Other common visual carriers accept only an empty stack. Empty stacks are omitted on serialization. Existing `update_item`/`timeline_update_item` operations preserve masks when omitted, replace the complete ordered stack when supplied, and clear it with `[]`. Creation aliases, batches, undo/redo and draft operations use the same validation and atomicity rules. Invalid later operations publish no earlier mutations.

The `mask_models_v1` capability belongs to the editor subsystem and is available whenever the editor is ready, independently of FFmpeg/FFprobe readiness. It is absent from rendering capabilities. Headless protocol version 1 and draft envelope version 2 remain unchanged; current project schema37 retains schema34 scoped track mattes and schema33 rendering/channels, while32 remains the mask authoring source threshold. Source projects from schemas 1–31 must not contain a `masks` field, including an empty array or null. Migration validates the current project, undo/redo entries and retained drafts before atomic publication. Reopening valid current-version data does not rewrite it merely to normalize an omitted empty stack. Supported schema34 projects adopt the current version atomically with their retained generations.

Changes to this model must update the canonical catalog, governed consumers, ownership declarations, schema fixtures and contract parity checks together. MCP compatibility evidence removes only the enumerated mask property locations, the new mask definition and capability, and the two approved schema-version literals to reproduce the immediate predecessor digest; unrelated schema or tool changes must remain visible to drift detection.

A complete `timeline_update_item` input replaces one leaf's stack:

```json
{
  "projectId": "project-id",
  "expectedRevision": 1,
  "itemId": "leaf-id",
  "masks": [{
    "id": "cutout",
    "source": {
      "type": "path",
      "path": {
        "fillRule": "nonzero",
        "commands": [
          {"type": "moveTo", "to": {"x": 0, "y": 0}},
          {"type": "lineTo", "to": {"x": 20, "y": 0}},
          {"type": "lineTo", "to": {"x": 0, "y": 20}},
          {"type": "close"}
        ]
      },
      "paint": {"type": "solid", "color": {"r": 1, "g": 0, "b": 0, "a": 0.5}}
    },
    "channel": "alpha",
    "operation": "subtract",
    "inverted": false,
    "transform": {
      "position": {"x": 0, "y": 0, "unit": "pixels"},
      "anchor": {"x": 0, "y": 0},
      "scaleX": 1, "scaleY": 1, "rotationDeg": 0,
      "skewXDeg": 0, "skewYDeg": 0, "opacity": 1
    },
    "featherPx": 0,
    "expansionPx": 0
  }]
}
```

The active coordinates use the owner's post-crop/clip, pre-effect local raster basis: pixel position uses local pixels; normalized position uses its logical width/density,height/density, with top-left zero and pixel center((x+.5)/density,(y+.5)/density). Normalized anchor uses the path's geometric bounds before expansion/feather. Transform order is anchor, scale, skew, rotation, position; opacity multiplies coverage once. Degenerate or move-only paths reserve zero coverage. Owner and ancestor transforms remain independent.

The following coverage definitions are active under schema33 and the linked mask-rendering contract. Alpha means painted coverage alpha; luma means `0.2126R + 0.7152G + 0.0722B` in premultiplied linear RGB (straight luminance times alpha). Transparent paint contributes zero; gradients retain existing premultiplied linear interpolation. Active order is geometric fill, signed expansion, original-coordinate paint/channel extraction, feather, affine transform/opacity, inversion, ordered combination, then multiply the owner's premultiplied RGBA before ordered effects. Expansion grows support when positive and contracts it when negative. Feather uses Gaussian sigma in local pixels with transparent extension and support `ceil(3*sigma)`; zero is identity. Exact alpha-aware grid expansion, sampling and checked aggregate budgets are specified in the active rendering contract.

For existing coverage A and next coverage B, add is `A+B−A*B`, subtract is `A*(1−B)`, intersect is `A*B`, and exclude is `A+B−2*A*B`. Seed is zero for a first add/exclude and one for a first subtract/intersect; an empty stack is identity one. Inversion maps B to `1−B` within the owner's local source raster domain and cannot revive crop/clip-removed pixels or extend source support. These equations execute in the shared schema33 renderer.

Malformed structures, mask bounds, duplicate item-local IDs and unsupported ownership return nonretryable `INVALID_ARGUMENT`. A missing edit target returns `ITEM_NOT_FOUND`. A stale expected revision returns retryable `REVISION_CONFLICT`; clients must reread before retrying. These failures retain existing precedence and publish no authoritative edits or resources.

All six active animation catalogs and both mask authorities report current schema37. Exact marker-only37→36 projections preserve their previous semantics; the retained34→33 proofs preserve schema33 mask semantics. Historical migration inputs retain their original source versions. Exact authorized additions/model annotations and current markers project to pinned schema32 semantics, then the six animation catalogs retain the earlier32→31 proof. The separate full MCP projection first removes exactly approved group-compositing additions and restores37→36, then composes parameterized-effect36→35, blend35→34 and track-matte34→33 projections before the earlier additive boundaries. Every historical pin remains unchanged.
