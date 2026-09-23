## ADDED Requirements

### Requirement: Closed typed animation channels
Editor-core MUST define a closed, versioned channel catalog spanning transform (`transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.rotation_deg`, `transform.skew_x_deg`, `transform.skew_y_deg`, `transform.anchor_x`, `transform.anchor_y`, `transform.opacity`), media (`media.crop_x`, `media.crop_y`, `media.crop_width`, `media.crop_height`, `media.source_position_ms`, `media.playback_rate`), graphics (`graphic.path_points`, `graphic.path_trim`, `graphic.fill_color`, `graphic.stroke_color`, `graphic.stroke_width`, `graphic.gradient_stops`), effects (`effect.blur_radius`, `effect.glow_radius`, `effect.tint_color`, `effect.vignette_amount`, `effect.particle_amount`), and audio (`audio.gain_db`, `audio.pan`). The catalog MUST give each name one closed value tag: scalar except path points (bounded point list), gradient stops (bounded ordered color-stop list), and fill/stroke/tint colors (RGBA). Effect and graphic targets MUST use typed scoped references when activated. This milestone MUST activate only `transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.opacity`, and `audio.gain_db`; the remaining names are discoverable but inactive. Unsupported catalog entries MUST fail with `INVALID_ARGUMENT` before persistence rather than being stored as inert animation.

Active visual channels MUST target media with visual content, text, solid color, rectangle, shape, SVG, or grid items whose `transform2d` is absent; position values are absolute composition pixels bounded to [-1,000,000, 1,000,000], scale values are absolute independent factors in (0, 100], and opacity is in [0, 1]. The missing channel's static legacy transform value MUST supply its axis or opacity. Active audio gain MUST target only media with audio, use finite decibels in [-96, 12], and multiply its base volume by `10^(gainDb/20)` during evaluated audio mixing. Active visual channels MUST reject audio-only media, group, component instance, repeater, caption, transition, or a target with `transform2d`; audio gain MUST reject media without audio. A channel MUST be accepted only when its target exists and the evaluator implements its semantics.

#### Scenario: Accept supported channel value
- **WHEN** a caller sets a supported channel on a compatible item with a value of its declared type
- **THEN** the typed value is stored without a lossy numeric or string conversion

#### Scenario: Reject incompatible or unavailable channel
- **WHEN** a value has the wrong tag, a target does not support its channel, a target reference is missing, or a catalog entry has no active evaluator
- **THEN** core returns a stable non-retryable typed error without changing project state

#### Scenario: Reject incompatible transform representation
- **WHEN** a visual channel is assigned to an item with active `transform2d`, or `transform2d` is assigned to an item with visual channels
- **THEN** core returns `INVALID_ARGUMENT` and leaves the existing transform and channels unchanged

#### Scenario: Resolve audio gain
- **WHEN** audio gain is sampled at a supported time on media containing audio
- **THEN** evaluated audio uses its decibel-to-linear multiplier with the item's existing volume, mute, fade, and ducking semantics

### Requirement: Bounded channel keyframes
Each channel MUST contain at most 1,000 keyframes at distinct, strictly increasing integer-millisecond offsets within its item's half-open duration, and each item MUST contain at most 64 channels with unique channel-and-target identity. Every scalar and component of a compound value MUST be finite and within the corresponding static property's canonical bounds; path points MUST contain at most 4,096 points and gradient stops at most 32 stops. Channel input MUST reject unknown fields, raw renderer expressions, executable SVG, arbitrary paths, and network resources. This milestone MUST use only `hold` and `linear` curves and absolute item-local time; later curve and marker variants remain unavailable.

#### Scenario: Reject invalid channel sequence
- **WHEN** an edit has duplicate channel identities, duplicate or descending times, an out-of-duration time, non-finite or out-of-bound value, or a named complexity limit overflow
- **THEN** core returns `INVALID_ARGUMENT` and leaves revision and history unchanged

#### Scenario: Reject deferred timing and curves
- **WHEN** a caller supplies a marker time, loop, cubic Bézier, spring, or unknown curve variant
- **THEN** the mutation fails with `INVALID_ARGUMENT` without interpreting the payload as an expression

### Requirement: Deterministic channel sampling
For every active channel, editor-core MUST sample item-local milliseconds with a held first value before its first keyframe, a held last value after its last keyframe, exact keyframe values at their timestamps, and `hold` or type-appropriate `linear` interpolation between values. Compound values MUST interpolate componentwise only when the channel declares that operation; channels with discrete topology MUST require `hold`. A static property MUST remain authoritative when its corresponding channel is absent. Preview and export MUST consume the same sampled evaluated scene.

#### Scenario: Sample a channel at a boundary
- **WHEN** preview and export sample the same item and time at a keyframe or between two keyframes
- **THEN** both receive identical resolved values and retain the documented render tolerance

#### Scenario: Preserve an unanimated project
- **WHEN** a project has no new channels
- **THEN** evaluation and rendered output remain equivalent to the pre-milestone behavior
