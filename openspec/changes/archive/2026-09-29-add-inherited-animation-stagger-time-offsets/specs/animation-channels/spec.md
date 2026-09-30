## MODIFIED Requirements

### Requirement: Closed typed animation channels
Editor-core MUST define a closed, versioned channel catalog spanning transform (`transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.rotation_deg`, `transform.skew_x_deg`, `transform.skew_y_deg`, `transform.anchor_x`, `transform.anchor_y`, `transform.opacity`), media (`media.crop_x`, `media.crop_y`, `media.crop_width`, `media.crop_height`, `media.source_position_ms`, `media.playback_rate`), graphics (`graphic.path_points`, `graphic.path_trim`, `graphic.fill_color`, `graphic.stroke_color`, `graphic.stroke_width`, `graphic.gradient_stops`), effects (`effect.blur_radius`, `effect.glow_radius`, `effect.tint_color`, `effect.vignette_amount`, `effect.particle_amount`), and audio (`audio.gain_db`, `audio.pan`). The catalog MUST give each name one closed value tag: scalar except path points (bounded point list), gradient stops (bounded ordered color-stop list), and fill/stroke/tint colors (RGBA). Each entry MUST identify its prospective target kind and activation state. Active entries MUST have finite numeric bounds; inactive entries MUST identify their bounds as deferred until their activation milestone. Effect and graphic targets MUST use typed scoped references when activated. This milestone MUST activate only `transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.opacity`, and `audio.gain_db`; the remaining names are discoverable but inactive. Unsupported catalog entries MUST fail with `INVALID_ARGUMENT` before persistence rather than being stored as inert animation.

Active visual channels MUST target media with visual content, text, solid color, rectangle, shape, SVG, grid, group, or component-instance items whose `transform2d` is absent; position values are absolute composition pixels bounded to [-1,000,000, 1,000,000], scale values are absolute independent factors in (0, 100], and opacity is in [0, 1]. The missing channel's static legacy transform value MUST supply its axis or opacity. Active audio gain MUST target only media with audio, use finite decibels in [-96, 12], and multiply its base volume by `10^(gainDb/20)` during evaluated audio mixing. Active visual channels MUST reject audio-only media, repeater, caption, transition, or a target with `transform2d`; audio gain MUST reject media without audio. A channel MUST be accepted only when its target exists and the evaluator implements its semantics.

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

#### Scenario: Animate a supported parent
- **WHEN** a group or component instance without transform2d receives an active visual channel
- **THEN** core stores the typed channel and descendants inherit its sampled transform or opacity without activating deferred properties
