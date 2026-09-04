## ADDED Requirements

### Requirement: Shared complete Transform2D rendering
Frame preview, audiovisual range preview, materialized draft preview, and final export MUST consume the same evaluated affine facts for every supported visual source. The adapter MUST preserve transformed source offsets and transparency, interpolate premultiplied alpha, clip to the composition, and apply Transform2D opacity once. It MUST preserve existing audio semantics and legacy rendering when transform2d is absent. No backend SHALL approximate or omit unsupported transform components.

#### Scenario: Render all transformed visual kinds
- **WHEN** asymmetric media, text, solid, rectangle, and caption fixtures use anchor, both scales, both skews, rotation, units, and opacity
- **THEN** all output intents match the canonical coordinate oracle and preserve transparency and clipping

#### Scenario: Compare preview and export
- **WHEN** equivalent timestamps and ranges are rendered in all supported intents
- **THEN** semantic affine plans agree exactly, visual SSIM is at least 0.99, aligned float-PCM RMS error is at most 0.0001, and timing differs by at most one output frame

#### Scenario: Preserve old rendered fixtures
- **WHEN** migrated legacy fixtures with non-default transform, animation, captions, and transitions render without Transform2D
- **THEN** their prior output and existing tolerance guarantees remain unchanged

### Requirement: Fail closed before affine rendering
If no configured local backend can execute the complete affine scene, rendering MUST fail with DEPENDENCY_UNAVAILABLE before rasterization, process execution, or artifact publication. Resource paths MUST retain existing managed/path-safe preparation and no raw expressions, executable SVG, or network resources SHALL be accepted as Transform2D input.

#### Scenario: Unavailable complete support
- **WHEN** a backend lacks any required affine operation
- **THEN** every intent fails deterministically with DEPENDENCY_UNAVAILABLE and publishes no degraded artifact

#### Scenario: Reject unsafe typed input
- **WHEN** a client inserts an expression, path, URL, or executable markup into a numeric or unit field
- **THEN** typed validation rejects it before path resolution, rendering, or artifact publication

### Requirement: Isolated transformed caption box
Active Transform2D captions MUST use source width max(1, ceil(measured text width)) + 24 and height max(1, ceil(measured text height)) + 24, 12-pixel text insets, existing font size/colors, and background alpha 0.75. Transform2D position MUST replace legacy bottom-center placement and anchor MUST refer to the complete source box. Legacy captions MUST retain their direct-render behavior.

#### Scenario: Place a transformed caption
- **WHEN** a caption has a noncentral anchor and explicit Transform2D position
- **THEN** its complete measured box is transformed at that position with the specified insets and background alpha

#### Scenario: Preserve an untransformed caption
- **WHEN** transform2d is absent
- **THEN** the existing bottom-centered caption rendering remains unchanged
