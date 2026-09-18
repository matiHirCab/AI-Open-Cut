# Styled text layers

Schema 20 and capability `styled_text_layers_v1` add optional `document.spans` and `style.paintLayers` to existing text operations. Status accepts `styledTextLayersVersion: 1`; unsupported versions fail with `INVALID_ARGUMENT`.

Span offsets are zero-based, half-open Unicode 16.0 extended grapheme indices across concatenated runs, without normalization. A combining sequence, flag, ZWJ emoji or CRLF counts once even across run boundaries. Up to 256 sorted, nonoverlapping, nonempty spans can override `bold`, `italic`, `color`, or `paintLayers`. Explicit false overrides a run's true. Paint-only boundaries preserve shaping: a ligature crossing a paint boundary uses the paint at its lowest logical source offset.

```json
{
  "operation": "add_text",
  "trackId": "overlay-track",
  "document": {
    "runs": [{"text": "Hello 👩‍💻"}],
    "spans": [{"start": 6, "end": 7, "style": {"bold": true}}]
  },
  "fontSize": 48,
  "color": "#ffffff",
  "startMs": 0,
  "durationMs": 2000,
  "style": {"paintLayers": [
    {"kind": "shadow", "color": "#000000", "opacity": 0.6, "offsetXPx": 2, "offsetYPx": 4, "blurSigmaPx": 3},
    {"kind": "stroke", "color": "#223366", "opacity": 1, "widthPx": 6},
    {"kind": "fill", "color": "#ffffff", "opacity": 1}
  ]}
}
```

For standalone MCP `timeline_add_text`, omit `operation` and add `projectId` and `expectedRevision` to these arguments. For `timeline_batch_edit`, put this edit in its `operations` array with `resultAlias: "title"`, and supply `projectId` and `expectedRevision` on the batch. A later operation `{ "operation": "update_item", "itemId": "@title", "style": { "paintLayers": [] } }` hides its glyph ink in the same atomic batch. Existing style replacement semantics apply; supply other style fields when preserving nondefault values.

Stacks contain 0–16 layers, painted in array order with source-over. Span stacks replace item stacks; an empty stack paints no ink while retaining layout and background. Omitted item stacks retain the legacy shadow/outline/fill path. Explicit fill colors override run/span color. Contiguous effective style segments paint in logical source order, each with its own complete stack. Paints use #RRGGBB and finite opacity in [0,1].

Coordinates are local project pixels before item/group transforms: x right, y down. Stroke widths are fractional total centered widths in (0,200], with round joins. Shadow offsets are in [-4096,4096]; Gaussian sigma is in [0,64], with transparent edges and support ceil(3*sigma). Zero sigma gives an unblurred shadow. Shadows use filled glyph coverage alone, excluding outlines, background and earlier layers.

Core checks raster dimensions (16384), area (16777216 pixels) and aggregate paint work (268435456 pixel passes) before output work. Each fill/stroke/unblurred shadow counts one pass; a blurred shadow counts three per effective segment. Existing text, glyph, line, scene and font limits still apply. Invalid fields, null, ranges, non-finite numbers or excess complexity fail atomically with `INVALID_ARGUMENT`; missing references, stale revisions and resource errors retain their existing codes.

Schema-19 projects and retained undo/redo migrate atomically to 20 with new fields absent, preserving existing rendering and pinned fonts. Older supported schemas first undergo their documented migrations. Existing drafts remain usable, unchanged reopen does not rewrite files, and unknown future versions fail closed. Older binaries cannot open schema 20; no downgrade is provided.

Simple `text` replacement clears spans/run overrides, while style-only edits preserve the document. Rich-text slot substitution replaces its document including spans. Copy, split, move, trim, undo/redo, drafts, frame/range preview and export share these semantics.

Range previews containing spans or paint stacks use the same medium/CRF-23 encoding quality as detailed procedural graphics to preserve visual parity with export.
