# Animation Edit Semantics Specification

## Purpose

Define exact source animation preservation through timeline split, trim and duplicate, including historical persistence and shared rendering.

## Requirements

### Requirement: Preserve continuous source animation through editing
Core MUST preserve each supported typed channel and legacy keyframe source function through split, trim and duplicate. Split MUST retain the original function on the left and sample original local time plus the cut offset on the right; trim MUST sample original local time plus newStart minus previousStart; duplicate MUST sample the exact same local function at the shifted placement. Existing explicit clocks MUST remain identical in the copy; an implicit clock MUST materialize as an equivalent zero-offset source clock only on an animated copy, leaving its original unchanged. Repeated edits MUST compose offsets without resampling endpoints. Bézier, spring and legacy easing interiors, scalar/point/RGBA/path/gradient values, supported extended targets, gainDb, repeat/ping-pong phase and finite exhaustion MUST remain unchanged at corresponding source times. Fractional inherited clocks MUST retain fractional time. Negative source time MUST hold the first key; post-key unlooped time MUST hold the last key; finite loops MUST retain their original exhaustion. Keys, curves, target kinds/scopes, loops and compilation provenance MUST remain intact. Split-right and duplicated items MUST remap self-owned graphic geometry/fill/stroke target IDs to the generated item identity; cloned effect target IDs MUST remain unchanged. Subsequent replacement keyframes/channels MUST establish fresh local animation unless a valid typed retained clock is explicitly supplied. Preset replacement MUST reset only newly generated/replaced properties and MUST preserve untouched property clocks and the independent legacy clock.

#### Scenario: Split nonlinear source segments and loops
- **WHEN** a supported animated item is split inside a Bézier, spring, legacy eased segment or repeat/ping-pong midcycle
- **THEN** left and right samples at interiors, exact seam/turn/exhaustion and fractional times equal the original source function at corresponding times and maintain existing half-open item activity

#### Scenario: Trim and duplicate retained animation
- **WHEN** an animated item is left/right trimmed, extended, trimmed again and duplicated
- **THEN** offsets compose exactly, source keys remain available, exhaustion does not restart and the copy remains independently editable with unchanged source samples

#### Scenario: Preserve all active value types and audio
- **WHEN** edits target compatible media, text, solid, rectangle, shape, SVG or grid animation, group trim/duplicate, or gain on media with audio
- **THEN** scalar and compound source values and independent audio gain are preserved using the same source clocks, while unsupported targets retain their documented stable errors

#### Scenario: Replace one preset property independently
- **WHEN** a preset replaces one property on an edited item with other retained typed or legacy properties
- **THEN** only the generated property starts on a fresh item-local clock and untouched source functions retain their clocks and samples

#### Scenario: Remap copied self-owned graphic channels
- **WHEN** path-points, path-trim or gradient channels are split or duplicated
- **THEN** copied channel targets name the new owning shape while source keys/curves/loop/clocks remain preserved and effect targets retain cloned per-item effect IDs

### Requirement: Bounded retained animation contracts
Core MUST support optional channel clock and legacyAnimationClock records with signed JavaScript-safe integer offsetMs and positive JavaScript-safe integer sourceDurationMs. Clock presence MUST require nonempty source animation; source keys MUST retain existing ordering/count/value/curve/target constraints and validate against sourceDurationMs rather than the edited item duration. The mapped window endpoints MUST remain signed JavaScript-safe integers. Null, unknown fields, fractional/unsafe values, invalid source bounds or unsafe composition MUST fail as non-retryable INVALID_ARGUMENT for ordinary edits/project documents; invalid recovery journals MUST retain the established non-retryable PROJECT_RECOVERY_FAILED error before committing state or artifacts. Omitted clocks MUST preserve existing unsliced behavior and public operations/error contracts.

#### Scenario: Reject malformed retained source records
- **WHEN** a retained clock in an ordinary edit or project document has invalid fields, bounds, empty source, invalid keys or arithmetic overflow
- **THEN** the operation fails unchanged with INVALID_ARGUMENT and no revision/history/alias/artifact publication

#### Scenario: Reject invalid retained-clock recovery journals before replay
- **WHEN** a recovery journal contains invalid clock windows, empty source animation or out-of-source keys in root/component items of its current project or undo/redo snapshots
- **THEN** core returns non-retryable PROJECT_RECOVERY_FAILED before replay and current project, retained history and journal bytes remain unchanged

### Requirement: Transactional edits and marker lifecycle
Animation-preserving edits MUST retain expected-revision, locked/missing-item, bounds, changed-ID, ordered alias, atomic rollback and one-step history semantics. Split MUST clear both start expressions; trim MUST clear only when numeric start changes; duplicate MUST shift marker offsets through existing lifecycle rules. Undo, redo and deterministic reopen MUST restore exact source records and clocks without shared mutable copies.

#### Scenario: Commit and recover an alias batch
- **WHEN** a batch creates an animated item by alias, trims/splits/duplicates it and commits, then undo/redo/reopen runs
- **THEN** one revision/history step commits the batch and every restored state contains the exact animation and marker records

#### Scenario: Roll back validation and reference failures
- **WHEN** a batch fails after valid animation editing, or a request has stale revision, locked track, missing item or invalid split/trim bounds
- **THEN** established stable error codes/retryability are retained and current project, retained history, aliases, resources and artifacts remain unchanged

#### Scenario: Preserve an independently duplicated animated audio item
- **WHEN** an animated media item with implicit source clocks is duplicated at a nonzero timeline offset
- **THEN** the copy materializes equivalent zero-offset source clocks, its original remains unchanged, and copied audio is silent before its new half-open activity interval without phase reset

### Requirement: Shared renderer and contract conformance
All render intents MUST use the retained source clocks for visual channels, legacy transforms/visibility and independent gain without double application. Retained animated non-instance media MUST reset timestamps and sample its local gain/volume envelope before receiving exactly one actual timeline audio delay, then sample ducking on the resulting absolute timeline clock, replacing timestamp-only placement before mixing so split-right, trimmed and duplicated clips remain silent before their span start; existing instance delay and unclocked pipeline behavior MUST remain unchanged. Independent expected samples at source interiors/seams/turns/exhaustion and inherited fractional times MUST be verified for frame, range, materialized draft and export. Canonical fixtures, typed consumers, headless/MCP schemas and guides MUST describe clock preservation, supported item restrictions, markers and schema31 accurately; required native tools MUST fail explicitly when unavailable.

#### Scenario: Preserve retained audio placement and global ducking
- **WHEN** retained non-instance media carries gain/legacy-volume or only visual clocks and is sampled in full or nonzero later range/draft/export windows with ducking
- **THEN** its local envelope, actual activity delay and global ducking clock apply exactly once and its decoded samples match independent source expectations

#### Scenario: Compare edited output with independent source expectations
- **WHEN** the same edited composition is rendered by each supported intent at selected interior/boundary times
- **THEN** its visual placement/opacity/compound state and decoded audio match independent source expectations within established tolerances

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
