## ADDED Requirements

### Requirement: Shared extended visual channel rendering
Frame preview, audiovisual range preview, draft preview, and export MUST consume one renderer-neutral evaluated result for rotation, crop, path points/trim, gradient stops, and blur/glow/tint/vignette channels. They MUST resolve existing half-open clocks, curves, loops, parent transforms, instance time scaling, and repeater timing consistently. Crop/local clip MUST precede graphic paint and ordered local effects, then local/ancestor affine transforms and existing compositing order. Raster caches MUST include all sampled visual properties and occurrence dependencies; they MUST NOT freeze geometry/paint/effects at the first frame. Shape rendering MUST preserve existing fill rules, gradient color space, strokes, anchor geometry, flattening tolerance and budgets; trimmed-path length MUST use the canonical flattened segments before affine transformation, restarting the fraction per subpath, preserving stroke style and suppressing fill for partially trimmed subpaths. Static unaffected items MUST retain prior output. Preview/export comparisons MUST meet SSIM >= 0.99, decoded aligned PCM RMS error <= 0.0001, and timing error no greater than one video frame.

#### Scenario: Compare each property through every intent
- **WHEN** root and nested occurrences animate each newly active property at first, interior, final, loop-seam and reflected-curve samples
- **THEN** frame, range, draft and export share exact evaluated samples and meet the documented decoded output tolerances with visibly changing intermediate output

#### Scenario: Preserve transformed geometry and audio
- **WHEN** cropped asymmetric media and morphing/trimmed gradient shapes use noncentral anchors, rotated parents and ordered effects alongside audio gain
- **THEN** all intents preserve source crop, geometry, paint, ordering, inherited transforms, effect support, and unchanged audio semantics

#### Scenario: Fail before output side effects
- **WHEN** persisted state has a malformed channel/target, missing resource, excessive evaluated work, or unavailable complete rendering support
- **THEN** existing typed preflight fails before destination inspection or artifact publication and leaves project/history/resource bytes unchanged

#### Scenario: Preserve existing regression output
- **WHEN** existing golden projects and migrated projects contain no extended animation, identity crop and empty effects
- **THEN** all existing semantic and rendered regression assertions continue to pass without changed revisions or state
