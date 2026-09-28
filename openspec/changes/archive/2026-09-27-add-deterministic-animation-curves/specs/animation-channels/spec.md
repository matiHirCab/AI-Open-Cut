## MODIFIED Requirements

### Requirement: Bounded channel keyframes
Each channel MUST contain at most 1,000 keyframes at distinct, strictly increasing integer-millisecond offsets within its item's half-open duration, and each item MUST contain at most 64 channels with unique channel-and-target identity. Every accepted active scalar and component of a compound value MUST be finite and within its declared canonical bounds; path points MUST contain at most 4,096 points and gradient stops at most 32 stops before any later activation. Inactive values MUST be rejected before persistence regardless of their deferred numeric bounds. Channel input MUST reject unknown fields, raw renderer expressions, executable SVG, arbitrary paths, and network resources. Active timing MUST remain absolute item-local milliseconds; marker and loop variants MUST remain unavailable. Curves MUST be `hold`, `linear`, `cubic_bezier`, or `spring`; existing string `hold`/`linear` payloads MUST remain accepted. A cubic Bézier MUST contain finite `x1`, `y1`, `x2`, `y2`, each in [0, 1], with `x1 <= x2`. A spring MUST contain finite `mass` in [0.01, 100], `stiffness` in [0.01, 10000], `damping` in [0.01, 1000], and `initialVelocity` in [-100, 100]. Parameterized curves MUST be strict closed tagged records and MUST be rejected on a keyframe with no following segment.

#### Scenario: Reject invalid channel sequence
- **WHEN** an edit has duplicate channel identities, duplicate or descending times, an out-of-duration time, non-finite or out-of-bound value, or a named complexity limit overflow
- **THEN** core returns `INVALID_ARGUMENT` and leaves revision and history unchanged

#### Scenario: Reject deferred timing and curves
- **WHEN** a caller supplies a marker time, loop, malformed or unknown curve variant, out-of-bound curve parameter, or parameterized curve on the terminal keyframe
- **THEN** the mutation fails with `INVALID_ARGUMENT` without interpreting the payload as an expression or changing revision and history

#### Scenario: Preserve existing curve payloads
- **WHEN** a caller writes existing string `hold` or `linear` channel keyframes
- **THEN** the values retain their existing wire representation and sampled results

### Requirement: Deterministic channel sampling
For every active channel, editor-core MUST sample item-local milliseconds with a held first value before its first keyframe, a held last value after its last keyframe, exact keyframe values at their timestamps, and type-appropriate interpolation between values. The starting keyframe's curve MUST determine the following segment. `hold` MUST return the start value; `linear` MUST use normalized elapsed segment time. Cubic Bézier MUST invert its monotone unit-interval X polynomial using exactly 40 bisection iterations, then evaluate its Y polynomial. Spring MUST use the closed-form solution of `mass * x'' + damping * x' + stiffness * (x - 1) = 0` over normalized segment time, with `x(0)=0` and `x'(0)=initialVelocity`, selecting underdamped, critically damped, or overdamped form from the discriminant. Exact segment endpoints MUST return the stored keyframe values. Intermediate spring overshoot MUST be allowed for position and gain, while every sampled property MUST be constrained to its existing canonical value bounds before evaluated-scene output; scale MUST use a positive lower bound of `0.000001`. Sampler outputs MUST be finite. Compound values MUST interpolate componentwise only when the channel declares that operation; channels with discrete topology MUST require `hold`. A static property MUST remain authoritative when its corresponding channel is absent. Preview and export MUST consume the same sampled evaluated scene. Fixed fixture samples MUST agree across supported platforms within `1e-9` absolute scalar error.

#### Scenario: Sample a channel at a boundary
- **WHEN** preview and export sample the same item and time at a keyframe or between two keyframes
- **THEN** both receive identical resolved values and retain the documented render tolerance

#### Scenario: Sample each parameterized curve
- **WHEN** an active scalar channel uses cubic Bézier or spring on a segment
- **THEN** fixed timestamps resolve to the specified curve values within `1e-9`, including the exact start and end values and the damped spring regimes

#### Scenario: Bound spring overshoot
- **WHEN** a valid spring would exceed an active channel's canonical value range between keyframes
- **THEN** the evaluated value remains finite and is constrained to that property's documented bounds

#### Scenario: Preserve an unanimated project
- **WHEN** a project has no new channels
- **THEN** evaluation and rendered output remain equivalent to the pre-milestone behavior

## ADDED Requirements

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
