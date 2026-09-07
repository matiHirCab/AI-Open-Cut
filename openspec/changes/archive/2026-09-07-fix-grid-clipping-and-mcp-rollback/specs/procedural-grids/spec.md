## ADDED Requirements

### Requirement: Geometric fractional grid coverage
Grid coverage MUST be intersected geometrically with the local viewport before antialiasing and source-over composition. Stroke dashes and cap/join areas MUST resolve before clipping; dot fills MUST use their evaluated contours. Clipping MUST preserve orientation, mark order, local gradients, alpha, anchors and scale-aware tolerance. It MUST NOT multiply independently averaged viewport and mark coverage. Derived contours MUST be bounded by existing per-grid/scene segment and memory limits, with finite checked arithmetic before excessive allocation. Shape and SVG rendering MUST retain their existing paths.

#### Scenario: Preserve partially covered edges
- **WHEN** a width-1 opaque rectangular grid has dimensions 10.5 by 10 and spacing 10 by 20
- **THEN** pixel (10,5) has half coverage, approximately 128/255, equal to the same grid at width 11; corresponding bottom/corner, dot, dashed/capped, translucent and gradient cases retain their geometric coverage without exterior spill

#### Scenario: Preserve transformed clipping
- **WHEN** any grid pattern uses fractional dimensions with scaling or other supported transforms
- **THEN** resolved coverage is bounded by its local viewport before transformation, while paint coordinates and canonical composition order remain unchanged

#### Scenario: Bound generated coverage
- **WHEN** outline flattening or clipping reaches or exceeds a derived segment or memory budget
- **THEN** the exact boundary is accepted subject to other limits and excess returns INVALID_ARGUMENT during preflight before artifact preparation, preserving project/history and destination bytes
