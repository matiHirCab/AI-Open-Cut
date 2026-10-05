## Why

Issue #49 (MG-M4-01) requires the normative premultiplied linear-light pipeline. Existing ordered effects run in linear local rasters, but their outputs and ordinary layers are blended by FFmpeg in encoded color space, so translucent overlaps and transformed edges violate the documented compositing semantics.

## What Changes

- Apply one source → crop/local clip → masks → ordered effects → local/ancestor transform → matte → opacity → destination blend pipeline to every currently supported evaluated visual, including visuals without effects or Transform2D.
- Keep floating-point linear premultiplied color through resampling, temporal averaging, and scene source-over; convert to the existing SDR output representation only after final scene composition.
- Make absent, currently unrepresentable mask/matte/non-normal blend stages explicit identity/default stages. Existing crop, SVG clipping, and four authored effects remain supported; this change adds no authoring fields.
- Add analytic color/alpha/order oracles, mixed-source production intent parity, bounded-work failure evidence, and deliberately reviewed golden updates where the correction changes output.

## Capabilities

### New Capabilities

- `linear-light-compositing`: Normative current SDR color representation, pipeline stages, source-over equations, and bounded shared scene composition.

### Modified Capabilities

- `motion-graphics-architecture`: Clarify hybrid renderer ownership and distinguish current representable stages from future mask/matte/blend milestones.
- `rendering-export`: Add readiness-gated client detection and require shared linear composition across all render intents and document the intentional correction to translucent legacy output.
- `render-regression-fixtures`: Require independently derived linear color/alpha/order evidence, reviewed corrected references, and honest versioned canonical benchmark workload definitions for full-scene materialization.

## Non-goals

No mask/matte model, non-normal blend modes, new effects, group clipping, authored color-space controls, HDR/wide-gamut management, new transport operations, or persisted schema changes. Issues #50–#56 retain their authoring scope. This milestone does not promise floating-point accuracy inside a pre-existing vector/text source rasterizer; its decoded local RGBA raster is the source boundary, and existing graphic paint/clipping semantics remain governed by their owning specifications.

## Impact

The owning code is editor-core evaluated-scene certification, raster/resource preparation, render planning, and renderer integration; FFmpeg retains media decode, audio, and encoding. Existing request/project/capability/error contracts, revision/history behavior, path safety, and persisted fixtures remain unchanged. Add a readiness-gated `linear_light_compositing_v1` capability to protocol v1 and its canonical headless/MCP catalogs, synchronize governed TypeScript/Rust consumers and parity tests, and obtain designated contract CODEOWNER review. This uniquely named capability is additive; no migration or protocol/schema major bump is required. This is an intentional rendering correction: encoded-space translucent blends are replaced with linear source-over while opaque identity content, geometry, clocks, audio, and simple operation acceptance remain compatible. Existing output bytes are not a compatibility promise; reference changes require independent analytic evidence and review. All repository-required checks and archive-only gates remain mandatory.

The canonical measured fixture advances to revision 4 and stage-definition version 2, with exact immediately preceding revision-3/schema-3/stage-1 migration and strict rejection of other version combinations.

Render preflight also certifies fractional native media source clocks to a conservative 1ns arithmetic-error bound. Unsafe floating-point combinations fail before resources; authored Java-safe whole times and persisted history remain valid, and exact whole-millisecond backend compatibility is retained.
