## MODIFIED Requirements

### Requirement: Normative coordinate and compositing semantics
Motion-graphics evaluation MUST use top-left origin with positive X right/Y down, integer-millisecond half-open intervals, explicit units, deterministic bottom-to-top ordering, documented mask/effect/matte/opacity/shutter/blend pipeline, premultiplied alpha and linear-light compositing before output conversion. Active masks MUST retain mask-models coordinate ownership and mask-rendering raster/sample rules before effects. Matte references MUST retain scoped finite DAG/occurrence/exact temporal semantics and isolated transparent provider coverage independent of destination and blend. Schema35 eligible visual occurrences including Caption MUST apply their closed seven-mode blend once after source assembly/shutter averaging against the existing opaque-black-backed current destination; normal/default SHALL remain exact. Model/animation capability support MUST NOT substitute for complete mask/matte/blend readiness.

The sole inward evaluated composition-resource owner MUST publish shared resource admission independently of the optional real matte DAG, together with canonical immutable completed-copy owner/mode facts for pure execution. Existing glyph/font/cursor/live-capacity rules MUST be reused without parallel policy or reverse authored/transport traversal; private owner inventory and architecture evidence MUST be updated together.

#### Scenario: Resolve equal z-index layers
- **WHEN** multiple visual items in one track share explicit z-index
- **THEN** stable item array order determines paint order and stable item ID remains only the final deterministic tie-break for synthesized/equivalent inputs

#### Scenario: Evaluate an inherited visual
- **WHEN** a visual has supported crop/clip/masks/effects/transforms/matte/opacity/blend and ancestors
- **THEN** source rasterization, crop/clip, masks, effects, local affine, nearest-to-outer ancestors, matte, inherited opacity/transition, owning shutter average and final declared destination blend occur in that order

#### Scenario: Distinguish current and future pipeline stages
- **WHEN** current visuals carry absent or authored schema33/34/35 static/animated masks and schema34/35 typed track mattes, while schema35 activates the seven typed blend selections
- **THEN** authored masks and typed mattes execute their approved semantics and absent masks/mattes remain explicit identity stages; normal destination blending remains exact linear premultiplied source-over, and non-normal blending executes only its approved typed activation after owning source shutter averaging

#### Scenario: Preserve active defaults and complete readiness
- **WHEN** visuals carry active masks/mattes/blend selections or omit those fields
- **THEN** authored stages execute only approved typed semantics, absent mask/matte stages remain identity and absent/normal blend retains exact source-over/opaque-black behavior; incomplete backend support cannot degrade silently

#### Scenario: Keep scope occurrence and paint order independent
- **WHEN** matte providers lie above/below recipients or repeated local IDs occur in differently transformed component instances
- **THEN** scoped dependency execution prepares isolated coverage without blend, then each direct eligible occurrence blends once in unchanged final destination paint order

#### Scenario: Keep Caption source assembly as one blend unit
- **WHEN** one Caption contains multiple lines, glyphs and background-box fragments
- **THEN** existing source-local assembly produces one owning occurrence plane and its declared mode applies once without per-line/glyph/background blending or new group isolation
