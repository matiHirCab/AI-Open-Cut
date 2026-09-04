## Context

The current private geometry probe selects stream dimensions and stream display matrices. JPEG EXIF orientation instead reaches FFmpeg through decoded-frame side data. The renderer therefore samples an automatically rotated frame using the wrong source dimensions.

## Goals / Non-Goals

Preserve full static-image extent and orientation for identity and combined Transform2D across frame, range, draft, and export. Preserve existing video probing, schema 8, public contracts, safe managed bindings, typed failures, validation ordering, and legacy output. Dynamic image/video geometry, backend upgrades, and migrations are excluded.

## Decisions

Extend the private geometry-probe port with image/video source kind obtained from the existing resolved media binding. Do not infer kind from filenames or add it to public payloads. Video requests keep their stream-only query. Image requests select v:0, use -read_intervals %+#1, and request frame width/height and frame-side-data type/displaymatrix as JSON. Require exactly one usable frame. Its dimensions and frame matrix are authoritative; absent orientation means identity. Do not fall back to encoded stream geometry.

Recognize the frame-side-data name `3x3 displaymatrix`, reuse the current fixed-point matrix parser and FFmpeg-compatible angle rounding/quarter-turn extent logic, and leave autorotation enabled. No orientation is applied by core. The alternative of directly parsing EXIF was considered and rejected by the user in favor of configured FFprobe interpreting the image in the same way as rendering.

Retain the 64 KiB stdout cap and kill/wait cleanup on overflow/read failure. Unusable frame data, invalid dimensions/matrix, or probe failure returns UNSUPPORTED_MEDIA; inability to start the configured probe returns DEPENDENCY_UNAVAILABLE. Raw metadata stays in the adapter, with only typed dimensions used for affine finalization.

Reference/value validation, canonical managed binding resolution, and text measurement/overflow validation precede metadata inspection. Inspection may allocate decoder-internal memory, but produces no raster artifact, workspace, or resource write. Completed affine validation precedes destination inspection and materialization. Cache dimensions once per transformed asset per preparation, and reuse the finalized scene without probing again.

Native tests generate an asymmetric 40x20 JPEG with configured FFmpeg and inject a minimal TIFF EXIF orientation segment in a test-only helper. Cover absent orientation and values 1-8. Compare legacy and identity decoded previews for geometry/content, including 800 colored pixels and 20x40 extent for orientation 6. Use quarter-turn and reflected images with combined transforms/noncentral anchors for draft/frame/range/export parity. Keep fixtures temporary and use existing optional/partial/required native configuration policy.

## Risks / Trade-offs

- First-frame inspection requires decoder work: bound packet inspection and metadata output; reject missing usable frames instead of silently using unrotated dimensions. This is explicitly permitted preflight work, distinct from rendering and artifact production.
- FFprobe frame metadata differs from stream metadata: hermetic tests cover the exact frame-side-data name, missing/malformed matrices, invalid dimensions, and missing frames; native tests cover all EXIF orientations.
- Host-specific dependency behavior: use configured absolute executable paths and checked-in fonts; run required Linux native coverage and the three-platform correctness matrix. Do not claim remote jobs passed from Windows-only evidence.

## Migration Plan

No migration or persisted rewrite. Existing projects receive corrected image dimensions on their next render. Rollback consists of reverting this private implementation and its documentation; no data conversion is needed. Retain historical verification archives and add a correction link rather than rewriting old evidence.

## Open Questions

None concerning implementation scope. Concrete artifacts were explicitly approved before implementation on 2026-09-04.
