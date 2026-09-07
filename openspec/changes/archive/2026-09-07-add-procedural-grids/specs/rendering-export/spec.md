## ADDED Requirements

### Requirement: Shared complete procedural grid rendering
Frame preview, audiovisual range preview, materialized draft preview and final export MUST consume the same evaluated grid facts and shared vector coverage/compositing semantics. Equivalent immutable settings MUST produce exactly equal semantic plans after established interval/intent normalization, visual SSIM at least 0.99, aligned float-PCM RMS error at most 0.0001 and timing within one output frame. Independent geometry/pixel evidence MUST cover every grid pattern, solid/gradient/translucent paints, dashed strokes, clipping, fractional geometry, composed transforms, component clocks and legacy overlap. Rendering MUST remain read-only and preserve existing audio, media/path safety and atomic output publication. Missing complete support MUST fail with DEPENDENCY_UNAVAILABLE; invalid values or excessive work MUST fail with INVALID_ARGUMENT before destination inspection, artifact allocation or render execution, without fallback.

#### Scenario: Compare every render intent
- **WHEN** fixed grid scenes render repeatedly at corresponding timestamps through frame, range, draft and export, including requested output sizes differing from project/component canvases
- **THEN** plans agree, independent geometry and paint expectations hold and decoded output meets the established visual/audio/timing thresholds without mutating drafts or projects

#### Scenario: Fail before output preparation
- **WHEN** invalid or excessive grid work, unsafe resources elsewhere or unavailable backend support is encountered
- **THEN** the established typed failure occurs before output preparation and preserves destination and project/history bytes

#### Scenario: Preserve legacy output
- **WHEN** existing fixtures without grids migrate and render
- **THEN** their established visual/audio golden guarantees and output publication behavior remain unchanged
