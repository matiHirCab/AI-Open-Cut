## MODIFIED Requirements

### Requirement: Closed typed animation channels
Editor-core MUST define a closed, versioned channel catalog spanning transform (`transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.rotation_deg`, `transform.skew_x_deg`, `transform.skew_y_deg`, `transform.anchor_x`, `transform.anchor_y`, `transform.opacity`), media (`media.crop_x`, `media.crop_y`, `media.crop_width`, `media.crop_height`, `media.source_position_ms`, `media.playback_rate`), graphics (`graphic.path_points`, `graphic.path_trim`, `graphic.fill_color`, `graphic.stroke_color`, `graphic.stroke_width`, `graphic.gradient_stops`), effects (`effect.blur_radius`, `effect.glow_radius`, `effect.tint_color`, `effect.vignette_amount`, `effect.particle_amount`), audio (`audio.gain_db`, `audio.pan`), and masks (`mask.path_points`, `mask.paint_color`, `mask.gradient_stops`, `mask.feather_px`, `mask.expansion_px`, `mask.transform.position_x`, `mask.transform.position_y`, `mask.transform.scale_x`, `mask.transform.scale_y`, `mask.transform.anchor_x`, `mask.transform.anchor_y`, `mask.transform.rotation_deg`, `mask.transform.skew_x_deg`, `mask.transform.skew_y_deg`, `mask.transform.opacity`). The catalog MUST give each name one closed value tag: scalar except path points (bounded point list), gradient stops (bounded ordered color-stop list), and fill/stroke/tint/mask paint colors (RGBA). Each entry MUST identify its prospective target kind and activation state. Active entries MUST have finite numeric bounds; inactive entries MUST identify their bounds as deferred until their activation milestone. Effect, graphic and mask targets MUST use typed scoped references when activated. Core MUST activate `transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.opacity`, `audio.gain_db`, the schema-27 subset defined by Bounded extended visual channel targets, and the schema 33 subset defined by Bounded mask channel targets; the remaining names are discoverable but inactive. Unsupported catalog entries MUST fail with `INVALID_ARGUMENT` before persistence rather than being stored as inert animation.

The original five legacy visual channels MUST target media with visual content, text, solid color, rectangle, shape, SVG, grid, group, or component-instance items whose `transform2d` is absent; position values are absolute composition pixels bounded to [-1,000,000, 1,000,000], scale values are absolute independent factors in (0, 100], and opacity is in [0, 1]. The missing channel's static legacy transform value MUST supply its axis or opacity. Active audio gain MUST target only media with audio, use finite decibels in [-96, 12], and multiply its base volume by `10^(gainDb/20)` during evaluated audio mixing. The original five legacy visual channels MUST reject audio-only media, repeater, caption, transition, or a target with `transform2d`; audio gain MUST reject media without audio. Extended visual channels MUST follow Bounded extended visual channel targets; mask channels MUST follow Bounded mask channel targets without activating unrelated deferred names. A channel MUST be accepted only when its target exists and the evaluator implements its semantics.

#### Scenario: Accept supported channel value
- **WHEN** a caller sets a supported channel on a compatible item with a value of its declared type
- **THEN** the typed value is stored without a lossy numeric or string conversion

#### Scenario: Reject incompatible or unavailable channel
- **WHEN** a value has the wrong tag, a target does not support its channel, a target reference is missing, or a catalog entry has no active evaluator
- **THEN** core returns a stable non-retryable typed error without changing project state

#### Scenario: Reject incompatible transform representation
- **WHEN** one of the original five legacy visual channels is assigned to an item with active `transform2d`, or `transform2d` is assigned to an item with any of the original five legacy visual channels
- **THEN** core returns `INVALID_ARGUMENT` and leaves the existing transform and channels unchanged

#### Scenario: Resolve audio gain
- **WHEN** audio gain is sampled at a supported time on media containing audio
- **THEN** evaluated audio uses its decibel-to-linear multiplier with the item's existing volume, mute, fade, and ducking semantics

#### Scenario: Animate a supported parent
- **WHEN** a group or component instance without transform2d receives an active visual channel
- **THEN** core stores the typed channel and descendants inherit its sampled transform or opacity without activating deferred properties

## ADDED Requirements

### Requirement: Bounded mask channel targets
Every new mask property MUST require a target {kind:mask,scope:root or component:<definition-id>,id:owning-item mask ID}. Resolution MUST stay within the addressed item's composition and mask vector. Missing ID MUST return ITEM_NOT_FOUND; wrong kind/scope/value/paint type MUST return INVALID_ARGUMENT. Channel identity MUST remain(property,target), with exact duplicates rejected and equal properties on distinct mask IDs allowed. Reordering masks MUST preserve target identity; removal/renaming of a referenced mask without clearing/replacing its channels in the final candidate MUST reject atomically.

mask.path_points MUST use existing path_points values in canonical command coordinate order, preserve static command/subpath/closure/fillRule topology, match static point count at most 4096 and coordinates±1000000. mask.paint_color MUST use rgba only on solid Paint; mask.gradient_stops MUST use gradient_stops only on linear/radial Paint with matching static count 2…32, offsets0/1 at endpoints, strict increasing order and components 0…1. Static model paints with33…64 stops MUST remain valid but SHALL NOT accept gradient animation under the unchanged compound-value cap. mask.feather_px MUST use scalar 0…128; mask.expansion_px scalar−128…128. The mask.transform position_x/y,scale_x/y,anchor_x/y,rotation_deg,skew_x/y_deg,opacity properties MUST use scalars with static Transform2D unit/bounds, including positive scale≤100. Unit/inversion/operation/channel/source/paint type/gradient geometry/path topology SHALL NOT be animated. Existing64-channel/1000-keyframe/curve/clock/loop bounds MUST apply unchanged. No arbitrary property strings or resource inputs SHALL be interpreted.

#### Scenario: Target every supported mask property
- **WHEN** compatible owning masks receive typed path, paint, feather, expansion and every transform component channel, including identical properties on two masks
- **THEN** core stores exact IDs/values and absent channels preserve each static component without modifying owner transform exclusions

#### Scenario: Reject missing incompatible or dangling targets atomically
- **WHEN** an edit uses a missing/cross-scope/wrong-kind mask, wrong paint/tag/topology/count, duplicate identity or removes a referenced mask in a final candidate
- **THEN** existing typed failure rejects standalone/batch/draft publication with unchanged state/history/revision/resources

### Requirement: Deterministic complete mask sampling
Every mask channel MUST use canonical SampleTime, retained offsets, fractional inherited clocks, curves, loop phase and exact integer endpoint selection once. Exact timestamps/holds MUST return stored values; path points and gradient offsets MUST interpolate componentwise, rgba/gradient colors in premultiplied linear light with zero-alpha RGB zero. Intermediate scalar/component overshoot MUST clamp only canonical bounds; scale MUST use intermediate floor 0.000001 while exact keys/holds preserve stored positive values. Gradient order failure SHALL NOT be sorted/repaired. Sampled analytic path bounds MUST resolve mask anchor; sampled static/unit/paint topology MUST remain immutable. Every actual and continuously reachable mask sample MUST satisfy canonical renderability/support/work/live-memory certification; unresolved intervals MUST fail atomically. Fixed compound scalar-equivalent samples MUST agree across platforms within 1e−9.

#### Scenario: Preserve integer fractional and shutter clocks
- **WHEN** mask channels use large integer endpoints beyond2^53, fractional component/repeater/stagger clocks, retained offsets and hold/linear/Bezier/spring/repeat/ping-pong/shutter samples
- **THEN** canonical actual consumers preserve endpoint/interior values, sampled mask semantics and all four intent agreement without rounding source clocks or applying offsets twice

#### Scenario: Reject unsafe intermediate mask samples
- **WHEN** individually valid keys produce reversed gradient offsets, singular/unsafe affine inverses or excessive density/support/work
- **THEN** canonical final-candidate/render preflight rejects before authoritative or artifact side effects without repairing keys
