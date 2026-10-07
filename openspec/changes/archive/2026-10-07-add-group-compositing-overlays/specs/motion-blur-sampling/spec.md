## MODIFIED Requirements

### Requirement: Canonical deterministic shutter interval
Core MUST calculate midpoint samples on a centered exposure of width shutterAngleDeg / 360 times 1000 / project fps milliseconds. For N enabled samples, sample i MUST use root time t plus width times ((i+0.5)/N - 0.5), in ascending i order. Root times MUST clamp to the project interval [0,durationMs) before evaluation; integer-root sampling MUST floor once after interval construction and retain all inherited fractional mappings thereafter. Disabled settings MUST sample exactly t. Midpoint offsets MUST floor the exact parsed binary64 shutter angle ratio shutterAngleDeg * 1000 * (2*i+1-N) / (720 * projectFPS * N) without floating cancellation or epsilon snapping, including subnormal angles. Sample computation MUST remain deterministic, use no seed or random jitter, and reject unsafe arithmetic. Every uncontrolled leaf and private named-provider sample MUST evaluate visibility, clipping, transitions, animated crop, paint, ordered effects, local transform, all parent/group/component transforms and opacity, loops, stagger and signed repeaters on their canonical clocks. Controlled aggregate drawing MUST use the explicit temporal boundary below, retaining these exact midpoint/root/fractional clock rules for descendant-relative stages.


An explicit controlled aggregate MUST sample its owner and uncontrolled outward ancestry affine/opacity only up to (excluding) the immediately enclosing controlled owner at that aggregate output sample once; top-level aggregates include ancestry through root. Descendant leaf shutters MUST finish their source and descendant-relative transforms/opacity on their own canonical midpoint clocks in that owner-local frame before clip/effects. Nested controlled aggregates MUST use the enclosing requested local sample as their output sample and freeze their own outward boundary there only through the enclosing aggregate local frame, excluding that enclosing owner and its outward factors. Owner outward visibility/interval and gain MUST be sampled at this boundary; descendant activity/relative visibility/interval MUST remain sampled at each descendant shutter time, intersecting the canonical retained spans. No ownership boundary MUST divide by owner opacity or invert an owner matrix. The uncontrolled scenarios below MUST remain unchanged; private named providers MUST retain full world-ancestry shutter sampling even under controlled owners. Group/instance authored motionBlur MUST remain unsupported.

#### Scenario: Sample inherited boundaries
- **WHEN** shutter intervals cross keyframes, repeat seams, ping-pong turns, finite exhaustion, nested fractional clocks, stagger or signed repeater offsets
- **THEN** sample times, inherited matrices, opacity and half-open activity match an independent oracle

#### Scenario: Clamp the timeline boundaries
- **WHEN** a centered exposure overlaps project start or exclusive end
- **THEN** clamped samples retain their equal weights and deterministic order without accessing out-of-range resources

#### Scenario: Floor exact midpoint boundaries for every allowed count
- **WHEN** valid exposures with any count 1..16 meet integer keyframe boundaries, fractional frame periods, representable neighboring angles, subnormal angles, high integer roots or timeline clipping
- **THEN** exact ratio flooring, ascending equally weighted samples and inherited held-keyframe coverage match independent arithmetic and native frame/range/draft/export oracles without changing tolerance or audio

#### Scenario: Freeze only explicit aggregate outward ancestry
- **WHEN** controlled owners translate/rotate, opacity crosses0, owner affine becomes singular, or overlapping children have unequal shutters
- **THEN** independent local midpoint planes and center-sampled outward gain/affine prove the boundary, without reprojecting world planes/dividing by opacity; uncontrolled/private provider full-world shutters remain exact

### Requirement: Shared bounded temporal composition
Each enabled uncontrolled leaf and private named provider MUST average equally weighted premultiplied linear-light canvas/query rasters after each sample's crop, local clip, paint, ordered effects, complete inherited affine and transition opacity; existing layer stacking MUST then compose the averaged leaf. Controlled aggregate drawing MUST instead average each child's completed relative-local premultiplied planes, including matching-world matte queries, then compose child copies in canonical order, clip/effect the aggregate, and apply its frozen outward affine/opacity once. Invisible samples MUST contribute transparent pixels. Core MUST reuse the issue43 pipeline for frame, audiovisual range, materialized draft and export. Independent native expected pixels MUST demonstrate blur rather than merely comparing identical consumers. Audio source sampling, gain and synchronization MUST remain unchanged. Native parity MUST meet SSIM >= 0.99, aligned decoded PCM RMS error <= 0.0001 and timing error at most one frame. Disabled/zero shutter and single samples MUST preserve previous output. Cumulative sample-weighted pixel work MUST not exceed 268435456 units per output scene frame, alongside stricter existing effect, geometry, occurrence and surface limits; all arithmetic MUST be checked. Hidden, unused, retained and expanded content MUST undergo canonical validation. Excess or unavailable rendering support MUST fail before destination inspection, workspace creation or persistence publication.


Every requested controlled/provider frame and actual sample MUST retain existing shared sample/effect/matte/geometry/live-memory certification. Existing inherited-motion scenarios below MUST continue unchanged for uncontrolled drawing; controlled native witnesses MUST separately prove their approved boundary and no aggregate-before-child-average substitution.

#### Scenario: Preserve full inherited motion and ordering
- **WHEN** asymmetric nested leaves animate under rotating parents with noncentral anchors, clipping, crop and differently ordered effects
- **THEN** independent midpoint and linear-light composition expectations match frame, range, draft and export within the documented tolerance

#### Scenario: Bound work and preserve output
- **WHEN** shutter multiplication exceeds a sample, occurrence, geometry, effect or pixel-work limit or an encoder fails
- **THEN** core returns its existing sanitized typed failure before publication and preserves destinations, project/history bytes and resources

#### Scenario: Preserve audio and compatibility
- **WHEN** enabled blur is rendered alongside audio and compared with zero shutter, single sample and omitted settings controls
- **THEN** audio remains synchronized and unchanged and compatibility controls retain prior visuals

#### Scenario: Preserve controlled nested shutter all-intent parity
- **WHEN** nested controlled groups/instances use unequal leaf/provider shutters, retiming/repeaters, positive effects and audio
- **THEN** independent midpoint and local source-over equations match frame/nonzero-origin range/draft/export with unchanged SSIM/PCM/timing bounds, source clocks and audio
