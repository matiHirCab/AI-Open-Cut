## ADDED Requirements

### Requirement: Hierarchical instance transforms and stacking
Core MUST flatten instances into the private owned EvaluatedScene without persisted references, paths or backend expressions. Local coordinates MUST use definition dimensions; instance position MUST resolve in the containing composition and instance normalized anchor against referenced definition dimensions. Matrices MUST compose outward through local groups, instance transforms and outer groups, multiplying opacity per descendant without implicit fit scaling or isolated compositing. Instance content MUST occupy one contiguous block at its parent track/zIndex/stackOrder/ID position, recursively using the same comparator within each definition. Local z-index MUST NOT escape the instance block. Only root output clipping SHALL apply; component dimensions SHALL NOT introduce an implicit clipping mask. Existing flat scene/group rules SHALL remain unchanged outside instance boundaries.

#### Scenario: Resolve nested coordinates and order
- **WHEN** instances with different composition sizes, noncentral anchors, skew/rotation and groups overlap root items and other occurrences
- **THEN** corners match an independent matrix oracle within 1e-9 pixels, opacity is the ancestor product and exact visual order preserves each occurrence block

#### Scenario: Preserve shared evaluation and overflow behavior
- **WHEN** the same revision is evaluated repeatedly or composed geometry exceeds existing limits
- **THEN** valid scenes are equal without project/history mutation and overflow uses existing typed failure before raster allocation or artifact preparation
