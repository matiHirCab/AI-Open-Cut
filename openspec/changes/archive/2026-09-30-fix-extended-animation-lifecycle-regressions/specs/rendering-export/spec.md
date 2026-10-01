## MODIFIED Requirements

### Requirement: Shared extended visual channel rendering
Frame preview, audiovisual range preview, draft preview, and export MUST consume one renderer-neutral evaluated result for rotation, crop, path points/trim, gradient stops, and blur/glow/tint/vignette channels. They MUST resolve existing half-open clocks, curves, loops, parent transforms, instance time scaling, and repeater timing consistently. Crop/local clip MUST precede graphic paint and ordered local effects, then local/ancestor affine transforms and existing compositing order. Raster caches MUST include all sampled visual properties and occurrence dependencies; they MUST NOT freeze geometry/paint/effects at the first frame. Shape rendering MUST preserve existing fill rules, gradient color space, strokes, anchor geometry, flattening tolerance and budgets; trimmed-path length MUST use the canonical flattened segments before affine transformation, restarting the fraction per subpath, preserving stroke style and suppressing fill for partially trimmed subpaths. Static unaffected items MUST retain prior output. Preview/export comparisons MUST meet SSIM >= 0.99, decoded aligned PCM RMS error <= 0.0001, and timing error no greater than one video frame.

Entering extended rendering with identity effects or a zero-degree rotation MUST preserve a legacy shape's local coordinate origin, authored anchor, position, scale and opacity, including nonzero or negative path origins and inherited transforms. Core MUST derive the affine in the original legacy coordinate basis without requiring persisted Transform2D conversion or changing static output.

Inherited transitions MUST use the same canonical occurrence clock and gain as legacy rendering when extended visual sampling is selected, including identity effects. Offset, scaled, nested and repeated occurrence transitions MUST retain their half-open timing and compositing behavior across frame, range, draft and export. Render-intent parity MUST be checked against independently expected gain or a verified legacy control, not solely against another intent sharing the same plan.

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

#### Scenario: Preserve an offset legacy path under an identity effect
- **WHEN** a legacy-transform rectangular path spans (10,20) to (30,40) on a 64x64 canvas and receives a vignette of amount zero
- **THEN** its red-pixel bounds remain x=10 through 29 and y=20 through 39, and frame, draft, range and export retain the same authored placement and existing output/audio tolerances

#### Scenario: Preserve legacy path coordinates under zero rotation
- **WHEN** legacy paths with nonzero or negative local origins and nonidentity legacy position/scale receive a zero-degree rotation channel, including inherited transforms
- **THEN** canonical extended affines preserve the prior legacy placement, anchor and opacity across render intents while explicit Transform2D behavior remains unchanged

#### Scenario: Preserve an offset occurrence fade under an identity effect
- **WHEN** a red rectangle with a local fade-out from 0 to 500 ms belongs to a component instance starting at root 1000 ms and gains an identity vignette
- **THEN** its root 1250 ms transition gain remains 0.5, matching the unaffected legacy control and independently expected decoded opacity across frame, range, draft and export

#### Scenario: Preserve scaled nested and repeated transition clocks
- **WHEN** supported fade and crossfade transitions occur inside scaled, nested or repeated instances with nonzero offsets and fractional inherited times
- **THEN** every intent preserves canonical transition gains at first, interior and final active samples and half-open boundaries, with unchanged stacking and audio

## ADDED Requirements

### Requirement: Safe sampled visual preparation failures
Core MUST report sampled visual encoder failures using existing stable error codes, retryability, stage and exit status, with sanitized stderr excerpts no longer than 4096 UTF-8 bytes and no exposed absolute private paths. Diagnostic collection MUST remain bounded, preserve valid UTF-8 boundaries, and use the shared process diagnostic policy. Failure MUST reap the child process, remove temporary preparation artifacts, and preserve existing destination and authoritative project/history/resource bytes. Headless MUST translate the safe core result; MCP MUST retain its existing diagnostic sanitization without acquiring rendering semantics.

#### Scenario: Encoder emits private paths and oversized stderr
- **WHEN** the sampled encoder consumes its input then exits unsuccessfully with more than the excerpt limit and quoted Windows or POSIX private paths, including multibyte text
- **THEN** core and headless expose the existing typed visual_prepare failure and exit status with bounded sanitized diagnostics, the child is reaped, temporary artifacts are removed, and no destination or authoritative state is changed

#### Scenario: Preserve existing bridge diagnostics compatibility
- **WHEN** a sampled preparation failure reaches MCP through headless
- **THEN** the stable error code and retryability remain aligned with the canonical catalog and bridge sanitization remains effective without raw private paths or changed public schemas
