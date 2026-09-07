## MODIFIED Requirements

### Requirement: Explicit SVG geometry and complexity
Root width and height MUST be positive finite unitless or px lengths at most 16384; percentages and other units MUST fail. An omitted viewBox MUST be (0,0,width,height); supplied viewBox MUST contain four finite numbers with positive extents and coordinates within existing vector bounds. preserveAspectRatio MUST be omitted or xMidYMid meet, mapping uniformly into the centered viewport with transparent margins and clipping to the viewport. Coordinates MUST use top-left origin, positive X right and Y down. Geometry MUST use existing vector coordinate and stroke bounds. Rectangles MUST support nonnegative rx/ry only when equal after copying one omitted radius to the other; effective radius MUST clamp to half the smaller dimension. Circle/ellipse radii MUST be positive. Polygon/polyline MUST contain at least three/two points respectively. Paths MUST accept only absolute M/L/H/V/Q/C/Z (including legal repeated argument sets), lower to structured paths, and reject arcs, relative commands and unsupported grammar. Solid colors MUST accept only none, #RGB, #RGBA, #RRGGBB and #RRGGBBAA, with SVG hex alpha semantics; inherited defaults are black fill/no stroke, nonzero fill rule, width 1, butt cap, miter join, miter limit 4, no dash and offset 0. Lines MUST ignore inherited fill and require visible stroke; other paintless geometry MUST fail. Shapes MUST fill before stroking, and open drawable subpaths MUST close only for fill.

Input MUST be at most 1048576 UTF-8 bytes, XML depth at most 32 including the root, element count at most 4096 including groups/root, and attribute count at most 32 per element. Each path MUST contain at most 4096 normalized commands; the document MUST contain at most 65536 normalized commands, counting primitive lowering. Checks MUST precede allocation or expansion beyond each budget. Existing compiled shape, curve subdivision, flattened segment, scene, transformed surface and raster memory limits MUST also apply, including hidden content and component expansion. Non-finite or overflowed numeric intermediates MUST fail with INVALID_ARGUMENT without lowering quality.


A supported drawing command L/H/V/Q/C following Z MUST begin a new normalized subpath at the closed subpath's initial point; the current point MUST reset to that initial point. An explicit M MUST start its own subpath without an extra move. Implicit MoveTo records MUST count against path and document command budgets before insertion. The persisted structured path grammar MUST remain unchanged.

SVG raster surface limits MUST apply to the mapped viewport at the composed sampling density, without phantom padding or an unused source-space child surface. Valid bounded source geometry that fits those limits after mapping MUST NOT fail solely because its standalone source-space surface would be larger. Surface dimensions MUST remain at most 16384 pixels and area at most 16777216 pixels. Compilation MUST preserve source geometry, subdivision and per-shape/scene work bounds, check remaining scene capacity before expansion, and reject non-finite or unrepresentable mapped raster coordinates, stroke widths and dash metrics before artifact preparation, without silent omission or quality reduction.

#### Scenario: Preserve viewport and drawing semantics
- **WHEN** valid artwork uses nonzero viewBox origin, unequal viewport aspect ratio, open curves, alpha colors and overlapping siblings
- **THEN** independently checked geometry and pixels preserve centered meet scaling, transparent margins, clipping, fill/stroke semantics and document order

#### Scenario: Enforce every boundary before expensive work
- **WHEN** valid fixtures hit each inclusive budget or counterexamples exceed one budget, use non-finite numbers or invalid grammar
- **THEN** valid boundaries succeed and counterexamples return INVALID_ARGUMENT before excess allocation, rasterization or writes

#### Scenario: Continue supported drawing after closepath
- **WHEN** legal absolute L/H/V/Q/C commands follow Z, with nonzero subpath starts, repeated argument sets or a later explicit M
- **THEN** normalized geometry starts each implicit continuation at the closed subpath's initial point and explicit moves introduce no duplicate move

#### Scenario: Count implicit continuation commands before expansion
- **WHEN** inserted MoveTo records bring a path or document to its inclusive command limit, or would exceed that limit
- **THEN** boundary inputs succeed and excess input returns INVALID_ARGUMENT before excess command allocation, with existing revision and batch rollback guarantees

#### Scenario: Render large source geometry through a small viewport
- **WHEN** a 100x100 SVG uses viewBox 0 0 10000 10000 and a matching rectangle, or equivalent mapped artwork uses nonzero origins, clipping, curves and strokes
- **THEN** rendering succeeds with independently correct viewport pixels and no unused source-space raster allocation

#### Scenario: Enforce actual mapped surface and precision budgets
- **WHEN** viewport mapping and composed magnification reach or exceed a surface, subdivision, scene-work or memory limit, including hidden or expanded component content, or mapped raster values become unrepresentable
- **THEN** inclusive valid bounds succeed and invalid values fail with INVALID_ARGUMENT before excess allocation or artifact publication without weakening standalone shape limits

Mapped SVG raster representability MUST include outward-rounded backend integer bounds, representable bound dimensions and coordinate arithmetic, plus conservative stroke expansion for caps and miters. Dash conversion and construction MUST succeed without non-finite intermediates or fallback to an undashed stroke. Unrepresentable geometry MUST return non-retryable INVALID_ARGUMENT before artifact preparation or backend execution. Legitimate empty coverage from offscreen geometry or degenerate fills MUST remain valid when numeric bounds are representable. Validation and raster preparation MUST use consistent coordinate conversion rules; geometry MUST NOT be individually clamped or silently dropped.

Transform-dependent SVG limits MUST use complete component occurrence transforms, including group ancestry and existing keyframe scale bounds and clock semantics. Referenced definitions MUST NOT additionally undergo isolated-root raster sizing. Hidden occurrences MUST undergo the same bounded validation. Definitions unreachable from project instances MUST be validated as virtual roots with identity outer transforms, including their nested instances. All stored normalized documents MUST remain validated. Remaining work and memory capacity MUST be enforced during occurrence expansion without counting intermediate compilation twice; existing cycle, depth, occurrence, subdivision, surface and aggregate limits MUST remain enforced.

#### Scenario: Reject finite but backend-unrepresentable mapping
- **WHEN** a 100x100 SVG uses viewBox 0 0 0.0001 0.0001 and a rectangle spanning -5000 to 5000 on both axes, or fill/stroke bounds overflow backend numeric conversion
- **THEN** evaluation returns non-retryable INVALID_ARGUMENT before artifact preparation, rather than successfully publishing blank or incomplete output

#### Scenario: Preserve representable clipping and empty coverage
- **WHEN** accepted boundary values, negative coordinates, viewport-crossing fills or strokes, caps, miters, dashes, offscreen geometry or degenerate fills remain numerically representable
- **THEN** evaluation succeeds and independent expected pixels or empty coverage match existing clipping and paint semantics

#### Scenario: Compose cancelling component scales before sizing
- **WHEN** a 100x100 SVG has local scale 100 inside a component instantiated at scale 0.01, including equivalent nested group and component ancestry using legacy transforms or Transform2D
- **THEN** the occurrence renders like the equivalent identity-scale scene without an isolated component surface rejection

#### Scenario: Validate hidden and unreachable occurrence graphs
- **WHEN** hidden occurrences, unreachable definitions with nested instances, or multiple differently scaled instances contain SVGs
- **THEN** each occurrence uses its complete applicable transform, unreachable definitions use identity outer transforms, and oversized or excessive graphs fail before exceeding existing budgets

#### Scenario: Preserve rendering lifecycle and failure atomicity
- **WHEN** headless or MCP operations render equivalent corrected scenes through frame, range, materialized draft and export before and after undo, redo and reopen, or encounter invalid evaluated geometry
- **THEN** existing independent pixel, semantic, audio and timing tolerances hold; failures preserve revision, state, retained history and published artifacts; stale edit revisions retain REVISION_CONFLICT precedence and schema 15 and protocol 1 remain unchanged
