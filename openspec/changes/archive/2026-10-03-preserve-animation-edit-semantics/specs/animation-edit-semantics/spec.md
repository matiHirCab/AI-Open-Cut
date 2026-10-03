## ADDED Requirements

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
