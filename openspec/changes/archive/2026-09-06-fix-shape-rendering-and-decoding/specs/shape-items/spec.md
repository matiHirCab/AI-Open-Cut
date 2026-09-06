## MODIFIED Requirements

### Requirement: Closed bounded shape geometry
Core MUST support a shape item with required geometry, nullable fill and stroke, and existing common visual/timing/order fields. Geometry MUST be a closed type-tagged object: rectangle {width,height}, roundedRectangle {width,height,radii}, ellipse {width,height}, line {start,end}, polygon {points}, star {center,outerRadius,innerRadius,pointCount,rotationDeg}, or path {path}. All coordinates MUST use existing local pixel vector coordinates. Dimensions and star radii MUST be positive and at most 16384; star innerRadius MUST be less than outerRadius, pointCount MUST be an integer in [3,2048], and rotationDeg MUST be finite in [-36000,36000]. Polygon MUST contain 3–4096 points and close implicitly with nonzero fill. Star vertices MUST alternate outer and inner radius, starting upward at zero rotation and proceeding clockwise. Line endpoints MUST differ. Path MUST reuse the existing structured VectorPath contract and its fill rule.

Rectangle and ellipse local boxes MUST start at (0,0). Shape anchor bounds MUST be the unstroked geometric bounds; strokes MUST NOT move the anchor. Positive geometric extents MUST retain their actual value even below one pixel. Only zero-extent axes of degenerate path geometry MUST use a one-pixel extent for anchor calculation, without changing geometry; zero-extent axes of other geometry MUST remain zero. Fill and stroke MUST reuse the vector paint/stroke contracts; at least one MUST be non-null, and lines MUST have null fill and a non-null stroke. Fill MUST draw before stroke. Unknown fields/tags, non-finite values, raw strings/SVG/expressions/URLs and exceeded limits MUST fail decoding or core validation with existing non-retryable INVALID_ARGUMENT semantics. Coordinates outside the viewport MUST clip, not relocate.

#### Scenario: Accept every geometry
- **WHEN** all seven variants use valid boundary values, paints and strokes
- **THEN** core preserves their typed values and produces the specified geometry, including rounded-radius proportional scaling and open path stroke semantics

#### Scenario: Reject invalid shapes
- **WHEN** geometry has unknown fields, invalid numbers, coincident line ends, invalid dimensions/radii, too few/many vertices, unsupported string content or invalid fill/stroke combinations
- **THEN** structural decoding or core semantic validation rejects it without mutation or renderer work

#### Scenario: Preserve geometric anchors and clipping
- **WHEN** a stroked, asymmetric or partially offscreen shape uses a noncentral anchor
- **THEN** placement uses unstroked bounds and output clipping retains the visible geometry without shifting its origin

#### Scenario: Preserve fractional and degenerate anchors
- **WHEN** a 0.5 by 0.5 rectangle at scale 100 uses anchor (1,1) and position (100,100), or anchor (0,0) and position (50,50)
- **THEN** both placements have identical geometry and coverage, and fractional extents, horizontal/vertical lines, and degenerate paths retain their specified anchor rules

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

## ADDED Requirements

### Requirement: Scale-aware shape coverage
Shape rasterization MUST account for maximum effective magnification over the rendered interval, including composed transforms, ancestors, component instances, and animated scale. Raster density MUST be at least one and at least the largest singular value of the effective local-to-output linear transform. Geometry, local paint coordinates, stroke width and dash phase, anchors, and opacity MUST remain unchanged by internal raster density. Preview, range rendering, and export MUST consume the same evaluated semantics. Density-adjusted surfaces and aggregate memory MUST satisfy existing limits before allocation or artifact writes; exceeding limits MUST return existing non-retryable INVALID_ARGUMENT rather than silently degrade quality.

#### Scenario: Match equivalent magnified geometry
- **WHEN** an opaque 2 by 2 ellipse is magnified 50 times and compared with an equivalently positioned 100 by 100 ellipse
- **THEN** interiors remain opaque, exterior corners remain transparent, and contour coverage agrees within antialiasing tolerance

#### Scenario: Preserve transformed styles and interval fidelity
- **WHEN** shapes use nonuniform scale, rotation, skew, transformed parents, component transforms, animated scale, gradients, or dashed strokes
- **THEN** density compensation preserves local style and geometric placement throughout the interval, and matching preview/range/export samples agree within the established codec tolerances

#### Scenario: Reject excessive density-adjusted work
- **WHEN** effective magnification causes a per-shape surface or aggregate scene allocation to exceed an existing limit
- **THEN** evaluation rejects the work before allocation, writes, or renderer execution, without mutating project state
