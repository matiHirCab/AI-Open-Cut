## Context

Issue #30 follows completed vector and shape milestones. Before this change, core persisted schema 15, including SVG items; shapes already supported scale-aware vector coverage, strict duplicate-preserving decoding, common visual lifecycle, component evaluation, and renderer parity. The user approved this change's proposal, design, delta specifications and tasks on 2026-09-07 before implementation. The user also approved the resulting cross-language contracts on 2026-09-07 in response to the final designated review request, satisfying the review gate before closure.

## Goals / Non-Goals

**Goals:** Editable rectangular, diagonal, dot and isometric grids; finite bounded core semantics; discoverable typed standalone/batch workflows; reversible durable editing; identical evaluated behavior for all output intents.

**Non-Goals:** See proposal. Grid geometry and paint animation, independently styled families, desktop creation controls and schema downgrade are excluded.

## Decisions

### Persist a procedural grid descriptor

Add `TimelineItem::Grid(GridItem)` with wire item type `grid`, common item/visual fields and required `grid`. The closed descriptor has `width`, `height`, `pattern`; pattern is one of:

| type | Required additional fields |
| --- | --- |
| rectangular | spacingX, spacingY, stroke |
| diagonal | spacing, stroke |
| dot | spacingX, spacingY, radius, paint |
| isometric | spacing, stroke |

All paints/strokes reuse existing typed vector objects. Local dimensions and spacing are finite in (0,16384]; dot radius is finite in (0,min(spacingX,spacingY)/2]. Every field shown is required; no new nullable style or default geometry fields. Strict objects reject unknown/missing/duplicate fields and positional arrays through raw JSON-bearing consumers.

Persisting parameters keeps grids editable and compact. Expanding into hundreds of persisted shapes would burden aliases, history and user edits. Adding grid as a ShapeGeometry variant would overload the fixed shape contract and its single-path limits; a separate item can compile many bounded primitives internally while preserving one timeline identity.

### Define geometry mathematically

The local viewport is [0,width] x [0,height], with transparent background and anchor bounds equal to that rectangle. Rectangular families are x=k*spacingX then y=k*spacingY. Dots are circles centered at (i*spacingX,j*spacingY), enumerated by ascending row j then column i. Include lattice positions on viewport boundaries, and include centers only inside the viewport.

Diagonal grids use two families with unit normals (1/sqrt(2),1/sqrt(2)) and (1/sqrt(2),-1/sqrt(2)). Isometric grids use normals (1,0), (1/2,sqrt(3)/2), (1/2,-sqrt(3)/2). For each family, draw n dot (x,y)=k*spacing for every integer k whose centerline intersects the viewport with positive length. Spacing is perpendicular distance between parallel lines. Enumerate families in listed order and k ascending; order segment endpoints lexicographically (x then y), with dash phase restarting at the first clipped endpoint. This supplies deterministic 45-degree diagonal and 60-degree isometric line families without an extra angle/offset contract. Position and arbitrary rotation use existing visual transforms.

Clip final stroked/filled coverage to the local viewport before the composed visual transform; do not relocate or stretch edge marks. Local viewport clipping is internal grid semantics, not a public mask feature. Stroke caps, dashes and paints retain existing semantics, with gradients evaluated in the grid coordinate system (not restarted per mark). Marks composite in enumeration order using existing premultiplied source-over; translucent intersections accumulate coverage explicitly.

### Bound procedural work before expansion

At most 4096 positive-length line segments or dot centers per grid; count using finite checked arithmetic before materializing geometry. A 63-by-63 dot grid with spacing 1 has 4096 centers and reaches the inclusive boundary. A 64-by-63 grid at spacing 1 has 4160 and fails. Reject underflow/overflow and non-finite derived calculations, even for hidden or unused definitions.

Keep existing per-primitive command/curve limits and add an aggregate 65536 flattened segments per grid; retain the 1048576 flattened segments per scene sample limit. Count expanded occurrences before preparing surfaces. Reuse scale-aware density, 16384 raster dimension, 16777216 pixel area and existing aggregate memory limits; checked preflight must account for all live grid coverage/paint/composite buffers. Never reduce density, spacing accuracy or curve tolerance to make a request fit. An analytic shader backend was considered but would duplicate evaluated behavior and complicate raster parity.

### Integrate within existing owners

Model holds strict records; validation owns descriptor and count acceptance; timeline owns add/update/lifecycle; migrations and persistence own atomic schema upgrades; evaluated_scene owns bounded deterministic grid expansion and composed transforms/clocks. Grid expansion emits process-local drawing facts consumed by render_plan/render_artifact and existing vector coverage. Render owners must not inspect persisted GridItem records. Use existing root re-exports for model primitives and preserve ADR 0003; any unavoidable new owner edge requires a reviewed ADR/architecture-test update before implementation.

`add_grid` requires trackId/startMs/durationMs/grid with optional complete transform2d/parent, matching shape defaults and revision envelopes. `update_item` gets a grid-only optional complete `grid` replacement; omission preserves, null fails. Generic geometry/fill/stroke patches do not target grids; nested styling changes require replacing `grid`. Root grids support the existing shape visual lifecycle. Definitions accept local grids through existing create/update operations. No new per-component editing transport is introduced.

### Preserve codec tolerance for fine procedural marks

The native fixture demonstrated SSIM 0.98425 when comparing the existing CRF-28/veryfast range preset with default export encoding. To implement the approved >=0.99 grid parity requirement, the render plan carries a private grid-fidelity fact derived from evaluated layers; grid-bearing ranges use the existing export quality (CRF 23, medium). Scenes without grids retain their prior range settings, and no public codec option, capability, persisted field or new dependency is introduced. Render-process consumes only that plan fact, preserving the ownership graph. The native fixture retains the fine translucent colored marks that revealed the mismatch.

### Contract synchronization

Add `contracts/procedural-grids-v1.json`, with version, exact identifiers/limits, activated status, valid/invalid structural and semantic fixtures, and deterministic geometry examples. Register all consumers in contract ownership, update operation/MCP surface/capability catalogs and parity runners, and obtain designated CODEOWNER review. Mirror representation in TypeScript without relocating domain ownership to adapters. Capability `grid_items` denotes editing; `grid_rendering` is available only with complete renderer readiness. Protocol 1 envelopes, errors and existing operations retain their meaning; the new project item variant requires a grid-aware reader. No claim of old-client support for decoding grid-bearing projects is made.

## Risks / Trade-offs

- Dense procedural expansion can be cheap to encode but expensive to render -> checked descriptor counts, primitive and scene budgets, transform-aware surface preflight, and maximum/overflow regression tests.
- Dash origin, translucent crossings, gradients and border clipping can vary between renderers -> canonical enumeration, endpoint order, local paint space and independent pixel/geometry expectations.
- A new item variant touches exhaustive consumers -> audit core, drafts, components, asset collection, headless/bridge and desktop matches; preserve media-free behavior and use generic presentation labels without authoring controls.
- Schema upgrades prevent opening new saves in older binaries -> explicit schema version, source gating, recovery tests and no in-place downgrade.

## Migration Plan

After approval, implement schema 16. Validate every source snapshot before relabeling; reject `grid` items under source versions below 16, including hidden/unused component items. Supported 1-15 snapshots retain all earlier migration steps; 15-to-16 changes only schemaVersion. Publish current and all retained undo/redo snapshots under the existing project lock as one recoverable generation. Preserve revisions, IDs, ordering, assets and provenance. Future versions retain existing INTERNAL_ERROR behavior. Invalid state fails without writes; injected interruptions recover a complete generation. Operational rollback uses a preserved pre-upgrade project copy with its matching history; no automatic backward rewrite.

## Verification Plan

Each requirement/scenario maps to tasks below and automated tests. Cover all patterns and paints, exact formula oracles, boundaries and representation failures; transaction errors and aliases; updates and lifecycle; mixed-history migrations and crash recovery; root/nested/retimed/draft evaluation; output parity and legacy regressions. Render fixtures require independent expected geometry/pixels in addition to shared-path comparison. Existing thresholds remain SSIM >=0.99, aligned float-PCM RMS <=0.0001 and timing within one output frame.

Run strict pinned OpenSpec validation during authoring. After implementation run Rust fmt/strict workspace Clippy/workspace tests, bridge typecheck/lint/unit/contract/integration/packaged smoke, applicable renderer golden gates and relevant hermetic Python worker checks. Record exact commands/results and any unavailable required checks; skipped required checks block completion. Run openspec-verify-change and resolve mismatches before archival. Because repository policy permits only archived changes at the protected Moon boundary, validate the active change directly, verify and archive it, then run `moon run root:openspec-validate` on the archived state as the final policy gate.

## Open Questions

The user explicitly approved this descriptor, grid origin, family orientations, paint overlap semantics and limits on 2026-09-07. No design question remains open. Any later scope change returns to the artifact gate.
