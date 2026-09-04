# Static Transform2D

Schema 8 adds optional `transform2d` common visual properties. A ready renderer advertises `transform2d`. Set it through `timeline_update_item` or `update_item` inside `timeline_batch_edit`; a batch can create an item with `resultAlias` and transform `@alias` in the same revision.

```json
{
  "operation": "update_item",
  "itemId": "@title",
  "transform2d": {
    "position": { "x": 0.5, "y": 0.5, "unit": "normalized" },
    "anchor": { "x": 0.5, "y": 0.5 },
    "scaleX": 1.25,
    "scaleY": 0.75,
    "rotationDeg": 20,
    "skewXDeg": 5,
    "skewYDeg": -3,
    "opacity": 0.8
  }
}
```

Every field is required when setting a transform. Omit `transform2d` to leave it unchanged; send `null` to restore the retained legacy `transform`. Setting legacy `transform` clears `transform2d`. Sending both in one update fails. Static transforms support visual media, text, solids, rectangles, and captions. Audio-only media and transition records have no independently transformable source. Position, scale, and opacity keyframes cannot coexist with active Transform2D; audio volume keyframes remain supported.

## Coordinates and order

The origin is top-left, positive X is right, and positive Y is down. Pixel position is literal; normalized position resolves against composition dimensions. Anchor fractions refer to the unscaled source box. Positive rotation is clockwise. With column vectors:

`T(position) R(rotation) Ky(skewY) Kx(skewX) S(scaleX,scaleY) T(-anchor*sourceSize)`

X shear precedes Y shear, using the tangent of each angle in degrees. Opacity is applied once after affine sampling. Timing remains integer-millisecond half-open intervals; layer ordering and audio semantics are unchanged.

Rectangles use declared dimensions; solids use composition dimensions; media uses the source dimensions after the configured backend applies display orientation, obtained through read-only metadata inspection of the managed asset. Text uses its measured styled box, including padding, outline, and shadow. Transformed captions use measured text dimensions rounded up, at least one pixel per axis, plus 24 pixels per axis; text is inset 12 pixels, with existing colors and background alpha 0.75. Explicit position replaces bottom-center placement. Captions without Transform2D retain legacy placement.

Font resolution and measurement occur read-only before core affine finalization. Measured text bounds are validated before metadata probing. A private, bounded FFprobe inspection supplies oriented media dimensions once per transformed asset; no persisted metadata is rewritten. For static images, inspection decodes the first image frame within a one-packet bound to read its dimensions and EXIF-derived display matrix; video inspection uses stream metadata. Decoder-internal memory is permitted during this read-only step, but it produces no raster artifact or resource write. Missing or unusable frame metadata fails closed instead of using encoded dimensions. Paths stay in the resource sidecar. Rendering reuses the selected font and layout. Core rejects excessive geometry before export-collision inspection, temporary-name allocation, workspace creation, text writes, or raster artifact production. A valid export collision still requires explicit overwrite permission. Frame, range, draft, and export consume the same evaluated affine facts.

## Validation and compatibility

All values must be finite. Anchor is in `[0,1]`, each scale in `(0,100]`, opacity in `[0,1]`, rotation in `[-36000,36000]` degrees, and each skew in `[-80,80]` degrees. Position magnitude is at most 1,000,000 pixels or 100 normalized units. Outward-rounded transformed bounds are limited to 16,384 pixels per dimension and 16,777,216 pixels in area before composition clipping. Existing scene collection limits still apply.

Invalid values, conflicting representations, incompatible animation, missing source measurements, and excessive geometry produce `INVALID_ARGUMENT`. Missing assets retain `ASSET_NOT_FOUND` precedence. Existing item, lock, revision, and rollback errors remain unchanged. The local affine adapter uses bounded output-sized coordinate maps and fails with `DEPENDENCY_UNAVAILABLE` before rasterization when source dimensions cannot be represented by its 16-bit maps (65,535 pixels or larger per dimension), or required filters are unavailable. It never publishes an approximated artifact. No expressions, SVG, paths, or network locators are accepted inside Transform2D.

Migration upgrades schemas 1–7 and all retained history atomically to schema 8 with Transform2D absent by default. Legacy transforms and output are preserved. Reopen, split, duplication, undo, and redo retain active transforms. Older binaries must reject schema 8; rollback requires a compatible reader or a complete pre-upgrade backup.

The runtime vocabulary and fixtures are in `contracts/transform2d-v1.json`. Remaining roadmap vocabulary stays fixture-only. Native regression checks require exact evaluated semantics, SSIM at least 0.99, aligned PCM RMS error at most 0.0001, and timing within one output frame.
