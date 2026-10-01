## MODIFIED Requirements

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
