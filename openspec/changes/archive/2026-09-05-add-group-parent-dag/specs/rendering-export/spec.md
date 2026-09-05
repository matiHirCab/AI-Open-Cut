## ADDED Requirements

### Requirement: Shared parented visual rendering
Frame, range, materialized draft, and export MUST consume the same complete evaluated ancestor transform, opacity, visibility, interval, and flat order. Existing unparented output and audio SHALL remain unchanged. Complete-backend readiness and path safety MUST remain mandatory; unsupported complete scenes SHALL fail with DEPENDENCY_UNAVAILABLE rather than degraded rendering.

#### Scenario: Compare all grouped render intents
- **WHEN** nested groups with asymmetric visuals, legacy and Transform2D children, transparency, hidden ancestors, clipped intervals, captions, and media are rendered at equivalent selections
- **THEN** semantic plans agree exactly, independent geometry and occlusion expectations hold, SSIM is at least 0.99, aligned float-PCM RMS error is at most 0.0001, timing differs by at most one frame, and draft preview leaves state/history unchanged

#### Scenario: Fail before artifact work
- **WHEN** a graph or composed geometry is invalid or the backend cannot render its complete evaluated scene
- **THEN** all intents return the specified typed failure before destination inspection, rasterization, render execution, or artifact publication

#### Scenario: Preserve existing golden output
- **WHEN** migrated unparented fixtures are rendered and reopened repeatedly
- **THEN** existing visual and audio baselines remain within their established tolerances
