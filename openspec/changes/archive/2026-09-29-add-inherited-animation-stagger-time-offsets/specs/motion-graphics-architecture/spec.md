## MODIFIED Requirements

### Requirement: Canonical group ancestor evaluation
Core MUST evaluate each visual's local affine transform followed by ancestors nearest outward, yielding Mroot ... Mparent Mlocal for column vectors. Group normalized position and anchor MUST resolve against root composition dimensions; groups SHALL have no measured child bounding box. Ancestor opacity MUST multiply per descendant, without isolated group compositing. A descendant's visual interval MUST intersect all ancestor half-open intervals and its visibility MUST include all ancestor item/track visibility. Empty intersections SHALL emit no visual instruction. With zero or absent staggerMs, child timing SHALL remain absolute containing-scope milliseconds. With staggerMs, each direct visual child branch SHALL use the ranked local-clock delay defined by inherited-animation-timing while parent interval and channel sampling remain unshifted. Audio timing/gain/visibility SHALL retain existing behavior independently of visual parents. Groups MUST NOT reorder children or create drawable instructions.

#### Scenario: Evaluate nested transforms independently
- **WHEN** asymmetric visuals use nested translation, noncentral anchors, independent scales, skew, rotation, and opacity
- **THEN** evaluated corners agree with an independent matrix oracle within 1e-9 pixels and opacity equals the product of local and ancestor opacity

#### Scenario: Apply visibility and intervals
- **WHEN** a child overlaps only part of ancestor intervals or any ancestor or ancestor track is hidden
- **THEN** visuals use the interval intersection or disappear while existing media audio behavior remains unchanged

#### Scenario: Preserve ordering and legacy behavior
- **WHEN** parented visuals span tracks or unparented legacy scenes are evaluated repeatedly
- **THEN** existing track/zIndex/stackOrder/ID order remains intact, legacy scenes remain equivalent, and evaluation never mutates project or history

#### Scenario: Inherit animated ancestors and staggered clocks
- **WHEN** a supported parent channel animates a nested group and its staggered children
- **THEN** each child uses the sampled parent matrix and opacity with its ranked local clock, while existing stacking and parent interval clipping remain stable
