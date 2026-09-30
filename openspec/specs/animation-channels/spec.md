# animation-channels Specification

## Purpose

Typed animation channel identity, bounds, compatibility, and deterministic sampling.

## Requirements

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

### Requirement: Bounded channel keyframes
Each channel MUST contain at most 1,000 keyframes at distinct, strictly increasing integer-millisecond offsets within its item's half-open duration, and each item MUST contain at most 64 channels with unique channel-and-target identity. Every accepted active scalar and component of a compound value MUST be finite and within its declared canonical bounds; path points MUST contain at most 4,096 points and gradient stops at most 32 stops before any later activation. Inactive values MUST be rejected before persistence regardless of their deferred numeric bounds. Channel input MUST reject unknown fields, raw renderer expressions, executable SVG, arbitrary paths, and network resources. Active keyframe timing MUST remain absolute item-local milliseconds; marker-relative keyframe timing MUST remain unavailable. Curves MUST be `hold`, `linear`, `cubic_bezier`, or `spring`; existing string `hold`/`linear` payloads MUST remain accepted. A cubic Bézier MUST contain finite `x1`, `y1`, `x2`, `y2`, each in [0, 1], with `x1 <= x2`. A spring MUST contain finite `mass` in [0.01, 100], `stiffness` in [0.01, 10000], `damping` in [0.01, 1000], and `initialVelocity` in [-100, 100]. Parameterized curves MUST be strict closed tagged records and MUST be rejected on a keyframe with no following segment. An optional channel `loop` MUST follow the `animation-loops` specification; a missing loop MUST retain prior held-first/held-last semantics.

#### Scenario: Reject invalid channel sequence
- **WHEN** an edit has duplicate channel identities, duplicate or descending times, an out-of-duration time, non-finite or out-of-bound value, or a named complexity limit overflow
- **THEN** core returns `INVALID_ARGUMENT` and leaves revision and history unchanged

#### Scenario: Reject deferred timing and malformed curves or loops
- **WHEN** a caller supplies a marker-relative keyframe time, malformed or unknown curve or loop variant, out-of-bound parameter, or parameterized curve on the terminal keyframe
- **THEN** the mutation fails with `INVALID_ARGUMENT` without interpreting the payload as an expression or changing revision and history

#### Scenario: Preserve existing curve payloads
- **WHEN** a caller writes existing string `hold` or `linear` channel keyframes without a loop
- **THEN** the values retain their existing wire representation and sampled results

### Requirement: Deterministic channel sampling
For every active channel without a loop, editor-core MUST sample item-local milliseconds with a held first value before its first keyframe, a held last value after its last keyframe, exact keyframe values at their timestamps, and type-appropriate interpolation between values. For a channel with a valid loop, editor-core MUST map item-local time according to the `animation-loops` specification before applying the same curve sampler. The starting keyframe's curve MUST determine the following segment. `hold` MUST return the start value; `linear` MUST use normalized elapsed segment time. Cubic Bézier MUST invert its monotone unit-interval X polynomial using exactly 40 bisection iterations, then evaluate its Y polynomial. Spring MUST use the closed-form solution of `mass * x'' + damping * x' + stiffness * (x - 1) = 0` over normalized segment time, with `x(0)=0` and `x'(0)=initialVelocity`, selecting underdamped, critically damped, or overdamped form from the discriminant. Exact segment endpoints MUST return the stored keyframe values. Intermediate spring overshoot MUST be allowed for position and gain, while every sampled property MUST be constrained to its existing canonical value bounds before evaluated-scene output; scale MUST use a positive lower bound of `0.000001`. Sampler outputs MUST be finite. Compound values MUST interpolate componentwise only when the channel declares that operation; channels with discrete topology MUST require `hold`. A static property MUST remain authoritative when its corresponding channel is absent. Preview and export MUST consume the same sampled evaluated scene. Fixed fixture samples MUST agree across supported platforms within `1e-9` absolute scalar error.

#### Scenario: Sample a channel at a boundary
- **WHEN** preview and export sample the same item and time at a keyframe or between two keyframes, including after valid loop phase mapping
- **THEN** both receive identical resolved values and retain the documented render tolerance

#### Scenario: Sample each parameterized curve
- **WHEN** an active scalar channel uses cubic Bézier or spring on a segment, including a reflected ping-pong segment
- **THEN** fixed timestamps resolve to the specified curve values within `1e-9`, including the exact start and end values and the damped spring regimes

#### Scenario: Bound spring overshoot
- **WHEN** a valid spring would exceed an active channel's canonical value range between keyframes
- **THEN** the evaluated value remains finite and is constrained to that property's documented bounds

#### Scenario: Preserve an unanimated project
- **WHEN** a project has no new channels or all channels omit loops
- **THEN** evaluation and rendered output remain equivalent to the pre-loop behavior

### Requirement: Transactional curve edits
The existing `set_animation_channels` standalone and `timeline_batch_edit` operations MUST accept the new curves through the same typed channel input. They MUST preserve expected-revision checks, atomic batch rollback, aliases for created item IDs, missing-reference errors, undo/redo, and deterministic reopen. Malformed curves MUST fail before persistence or rendering side effects with `INVALID_ARGUMENT`; missing items and stale revisions MUST retain their existing stable error codes and retryability.

#### Scenario: Edit a newly created item in one batch
- **WHEN** a batch creates a compatible item and addresses its alias with a valid parameterized curve
- **THEN** the channel edit commits atomically and undo/redo restores the exact typed curve and values

#### Scenario: Reject a curve after earlier batch edits
- **WHEN** a batch includes a malformed curve after other valid edits
- **THEN** the entire batch rolls back with no revision, history, or artifact change

#### Scenario: Preserve missing and stale failures
- **WHEN** a caller uses a missing item reference or stale expected revision
- **THEN** the operation returns the established stable typed error and changes no state

### Requirement: Render-accurate parameterized curve scalars

For every accepted cubic Bézier or spring channel, editor-core's backend scalar evaluation MUST agree with the canonical numeric sampler at fixed item-local timestamps within `1e-9` absolute error, including valid parameters immediately adjacent to the spring critical-damping boundary. A parameterized segment MUST retain exact stored endpoint values and existing canonical property bounds. Legacy `hold`, `linear`, and `set_keyframes` results MUST remain unchanged.

#### Scenario: Preserve a near-critical spring sample
- **WHEN** a valid spring has `mass=1`, `stiffness=100`, `damping=19.99999999999999`, and `initialVelocity=0` and a segment is sampled at normalized time `0.5`
- **THEN** the backend scalar differs from the core sampler by at most `1e-9` and does not treat the nonzero damped frequency as zero

#### Scenario: Compare curve regimes and boundaries
- **WHEN** fixed valid Bézier, underdamped, critical, and overdamped segments are sampled at their start, interior timestamps, and end
- **THEN** backend scalar results match canonical samples within `1e-9`, endpoints use stored values, and intermediate active-property results respect their canonical bounds

#### Scenario: Preserve old curves
- **WHEN** a project uses only `hold`, `linear`, or legacy keyframe easing
- **THEN** its sampled values and rendered output retain their existing behavior
