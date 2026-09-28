## ADDED Requirements

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
