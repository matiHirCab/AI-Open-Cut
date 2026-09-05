## ADDED Requirements

### Requirement: Canonical group ancestor evaluation
Core MUST evaluate each visual's local affine transform followed by ancestors nearest outward, yielding Mroot ... Mparent Mlocal for column vectors. Group normalized position and anchor MUST resolve against root composition dimensions; groups SHALL have no measured child bounding box. Ancestor opacity MUST multiply per descendant, without isolated group compositing. A descendant's visual interval MUST intersect all ancestor half-open intervals and its visibility MUST include all ancestor item/track visibility. Empty intersections SHALL emit no visual instruction. Child timing SHALL remain absolute root milliseconds, with no offset or retiming from a group. Audio timing/gain/visibility SHALL retain existing behavior independently of visual parents. Groups MUST NOT reorder children or create drawable instructions.

#### Scenario: Evaluate nested transforms independently
- **WHEN** asymmetric visuals use nested translation, noncentral anchors, independent scales, skew, rotation, and opacity
- **THEN** evaluated corners agree with an independent matrix oracle within 1e-9 pixels and opacity equals the product of local and ancestor opacity

#### Scenario: Apply visibility and intervals
- **WHEN** a child overlaps only part of ancestor intervals or any ancestor or ancestor track is hidden
- **THEN** visuals use the interval intersection or disappear while existing media audio behavior remains unchanged

#### Scenario: Preserve ordering and legacy behavior
- **WHEN** parented visuals span tracks or unparented legacy scenes are evaluated repeatedly
- **THEN** existing track/zIndex/stackOrder/ID order remains intact, legacy scenes remain equivalent, and evaluation never mutates project or history

### Requirement: Bounded derived group geometry
Core MUST validate all graph and local values before resource work, and finite composed matrices/coordinates and existing 16384-pixel dimension and 16777216-pixel area limits before raster allocation, workspace writes, or renderer execution. Necessary existing bounded read-only font/media measurement SHALL remain path-safe and outside scene semantics. No paths, backend expressions, or persisted references SHALL enter EvaluatedScene. Existing missing-asset precedence SHALL remain unchanged.

#### Scenario: Reject composed overflow
- **WHEN** individually valid transforms combine into non-finite or oversized derived geometry
- **THEN** core returns INVALID_ARGUMENT before render side effects instead of clamping or dropping ancestors

### Requirement: Legacy anchors and bounded animated sampling
Core MUST preserve legacy styled-text anchors through static and animated parenting. Explicit Transform2D anchors SHALL remain authoritative. Core MUST validate finite composed coordinates and per-object raster limits before clipping, but MUST NOT reject motion solely because its movement envelope exceeds those limits. Animated sampling SHALL intersect the composition and use deterministic non-overlapping tiles no larger than 4096 by 4096, finalized before artifact creation. Empty sampling intersections SHALL emit no visual work or change audio.

#### Scenario: Preserve anchored text through identity parenting
- **WHEN** styled text uses any of the nine anchors with static or animated position and scale
- **THEN** identity parenting preserves its placement and nested ancestors apply after its local anchor

#### Scenario: Render long-distance motion with bounded sampling
- **WHEN** a small visual travels offscreen and returns across a movement envelope larger than the raster limits
- **THEN** every render intent succeeds with bounded seam-free sampling, while genuinely oversized object geometry still fails before side effects
