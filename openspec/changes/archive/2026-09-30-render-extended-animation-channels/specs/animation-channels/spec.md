## MODIFIED Requirements

### Requirement: Closed typed animation channels
Editor-core MUST define a closed, versioned channel catalog spanning transform (`transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.rotation_deg`, `transform.skew_x_deg`, `transform.skew_y_deg`, `transform.anchor_x`, `transform.anchor_y`, `transform.opacity`), media (`media.crop_x`, `media.crop_y`, `media.crop_width`, `media.crop_height`, `media.source_position_ms`, `media.playback_rate`), graphics (`graphic.path_points`, `graphic.path_trim`, `graphic.fill_color`, `graphic.stroke_color`, `graphic.stroke_width`, `graphic.gradient_stops`), effects (`effect.blur_radius`, `effect.glow_radius`, `effect.tint_color`, `effect.vignette_amount`, `effect.particle_amount`), and audio (`audio.gain_db`, `audio.pan`). The catalog MUST give each name one closed value tag: scalar except path points (bounded point list), gradient stops (bounded ordered color-stop list), and fill/stroke/tint colors (RGBA). Each entry MUST identify its prospective target kind and activation state. Active entries MUST have finite numeric bounds; inactive entries MUST identify their bounds as deferred until their activation milestone. Effect and graphic targets MUST use typed scoped references when activated. Core MUST activate `transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.opacity`, `audio.gain_db`, and the schema-27 subset defined by Bounded extended visual channel targets; the remaining names are discoverable but inactive. Unsupported catalog entries MUST fail with `INVALID_ARGUMENT` before persistence rather than being stored as inert animation.

The original five legacy visual channels MUST target media with visual content, text, solid color, rectangle, shape, SVG, grid, group, or component-instance items whose `transform2d` is absent; position values are absolute composition pixels bounded to [-1,000,000, 1,000,000], scale values are absolute independent factors in (0, 100], and opacity is in [0, 1]. The missing channel's static legacy transform value MUST supply its axis or opacity. Active audio gain MUST target only media with audio, use finite decibels in [-96, 12], and multiply its base volume by `10^(gainDb/20)` during evaluated audio mixing. The original five legacy visual channels MUST reject audio-only media, repeater, caption, transition, or a target with `transform2d`; audio gain MUST reject media without audio. Extended visual channels MUST follow Bounded extended visual channel targets. A channel MUST be accepted only when its target exists and the evaluator implements its semantics.

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

### Requirement: Bounded extended candidate certification
Before publishing a candidate containing extended channels, crop or effects, core MUST certify coupled sample geometry and expanded raster/effect work using deterministic canonical interval analysis. The cumulative budget MUST be 65536 interval-analysis nodes for the final candidate including expanded and retained content; one node MUST represent one bounded time interval for one visual occurrence. Occurrences MUST be processed in canonical order and subdivision MUST visit the left interval before the right. A proved unsafe bound or unresolved safety after budget exhaustion MUST return non-retryable `INVALID_ARGUMENT` before mutation/publication, preserving project, history, draft, revision, aliases and managed-resource bytes. This certification MAY conservatively reject newly supported work whose safety cannot be established within the budget. Legacy candidates without extended properties MUST retain their existing behavior. The budget and conservative rejection semantics MUST be recorded in the canonical extended-visual contract.

#### Scenario: Reject unsafe intermediate coupled crop
- **WHEN** a crop of width 0.5 animates X from 0 to 0.5 over 500 milliseconds using a spring with mass 1, stiffness 100, damping 1 and initialVelocity 0
- **THEN** core rejects the candidate before publication because intermediate X plus width exceeds 1 despite valid endpoints

#### Scenario: Certify correlated safe channels
- **WHEN** correlated channels retain valid coupled geometry throughout their canonical sample envelope within the analysis budget
- **THEN** core accepts the candidate and all render intents consume the certified sampler results

#### Scenario: Bound deterministic analysis work
- **WHEN** safety remains unresolved after the final candidate consumes 65536 interval-analysis nodes
- **THEN** core returns `INVALID_ARGUMENT` deterministically without additional analysis or changing authoritative state, including during alias-aware batches and draft edits

#### Scenario: Preserve legacy candidate behavior
- **WHEN** a candidate contains no extended properties
- **THEN** this certification budget introduces no additional rejection or change to its previous output and persistence behavior

### Requirement: Bounded extended visual channel targets
Core MUST activate rotation, crop X/Y/width/height, path points/trim, gradient stops, blur radius, glow radius, tint color, and vignette amount only on schema-27 compatible targets. Rotation MUST be targetless, finite in [-36000,36000], supported on the existing visual target kinds, and sampled as absolute degrees without shortest-arc normalization. It MUST replace only transform2d.rotationDeg when transform2d exists; otherwise it MUST preserve the legacy position/scale/anchor with zero static rotation. All existing five legacy visual channels MUST retain their transform2d exclusion.

Crop channels MUST be targetless on visual media only and use normalized oriented-source coordinates: X/Y in [0,1], width/height in (0,1], with X+width <= 1 and Y+height <= 1. An optional static `crop` MUST be a closed `{x,y,width,height}` record with the same bounds and default `{x:0,y:0,width:1,height:1}`. Absent crop channels MUST use the static component. Cropping MUST preserve the item's authored destination size/anchor and existing fit behavior while selecting source content before scaling. Audio-only media and non-media crop targets MUST fail with `INVALID_ARGUMENT`.

Path points and trim MUST use a scoped `graphic_geometry` target on a shape whose geometry is a structured path. Path points MUST list every authored command coordinate in command order (move/line endpoints, quadratic control then endpoint, cubic controls then endpoint, close contributes none), with exactly the static path's point count, at most 4096 points, and finite coordinates in [-1000000,1000000]. Channels MUST preserve command tags, subpath boundaries, closure, and fill rule. Trim MUST be a finite visible-prefix fraction in [0,1], default 1, applied independently to each subpath. Gradient stops MUST target an existing linear/radial fill or stroke gradient through `graphic_fill` or `graphic_stroke`; each keyframe MUST match the static stop count with 2 through 32 ordered stops, first offset 0, last offset 1, finite offsets/components in [0,1], and strictly increasing offsets. Solid/missing paints, arbitrary SVG sub-elements, topology mismatches, and excessive counts MUST fail without normalization.

Effect channels MUST target an existing scoped effect ID on the owning item: blur radius to gaussian_blur, glow radius to glow, tint color to color_tint, and vignette amount to vignette, with the static effect bounds. Unanimated components MUST retain their static values. Channel identity MUST include property and typed target; equal properties on distinct valid effects MUST be allowed, exact duplicate identities MUST fail. Missing target IDs MUST retain the stable missing-reference error; wrong kind/scope/incompatible type MUST return non-retryable `INVALID_ARGUMENT` without mutation. The other deferred channels MUST remain unavailable.

#### Scenario: Accept every extended property with static fallback
- **WHEN** a schema-27 compatible item receives any supported extended channel, including rotation on transform2d and two distinct effect targets
- **THEN** core stores its exact typed value and resolves only that property while preserving unanimated static components

#### Scenario: Reject bad targets and coupled geometry
- **WHEN** channels address missing/wrong-scope/incompatible targets, duplicate identities, invalid crop edges, changed path topology, malformed gradients, or exceeded bounds
- **THEN** core returns the stable typed failure and preserves revision, history, resources and artifacts

### Requirement: Deterministic compound visual sampling
Extended channels MUST use existing item-local clocks, curve endpoints, inherited timing and loops. Scalar rotation/crop/trim/effect channels MUST use the canonical scalar sampler; radius, fraction and normalized channels MUST clamp intermediate overshoot to their bounds. Path coordinates MUST interpolate componentwise for equal topology. Gradient offsets MUST interpolate componentwise while colors and tint MUST interpolate in premultiplied linear light, converting authored unassociated sRGB on input/output; zero alpha MUST resolve zero RGB on unpremultiplication. Compound hold MUST return the exact stored value, exact timestamps MUST return exact authored keyframes, and scalar/component samples MUST remain finite. Path coordinates MUST clamp to their coordinate bounds; compound color/offset samples MUST clamp to [0,1]. A crop violating coupled bounds or gradient losing strict stop order after sampling MUST fail with `INVALID_ARGUMENT`, without sorting, edge repair, or output publication. The positive lower bound for interpolated crop width/height MUST be 0.000001; exact authored keyframes and hold samples MUST preserve their stored positive values. Every reachable sample MUST remain subject to canonical geometry/raster complexity preflight before commit; sampled failures MUST also fail closed before render output. Compound scalar-equivalent fixture components MUST agree across platforms within 1e-9.

#### Scenario: Sample structured values across curves and loops
- **WHEN** equal-topology paths, matched gradients and effects use hold, linear, cubic Bézier or spring curves with repeat/ping-pong loops
- **THEN** canonical fixed samples preserve exact endpoints, color semantics, bounds, topology and clock agreement across all output intents

#### Scenario: Reject unsafe intermediate samples
- **WHEN** individually valid keyframes produce invalid coupled crop geometry, reversed gradient ordering or excessive expanded render work
- **THEN** canonical candidate preflight rejects the edit and render preflight rejects persisted work without publication

### Requirement: Transactional extended visual edits
Existing standalone and alias-aware batch channel/visual-property edits MUST accept valid extended channels, crop and effects in one atomic revision and undo step. Validation MUST preserve expected-revision precedence, locked-track and missing-reference errors, and ordered batch alias semantics. Retained current/history/draft state MUST be validated, including hidden/unused component content. Undo, redo, clear-channel edits and reopen MUST restore authored properties and fallback semantics deterministically; failed edits MUST leave state/history/draft/resource bytes unchanged. No new top-level operation or transport-side semantic validator MUST be introduced.

#### Scenario: Create and animate with aliases
- **WHEN** a batch creates compatible media or a shape, assigns static crop/effects, then targets its creation alias with extended channels
- **THEN** all changes commit atomically and undo/redo/reopen preserve IDs, values, timing and samples

#### Scenario: Roll back malformed and conflicting edits
- **WHEN** a standalone, batch or draft edit contains malformed extended data, a stale revision, locked target or missing reference
- **THEN** it returns the existing stable error/retryability and publishes no partial candidate or aliases

#### Scenario: Clear channels without changing static state
- **WHEN** extended channels are removed from an animated item
- **THEN** evaluation returns its authored static transform, crop, geometry, gradient and effects in one undoable edit
