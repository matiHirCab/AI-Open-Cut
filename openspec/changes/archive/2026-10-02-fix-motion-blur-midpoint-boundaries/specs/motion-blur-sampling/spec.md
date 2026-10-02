## MODIFIED Requirements

### Requirement: Canonical deterministic shutter interval
Core MUST calculate midpoint samples on a centered exposure of width shutterAngleDeg / 360 times 1000 / project fps milliseconds. For N enabled samples, sample i MUST use root time t plus width times ((i+0.5)/N - 0.5), in ascending i order. Root times MUST clamp to the project interval [0,durationMs) before evaluation; integer-root sampling MUST floor once after interval construction and retain all inherited fractional mappings thereafter. Disabled settings MUST sample exactly t. Midpoint offsets MUST floor the exact parsed binary64 shutter angle ratio shutterAngleDeg * 1000 * (2*i+1-N) / (720 * projectFPS * N) without floating cancellation or epsilon snapping, including subnormal angles. Sample computation MUST remain deterministic, use no seed or random jitter, and reject unsafe arithmetic. Every sample MUST evaluate visibility, clipping, transitions, animated crop, paint, ordered effects, local transform, all parent/group/component transforms and opacity, loops, stagger and signed repeaters on their canonical clocks.

#### Scenario: Sample inherited boundaries
- **WHEN** shutter intervals cross keyframes, repeat seams, ping-pong turns, finite exhaustion, nested fractional clocks, stagger or signed repeater offsets
- **THEN** sample times, inherited matrices, opacity and half-open activity match an independent oracle

#### Scenario: Clamp the timeline boundaries
- **WHEN** a centered exposure overlaps project start or exclusive end
- **THEN** clamped samples retain their equal weights and deterministic order without accessing out-of-range resources

#### Scenario: Floor exact midpoint boundaries for every allowed count
- **WHEN** valid exposures with any count 1..16 meet integer keyframe boundaries, fractional frame periods, representable neighboring angles, subnormal angles, high integer roots or timeline clipping
- **THEN** exact ratio flooring, ascending equally weighted samples and inherited held-keyframe coverage match independent arithmetic and native frame/range/draft/export oracles without changing tolerance or audio
