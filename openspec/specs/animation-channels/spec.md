# animation-channels Specification

## Purpose

Typed animation channel identity, bounds, compatibility, and deterministic sampling.

## Requirements

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

### Requirement: Bounded channel keyframes
Each channel MUST contain at most 1,000 keyframes at distinct, strictly increasing integer-millisecond offsets within its effective source duration: the retained clock's `sourceDurationMs` when present, otherwise its item's half-open duration, and each item MUST contain at most 64 channels with unique channel-and-target identity. Every accepted active scalar and component of a compound value MUST be finite and within its declared canonical bounds; path points MUST contain at most 4,096 points and gradient stops at most 32 stops before any later activation. Inactive values MUST be rejected before persistence regardless of their deferred numeric bounds. Channel input MUST reject unknown fields, raw renderer expressions, executable SVG, arbitrary paths, and network resources. Active keyframe timing MUST remain absolute integer-millisecond offsets in the original source item-local timeline; retained keys MAY lie outside an edited item’s active window only when a valid retained clock identifies that source duration, and omitted clocks MUST retain the original item-duration validation; marker-relative keyframe timing MUST remain unavailable. Curves MUST be `hold`, `linear`, `cubic_bezier`, or `spring`; existing string `hold`/`linear` payloads MUST remain accepted. A cubic Bézier MUST contain finite `x1`, `y1`, `x2`, `y2`, each in [0, 1], with `x1 <= x2`. A spring MUST contain finite `mass` in [0.01, 100], `stiffness` in [0.01, 10000], `damping` in [0.01, 1000], and `initialVelocity` in [-100, 100]. Parameterized curves MUST be strict closed tagged records and MUST be rejected on a keyframe with no following segment. An optional channel `loop` MUST follow the `animation-loops` specification; a missing loop MUST retain prior held-first/held-last semantics.

#### Scenario: Reject invalid channel sequence
- **WHEN** an edit has duplicate channel identities, duplicate or descending times, a time outside its effective source duration, non-finite or out-of-bound value, or a named complexity limit overflow
- **THEN** core returns `INVALID_ARGUMENT` and leaves revision and history unchanged

#### Scenario: Reject deferred timing and malformed curves or loops
- **WHEN** a caller supplies a marker-relative keyframe time, malformed or unknown curve or loop variant, out-of-bound parameter, or parameterized curve on the terminal keyframe
- **THEN** the mutation fails with `INVALID_ARGUMENT` without interpreting the payload as an expression or changing revision and history

#### Scenario: Preserve existing curve payloads
- **WHEN** a caller writes existing string `hold` or `linear` channel keyframes without a loop
- **THEN** the values retain their existing wire representation and sampled results

#### Scenario: Validate retained source keys beyond an edited window
- **WHEN** a channel retains valid original source keys beyond a split or trimmed item's active duration and a bounded source clock identifies the original source duration
- **THEN** core accepts keys inside that source duration with unchanged ordering/value/curve/count constraints, while the same out-of-item-duration keys without a retained clock fail with INVALID_ARGUMENT

### Requirement: Deterministic channel sampling
Editor-core MUST derive source time by adding the optional retained clock offset to item-local milliseconds exactly once, retaining fractional inherited time and checked integer arithmetic; negative source time MUST hold the first key. Omitted clocks MUST preserve the previous item-local clock. For every active channel without a loop, editor-core MUST sample this source time with a held first value before its first keyframe, a held last value after its last keyframe, exact keyframe values at their timestamps, and type-appropriate interpolation between values. For a channel with a valid loop, editor-core MUST map source time according to the `animation-loops` specification before applying the same curve sampler. The starting keyframe's curve MUST determine the following segment. `hold` MUST return the start value; `linear` MUST use normalized elapsed segment time. Cubic Bézier MUST invert its monotone unit-interval X polynomial using exactly 40 bisection iterations, then evaluate its Y polynomial. Spring MUST use the closed-form solution of `mass * x'' + damping * x' + stiffness * (x - 1) = 0` over normalized segment time, with `x(0)=0` and `x'(0)=initialVelocity`, selecting underdamped, critically damped, or overdamped form from the discriminant. Exact segment endpoints MUST return the stored keyframe values. Intermediate spring overshoot MUST be allowed for position and gain, while every sampled property MUST be constrained to its existing canonical value bounds before evaluated-scene output; scale MUST use a positive lower bound of `0.000001`. Sampler outputs MUST be finite. Compound values MUST interpolate componentwise only when the channel declares that operation; channels with discrete topology MUST require `hold`. A static property MUST remain authoritative when its corresponding channel is absent. Preview and export MUST consume the same sampled evaluated scene. Fixed fixture samples MUST agree across supported platforms within `1e-9` absolute scalar error.

#### Scenario: Sample a channel at a boundary
- **WHEN** preview and export sample the same item and time at a source keyframe or between two source keyframes, including after valid retained-clock and loop phase mapping
- **THEN** both receive identical resolved values and retain the documented render tolerance

#### Scenario: Sample each parameterized curve
- **WHEN** an active scalar channel uses cubic Bézier or spring on a segment, including a reflected ping-pong segment
- **THEN** fixed timestamps resolve to the specified curve values within `1e-9`, including the exact start and end values and the damped spring regimes

#### Scenario: Bound spring overshoot
- **WHEN** a valid spring would exceed an active channel's canonical value range between keyframes
- **THEN** the evaluated value remains finite and is constrained to that property's documented bounds

#### Scenario: Preserve an unanimated project
- **WHEN** a project has no new channels or all channels omit loops and retained clocks
- **THEN** evaluation and rendered output remain equivalent to the pre-loop behavior

#### Scenario: Preserve retained curve interiors and fractional source time
- **WHEN** a split, trim or duplicate samples a retained Bézier, spring or compound source segment at an interior fractional item-local time
- **THEN** its sample matches the original source at item-local time plus its retained offset without endpoint resampling, intermediate rounding or double application

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

### Requirement: Bounded extended candidate certification
Before publishing a candidate containing extended channels, crop or effects, core MUST certify coupled sample geometry and expanded raster/effect work using deterministic canonical interval analysis. The cumulative budget MUST be 65536 interval-analysis nodes for the final candidate including expanded and retained content; one node MUST represent one bounded time interval for one visual occurrence. Occurrences MUST be processed in canonical order and subdivision MUST visit the left interval before the right. A proved unsafe bound or unresolved safety after budget exhaustion MUST return non-retryable `INVALID_ARGUMENT` before mutation/publication, preserving project, history, draft, revision, aliases and managed-resource bytes. This certification MAY conservatively reject newly supported work whose safety cannot be established within the budget. Legacy candidates without extended properties MUST retain their existing behavior. The budget and conservative rejection semantics MUST be recorded in the canonical extended-visual contract.

Certification MUST include continuous local and inherited scale/affine envelopes for every extended visual occurrence regardless of whether a rotation channel exists. Correlated crop bounds MUST cover the canonical interpolated extent floor as well as exact authored endpoints and holds; equal clocks and curves alone MUST NOT establish an unclamped coupled bound. Existing node, raster, effect and retained/expanded-content limits and deterministic accounting MUST remain unchanged.

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

#### Scenario: Reject effects-only scale overshoot
- **WHEN** a 1000x1000 legacy rectangle has a zero-amount vignette and scale X/Y channels from 1 to 3 over 500 ms with spring mass 1, stiffness 100, damping 1 and initialVelocity 0, without a rotation channel
- **THEN** core returns non-retryable INVALID_ARGUMENT before publication because reachable transformed raster work exceeds the existing limit, preserving project/history/draft/resource bytes, revision and aliases

#### Scenario: Certify scale without rotation in retained and expanded content
- **WHEN** effects-only hidden or retained content or a nested expanded occurrence has unsafe local or inherited scale extrema without a rotation channel
- **THEN** canonical certification rejects it before publication with unchanged authoritative bytes and existing deterministic budgets, while provably safe controls remain accepted

#### Scenario: Reject correlated crop affected by interpolation floor
- **WHEN** crop X interpolates linearly from 0.9999999 to 0.9999998 and width from 0.0000001 to 0.0000002 over 500 ms with Y 0 and height 1, or the corresponding Y/height case is authored
- **THEN** core rejects the candidate with INVALID_ARGUMENT before publication because the clamped interpolated extent violates the coupled crop bound despite endpoint sums of 1, preserving authoritative bytes and revision

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

Exactly integer root clocks MUST retain integer segment, endpoint, hold and loop selection through the actual extended scalar and compound consumers, including timestamps beyond the exact f64 integer range. Interpolation progress MUST preserve differences between adjacent representable integer timestamps without converting absolute times first. This guarantee MUST NOT round or floor fractional inherited clocks or alter existing static fallbacks.

#### Scenario: Sample structured values across curves and loops
- **WHEN** equal-topology paths, matched gradients and effects use hold, linear, cubic Bézier or spring curves with repeat/ping-pong loops
- **THEN** canonical fixed samples preserve exact endpoints, color semantics, bounds, topology and clock agreement across all output intents

#### Scenario: Reject unsafe intermediate samples
- **WHEN** individually valid keyframes produce invalid coupled crop geometry, reversed gradient ordering or excessive expanded render work
- **THEN** canonical candidate preflight rejects the edit and render preflight rejects persisted work without publication

#### Scenario: Sample large integer rotation through extended evaluation
- **WHEN** a root rotation channel uses a linear 0-to-100 ramp from integer timestamp 9007199254740993 to 9007199254740997
- **THEN** actual extended evaluation returns exactly 0 and 100 at endpoints and 25, 50, 75 at the three intervening integer timestamps

#### Scenario: Preserve large integer holds and loop boundaries
- **WHEN** supported extended scalar or compound channels use adjacent integer keyframes, holds, repeat or ping-pong seams beyond the exact f64 integer range, including supported timestamps near the u64 limit
- **THEN** actual consumers preserve exact authored keyframes, hold switching, reflected/repeating timing and static fallbacks without timestamp overflow or precision collapse

#### Scenario: Preserve compound and fractional inherited timing
- **WHEN** path-point, gradient-stop or tint channels use large exactly integer root clocks or existing fractional inherited clocks
- **THEN** component sampling preserves the canonical numeric/color/topology semantics and existing fractional-clock tolerances without introducing integer quantization

#### Scenario: Preserve tiny exact crop endpoints and holds
- **WHEN** a supported crop channel has a positive authored width or height below 0.000001 at an exact keyframe or held sample
- **THEN** sampling preserves the exact authored value there while interpolated samples retain the existing 0.000001 floor, and candidate certification covers both behaviors

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
