## ADDED Requirements

### Requirement: Migrated motion pack edit attribution
Split, trim and duplicate of migrated motion-pack animations MUST use the same retained source-clock semantics as all supported animations and preserve exact original tagged preset/compiler parameters and source primitive keys. Replacement compilers MUST start only their new properties without inherited clocks, preserving unrelated properties. No edit may recompile from catalog metadata or discard provenance to avoid validation.

#### Scenario: Edit migrated Pack source at interior seams
- **WHEN** a migrated scalar or Pack animation is split midsegment/midcycle, left-trimmed or duplicated
- **THEN** interior, seam, turn and finite-exhaustion samples remain equivalent at composed source times and exact provenance records survive undo/redo and reopen

#### Scenario: Preserve atomic migrated Pack workflows
- **WHEN** a bridge alias batch edits migrated Pack source and a later command fails or the item is stale, locked, missing or out of bounds
- **THEN** the established errors preserve the complete current/history generation; successful edits remain identical through bridge/headless contracts and persisted reopen

### Requirement: Rectangle native pointwise sample fidelity
The primitive affine rectangle pipeline MUST preserve mathematical source/edit RGBA samples at atlas boundaries during pixelwise premultiply, unpremultiply and opacity operations across supported FFmpeg backends. Identity-coordinate channel lookups MUST use nearest pixel reads, as the existing shape path does; geometric coordinate remaps and bilinear interpolation MUST remain unchanged. Existing media/text orientation semantics, budgets, independent animation equations and native/golden tolerances MUST remain unchanged.

#### Scenario: Native source and retained samples across backend versions
- **WHEN** unclocked rectangle source and retained split/trim/copy segments are rendered under FFmpeg6.1 and7.1 at interior, seam, turn and finite-exhaustion times
- **THEN** all three render intents match the unchanged independent geometry/alpha/curve/loop oracle without an inflated atlas border or a loosened tolerance

### Requirement: Sampled raster seam cadence
Prepared sampled CPU-raster inputs MUST use canonical canvas frame cadence and a sufficiently precise timebase before existing placement and compositing, preserving absolute millisecond interval placement, exact visible bounds, source-time evaluation and retained clocks. Default image-demuxer cadence MUST NOT suppress a retained segment at its exact visible start. Already canonical-cadence encoded sampled intervals, non-frame-aligned starts and ordinary media inputs MUST retain their existing semantics; no oracle equation, time, count or tolerance may be relaxed.

#### Scenario: CPU effect retained segment at exact trim seam
- **WHEN** CPU-effect source and retained split/trim/copy segments are composited at their exact visible seam on a10fps canvas under FFmpeg6.1 and7.1
- **THEN** all three render intents show the unchanged independent source sample at the seam and preserve subsequent fractional source phase, undo/redo/reopen and native/golden parity

### Requirement: Canonical sampled raster activity
Sampled CPU-raster compositing MUST use the canonical sampler's encoded timeline/shutter activity as its visibility authority, retaining existing placement and EOF-pass behavior. A redundant backend floating-point activation comparison MUST NOT suppress an otherwise active exact seam. Non-motion-blurred inactive gaps/end bounds MUST remain transparent, existing motion-blur shutter activity MUST remain unchanged, and ordinary media/affine activation rules MUST remain unchanged. No epsilon, widened bound or oracle relaxation is permitted.

#### Scenario: Exact draft seam and inactive CPU gaps
- **WHEN** CPU-effect retained edits and a draft move are rendered at inactive edited gaps400/1000ms, inactive draft1500ms and active draft1600ms under FFmpeg6.1 and7.1
- **THEN** every supported render intent follows the unchanged canonical sampled activity and independent oracle, without leaking a cached image into a gap or suppressing the exact active seam

### Requirement: Fractional retained clock precision
Within the existing accepted integer clock/key ranges, retained integer offsets MUST NOT discard a fractional part already represented in local inherited time. Source composition, before-first/negative holds, repeat/ping-pong phase and finite exhaustion, key selection and relative segment interpolation MUST preserve integer whole time separately from the represented fraction until safe normalized evaluation. Scalar, compound and evaluated legacy samplers, conservative bounds and renderer expressions MUST agree. Accepted bounds, persisted clock shape, source keys/curves/loops and ordinary integer behavior MUST remain unchanged; input precision already absent before clock composition is not invented.

#### Scenario: Huge retained offset and fractional loop phase
- **WHEN** an accepted retained offset at2^52 or near the safe limit is combined with local0.25/0.75 on repeat or ping-pong channels
- **THEN** canonical and backend samples equal the mathematically equivalent small-phase reference, including negative holds, finite budget/exhaustion, turns and seams

#### Scenario: Relative short segment near a huge key origin
- **WHEN** scalar, compound or evaluated legacy channels have an accepted huge integer key origin and a short segment sampled through a retained clock
- **THEN** represented fractional progress and conservative bounds remain correct without a huge floating addition or subtraction swallowing the fraction, and source/edit persistence remains unchanged
