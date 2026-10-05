## MODIFIED Requirements

### Requirement: Normative coordinate and compositing semantics
Motion-graphics evaluation MUST use a top-left coordinate origin with positive X rightward and positive Y downward, integer-millisecond half-open time intervals, explicit coordinate units, deterministic bottom-to-top layer ordering, the documented transform/mask/effect pipeline, premultiplied alpha, and linear-light compositing before output-color conversion. Schema 32 authored mask metadata MUST follow mask-models ownership/coordinate rules while remaining visually inactive; the rendered mask stage MUST stay identity until the separately approved rasterization milestone. Authored metadata support SHALL NOT be reported as mask-rendering readiness.

#### Scenario: Resolve equal z-index layers
- **WHEN** multiple visual items in one track have the same explicit z-index
- **THEN** evaluation orders them by stable item array order and uses stable item ID only as a final deterministic tie-break for synthesized or otherwise equivalent order inputs

#### Scenario: Evaluate an inherited visual
- **WHEN** a visual has supported evaluated local crop, clip, masks, effects, transform, matte, opacity, blend mode, and ancestor transforms
- **THEN** evaluation applies source rasterization, crop and clip, declared masks, declared effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestor transforms, matte, inherited opacity, and destination blend in that order

#### Scenario: Distinguish current and future pipeline stages
- **WHEN** current visuals carry absent or authored schema 32 mask metadata, while evaluated rendering does not activate masks, track mattes or non-normal blend modes
- **THEN** mask/matte stages are explicit identity stages and destination blending is normal linear premultiplied source-over; typed mask metadata is editable/persisted but its rasterization and animation await the separate approved milestone, and later matte/blend milestones retain their typed activation requirements
