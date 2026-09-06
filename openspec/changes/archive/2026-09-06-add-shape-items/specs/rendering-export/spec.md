## ADDED Requirements

### Requirement: Shared complete shape rendering
Frame, audiovisual range, materialized draft and final export MUST consume the same evaluated shape semantics and shared raster/compositing path. Backend code MUST NOT reconstruct shapes from persisted records or accept SVG, raw renderer expressions, arbitrary resource paths or network content. Shapes MUST preserve existing audio, composition order, transforms, parent/instance semantics and atomic output publication.

Equivalent immutable scene settings MUST yield exactly equal semantic plans after interval/intent normalization, visual SSIM at least 0.99, aligned float-PCM RMS error at most 0.0001 and timing within one output frame. Native golden evidence MUST cover all seven geometries, asymmetric transforms, gradients, strokes, transparency, fill rules, clipping, hierarchy, components and legacy overlap, with independent expected geometry/pixel assertions. If complete semantics are unavailable, every intent MUST return DEPENDENCY_UNAVAILABLE before rasterization or output preparation; no approximate fallback MUST be published.

#### Scenario: Compare every output intent
- **WHEN** the fixed shape fixture renders through frame, range, draft and export at equivalent selections repeatedly
- **THEN** semantic plans match, independent geometry/color expectations hold and decoded visual/audio/timing results meet the documented tolerances without draft mutation

#### Scenario: Fail before output side effects
- **WHEN** shape values or derived work are invalid, resources elsewhere are unsafe, or complete backend support is unavailable
- **THEN** existing typed errors occur before collision inspection, workspace/output allocation, rasterization or render execution, and existing destination/project/history bytes remain unchanged

#### Scenario: Preserve legacy rendering
- **WHEN** migrated legacy fixtures contain no shape items
- **THEN** their previous visual/audio golden guarantees remain satisfied
