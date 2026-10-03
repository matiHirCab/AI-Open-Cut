## MODIFIED Requirements

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
