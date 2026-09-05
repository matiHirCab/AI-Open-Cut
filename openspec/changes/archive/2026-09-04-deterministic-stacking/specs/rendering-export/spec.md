## ADDED Requirements

### Requirement: Shared stacking across render intents
Frame preview, audiovisual range preview, materialized draft preview, and export MUST consume canonical evaluated visual order without backend-specific sorting. Fixtures MUST cover overlapping opaque/transparent visuals, equal and unequal z-index, reordered tracks/items, captions, transitions, hidden items, legacy transforms, and Transform2D. Existing complete-backend readiness and path-safety rules MUST remain unchanged.

#### Scenario: Compare rendered stacking
- **WHEN** all intents render the same ordered project revision and equivalent timestamps/ranges
- **THEN** semantic layer plans agree exactly, expected occlusion is verified independently, visual SSIM is at least 0.99, aligned float-PCM RMS error is at most 0.0001, and timing differs by at most one frame

#### Scenario: Preserve migrated legacy output
- **WHEN** equivalent pre-migration and schema-9 fixtures render with default z-index
- **THEN** existing visual/audio baselines remain within documented tolerances and repeated reopen/render preserves order

#### Scenario: Reject unsupported complete scenes
- **WHEN** no configured local backend supports the complete evaluated scene
- **THEN** all intents fail with DEPENDENCY_UNAVAILABLE before rasterization, render execution, or artifact publication rather than emitting reordered or degraded output
