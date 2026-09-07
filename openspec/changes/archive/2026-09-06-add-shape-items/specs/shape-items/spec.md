## ADDED Requirements

### Requirement: Closed bounded shape geometry
Core MUST support a shape item with required geometry, nullable fill and stroke, and existing common visual/timing/order fields. Geometry MUST be a closed type-tagged object: rectangle {width,height}, roundedRectangle {width,height,radii}, ellipse {width,height}, line {start,end}, polygon {points}, star {center,outerRadius,innerRadius,pointCount,rotationDeg}, or path {path}. All coordinates MUST use existing local pixel vector coordinates. Dimensions and star radii MUST be positive and at most 16384; star innerRadius MUST be less than outerRadius, pointCount MUST be an integer in [3,2048], and rotationDeg MUST be finite in [-36000,36000]. Polygon MUST contain 3–4096 points and close implicitly with nonzero fill. Star vertices MUST alternate outer and inner radius, starting upward at zero rotation and proceeding clockwise. Line endpoints MUST differ. Path MUST reuse the existing structured VectorPath contract and its fill rule.

Rectangle and ellipse local boxes MUST start at (0,0). Shape anchor bounds MUST be the unstroked geometric bounds; strokes MUST NOT move the anchor. Degenerate path bounds MUST use a one-pixel minimum only for anchor calculation, without changing geometry. Fill and stroke MUST reuse the vector paint/stroke contracts; at least one MUST be non-null, and lines MUST have null fill and a non-null stroke. Fill MUST draw before stroke. Unknown fields/tags, non-finite values, raw strings/SVG/expressions/URLs and exceeded limits MUST fail decoding or core validation with existing non-retryable INVALID_ARGUMENT semantics. Coordinates outside the viewport MUST clip, not relocate.

#### Scenario: Accept every geometry
- **WHEN** all seven variants use valid boundary values, paints and strokes
- **THEN** core preserves their typed values and produces the specified geometry, including rounded-radius proportional scaling and open path stroke semantics

#### Scenario: Reject invalid shapes
- **WHEN** geometry has unknown fields, invalid numbers, coincident line ends, invalid dimensions/radii, too few/many vertices, unsupported string content or invalid fill/stroke combinations
- **THEN** structural decoding or core semantic validation rejects it without mutation or renderer work

#### Scenario: Preserve geometric anchors and clipping
- **WHEN** a stroked, asymmetric or partially offscreen shape uses a noncentral anchor
- **THEN** placement uses unstroked bounds and output clipping retains the visible geometry without shifting its origin

### Requirement: Canonical shape evaluation and bounded raster work
Core MUST evaluate shapes into process-local drawing instructions carrying resolved geometry, paint, stroke, transforms, visibility, half-open timing and canonical stacking. Root and component-local shapes MUST obey existing parent scopes, instance clocks, inherited opacity and expansion limits. Hidden and unused persisted content MUST still validate. Shapes MUST introduce no media references. Shape rendering MUST preserve linear-light premultiplied gradient interpolation, pad extension, fill rules, centered strokes, caps/joins/dashes and miter fallback from vector-primitives.

Each shape MUST compile to at most 8192 path commands and at most 65536 flattened segments, with a maximum curve subdivision depth of 16 and target deviation at most 0.25 output pixels. The authored VectorPath limit MUST remain 4096 commands; the compiled limit accommodates closure of maximum-sized polygons and stars. A scene sample MUST contain at most 1048576 flattened segments; checks MUST precede allocation beyond these limits. Failure to meet tolerance within the bounds MUST return INVALID_ARGUMENT without degraded geometry. Existing evaluated geometry, surface-memory and visual-layer limits MUST also apply before rasterization.

#### Scenario: Evaluate nested and repeated shapes
- **WHEN** root and component-local shapes appear under transformed parents and repeated time-scaled instances
- **THEN** evaluation preserves occurrence identity, local clocks, canonical order, visibility and opacity without changing audio or consulting persisted state downstream

#### Scenario: Bound expensive geometry
- **WHEN** geometry exceeds command, subdivision, segment, scene or existing surface limits, including after transforms and instance expansion
- **THEN** core returns INVALID_ARGUMENT before raster allocation, writes or render execution

#### Scenario: Render exact vector semantics
- **WHEN** fixtures exercise gradient transparency, both fill rules, open and closed curves, dash phase, every cap/join and miter fallback
- **THEN** independently checked pixels and geometry reflect the existing vector contract rather than backend defaults
