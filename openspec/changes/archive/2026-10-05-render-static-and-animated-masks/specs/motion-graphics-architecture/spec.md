## MODIFIED Requirements

### Requirement: Normative coordinate and compositing semantics
Motion-graphics evaluation MUST use a top-left coordinate origin with positive X rightward and positive Y downward, integer-millisecond half-open time intervals, explicit coordinate units, deterministic bottom-to-top layer ordering, the documented transform/mask/effect pipeline, premultiplied alpha, and linear-light compositing before output-color conversion. Schema33 masks MUST follow mask-models ownership/coordinate rules and mask-rendering raster/sample semantics before effects. Model/animation capability support SHALL NOT substitute for complete mask-rendering readiness.

#### Scenario: Resolve equal z-index layers
- **WHEN** multiple visual items in one track have the same explicit z-index
- **THEN** evaluation orders them by stable item array order and uses stable item ID only as a final deterministic tie-break for synthesized or otherwise equivalent order inputs

#### Scenario: Evaluate an inherited visual
- **WHEN** a visual has supported evaluated local crop, clip, masks, effects, transform, matte, opacity, blend mode, and ancestor transforms
- **THEN** evaluation applies source rasterization, crop and clip, declared masks, declared effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestor transforms, matte, inherited opacity, and destination blend in that order

#### Scenario: Distinguish current and future pipeline stages
- **WHEN** current visuals carry absent or authored schema 33 static/animated masks, while evaluated rendering does not activate track mattes or non-normal blend modes
- **THEN** authored masks execute approved mask-rendering semantics and absent masks/mattes remain explicit identity stages; destination blending is normal linear premultiplied source-over, and later matte/blend milestones retain their typed activation requirements
