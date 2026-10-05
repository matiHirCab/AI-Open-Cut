## ADDED Requirements

### Requirement: Mask metadata preserves current evaluated rendering
Schema32 authored masks MUST remain inactive at the mask stage introduced by linear-light-compositing. Valid nonempty metadata SHALL NOT alter evaluated scene ordering, clocks, local rasters, geometric/raster certification, resource requests, normalized render graphs, prepared visual samples, decoded pixels/audio, timing or artifact publication for otherwise identical frame, audiovisual range, draft preview or export requests. The shared mask stage MUST remain identity in every intent. Persisted/draft/snapshot and revision-scoped cache fingerprints MAY change when authored metadata commits a new revision. Existing revision-scoped cache invalidation MUST remain unchanged. Equal-identity/equal-revision fixture render plans MUST match exactly; before/after public-edit comparison MUST normalize only explicitly named revision/snapshot admission identities and established output/temp-path fields, with all geometry, source/resource selection, clock/effect/order semantics and normalized graph content equal. Model support SHALL NOT emit a mask-rendering capability or activate mask animation targets.

#### Scenario: Preserve every intent with authored masks
- **WHEN** otherwise identical #49 scenes are rendered with masks omitted or valid nonempty painted alpha/luma stacks
- **THEN** evaluated geometry/clock/effect/order/resource semantics and normalized graphs are equal under that explicit identity projection across frame/range/draft/export, lossless native pixels are equal, audio/timing remain equal and metadata stays persisted

#### Scenario: Keep render failures and budgets unchanged
- **WHEN** unchanged media/dependency/renderer failure or existing excessive-render-work input is submitted with valid mask metadata
- **THEN** existing typed failure, bounded-work and no-publication guarantees remain unchanged without rasterizing masks
