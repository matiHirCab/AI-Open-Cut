## ADDED Requirements

### Requirement: Strict bounded grid descriptors
Core MUST support item type `grid` with required descriptor `grid` containing width, height and pattern. Pattern MUST be a closed type-tagged object: rectangular {spacingX,spacingY,stroke}, diagonal {spacing,stroke}, dot {spacingX,spacingY,radius,paint}, or isometric {spacing,stroke}. Dimensions and spacing MUST be finite in (0,16384]; dot radius MUST be finite in (0,min(spacingX,spacingY)/2]. Paint and stroke MUST use existing vector-primitives semantics. Objects MUST reject missing, unknown and duplicate fields and positional arrays, including raw nested edit, batch, draft, component, project and history input. Raw strings, SVG, expressions, paths and network resources MUST NOT be accepted as grid descriptors or paints. Malformed structures MUST fail decoding; semantic invalidity MUST return non-retryable INVALID_ARGUMENT without mutation or I/O.

#### Scenario: Accept every pattern and paint
- **WHEN** all four pattern variants use legal fractional/boundary dimensions and spacing, legal dot radius and solid, linear or radial paints with supported stroke options
- **THEN** strict decoding and core validation preserve every supplied value and canonical field, including transparent paints

#### Scenario: Reject malformed and unsafe descriptors
- **WHEN** any grid-bearing consumer receives unknown/missing/duplicate fields, positional records, unsupported content, null grid, non-finite values, invalid stroke/paint, zero/oversized spacing or an excessive dot radius
- **THEN** decoding or core validation rejects it before mutation, normalization, resource resolution or rendering

### Requirement: Canonical grid geometry and paint coordinates
Grids MUST use a transparent local viewport [0,width] x [0,height], top-left origin, X right and Y down; this viewport MUST define anchor bounds. Rectangular grids MUST enumerate vertical x=k*spacingX lines then horizontal y=k*spacingY lines. Dot grids MUST enumerate centers (i*spacingX,j*spacingY) inside the inclusive viewport in ascending j then i order, with the specified circle radius. Diagonal grids MUST use unit normals (1/sqrt(2),1/sqrt(2)), (1/sqrt(2),-1/sqrt(2)); isometric grids MUST use (1,0), (1/2,sqrt(3)/2), (1/2,-sqrt(3)/2). Each slanted family MUST include n dot p=k*spacing for every integer k intersecting the viewport with positive length, in listed-family then ascending-k order. Lines MUST be clipped to the viewport and endpoints ordered lexicographically by x then y, with existing dash semantics restarting at the first endpoint. Final stroke/fill coverage MUST clip to the viewport before visual transforms. Gradient coordinates MUST remain grid-local across all marks. Each mark MUST composite in enumeration order using existing premultiplied source-over semantics, including alpha accumulation at intersections.

#### Scenario: Match independent lattice oracles
- **WHEN** rectangular, diagonal, dot and isometric descriptors expand, including fractional spacing and non-square dimensions
- **THEN** coordinates, family angles, perpendicular spacing, boundary inclusion, endpoint order and mark enumeration match independently computed expected geometry

#### Scenario: Preserve paint and edge semantics
- **WHEN** a grid uses translucent or gradient paint, dashed strokes, border marks and an asymmetric transform or anchor
- **THEN** crossings accumulate alpha in canonical order, gradients do not restart per mark, dash origins follow endpoint order and coverage clips without moving the viewport origin

### Requirement: Bounded procedural expansion and evaluated behavior
Core MUST allow at most 4096 line segments of positive length or dot centers per grid, counted with checked finite arithmetic before expansion. Hidden/unused content MUST receive the same descriptor checks. Existing per-primitive command/subdivision/tolerance limits MUST apply, with at most 65536 flattened segments per grid and 1048576 per scene sample. Expanded component occurrences, visual layers, finite transformed bounds, scale-aware density and existing surface/aggregate memory limits MUST apply before allocation beyond a budget. Overflow or excess work MUST return INVALID_ARGUMENT without approximation or side effects. Evaluated instructions MUST carry resolved grid facts, viewport clipping, paint coordinates, inherited opacity/visibility, canonical order and half-open local clocks. Root and component-local grids MUST introduce no media or audio references, and downstream render consumers MUST NOT reconstruct grid semantics from persisted state.

#### Scenario: Enforce exact mark boundaries
- **WHEN** a dot grid has width/height 63 and spacingX/spacingY 1 versus width 64 and height 63 at the same spacing, or a line grid reaches 4096 versus 4097 marks
- **THEN** the 4096 boundary passes descriptor complexity validation, subject to all other budgets, while excessive counts fail before expansion even if hidden or offscreen

#### Scenario: Bound derived render work
- **WHEN** tiny positive spacing, non-finite derived arithmetic, repeated components, curve flattening or magnified surfaces exceed a grid/scene/work/memory limit
- **THEN** core rejects before excessive allocation, artifact preparation or backend execution and preserves current state and history

#### Scenario: Evaluate composed occurrences deterministically
- **WHEN** grids occur in root, nested/retimed components, groups and materialized drafts with static or supported animated transforms
- **THEN** occurrence identity, local clocks, paint space, viewport clipping and inherited state remain deterministic, and constant animation leaves equivalent static geometry unchanged
