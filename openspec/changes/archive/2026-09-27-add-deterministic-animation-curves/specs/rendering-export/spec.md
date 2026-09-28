## ADDED Requirements

### Requirement: Shared parameterized curve rendering
Frame preview, audiovisual range preview, materialized draft preview, and final export MUST consume the same evaluated-scene Bézier and spring samples for every active channel, including visual transforms, opacity, and audio gain. Their semantic plans MUST agree exactly at equivalent timestamps and decoded output MUST satisfy the existing visual, audio, and timing tolerances. Invalid persisted curve parameters or non-finite work MUST fail in canonical preflight before destination inspection, renderer execution, temporary files, or artifact publication. Scenes without new curves MUST retain existing output.

#### Scenario: Compare each render intent
- **WHEN** equivalent immutable scenes with visual and audio curves are rendered at matching fixed timestamps
- **THEN** evaluated values and semantic plans agree and decoded outputs meet the documented tolerance

#### Scenario: Reject invalid persisted curve before output
- **WHEN** a persisted scene contains invalid parameterized curve data
- **THEN** each render intent returns a stable typed error and publishes no artifact

#### Scenario: Preserve legacy output
- **WHEN** a project uses only existing `hold`, `linear`, or legacy keyframe easing
- **THEN** frame, range, draft, and export output remain equivalent to pre-activation behavior
