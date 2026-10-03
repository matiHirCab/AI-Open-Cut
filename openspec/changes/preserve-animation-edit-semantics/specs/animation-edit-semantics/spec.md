## ADDED Requirements

### Requirement: Preserve continuous source animation through editing
Core MUST preserve each supported typed channel and legacy keyframe source function through split, trim and duplicate. Split MUST retain the original function on the left and sample original local time plus the cut offset on the right; trim MUST sample original local time plus newStart minus previousStart; duplicate MUST sample the exact same local function at the shifted placement. Repeated edits MUST compose offsets without resampling endpoints. Bézier, spring and legacy easing interiors, scalar/point/RGBA/path/gradient values, supported extended targets, gainDb, repeat/ping-pong phase and finite exhaustion MUST remain unchanged at corresponding source times. Fractional inherited clocks MUST retain fractional time. Negative source time MUST hold the first key; post-key unlooped time MUST hold the last key; finite loops MUST retain their original exhaustion. Keys, curves, targets, loops and compilation provenance MUST remain intact. Subsequent replacement keyframes/channels MUST establish fresh local animation unless a valid typed retained clock is explicitly supplied. Preset replacement MUST reset only newly generated/replaced properties and MUST preserve untouched property clocks and the independent legacy clock.

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

### Requirement: Bounded retained animation contracts
Core MUST support optional channel clock and legacyAnimationClock records with signed JavaScript-safe integer offsetMs and positive JavaScript-safe integer sourceDurationMs. Clock presence MUST require nonempty source animation; source keys MUST retain existing ordering/count/value/curve/target constraints and validate against sourceDurationMs rather than the edited item duration. The mapped window endpoints MUST remain signed JavaScript-safe integers. Null, unknown fields, fractional/unsafe values, invalid source bounds or unsafe composition MUST fail as non-retryable INVALID_ARGUMENT before committing state or artifacts. Omitted clocks MUST preserve existing unsliced behavior and public operations/error contracts.

#### Scenario: Reject malformed retained source records
- **WHEN** a supplied/persisted retained clock has invalid fields, bounds, empty source, invalid keys or arithmetic overflow
- **THEN** the operation fails unchanged with INVALID_ARGUMENT and no revision/history/alias/artifact publication

### Requirement: Transactional edits and marker lifecycle
Animation-preserving edits MUST retain expected-revision, locked/missing-item, bounds, changed-ID, ordered alias, atomic rollback and one-step history semantics. Split MUST clear both start expressions; trim MUST clear only when numeric start changes; duplicate MUST shift marker offsets through existing lifecycle rules. Undo, redo and deterministic reopen MUST restore exact source records and clocks without shared mutable copies.

#### Scenario: Commit and recover an alias batch
- **WHEN** a batch creates an animated item by alias, trims/splits/duplicates it and commits, then undo/redo/reopen runs
- **THEN** one revision/history step commits the batch and every restored state contains the exact animation and marker records

#### Scenario: Roll back validation and reference failures
- **WHEN** a batch fails after valid animation editing, or a request has stale revision, locked track, missing item or invalid split/trim bounds
- **THEN** established stable error codes/retryability are retained and current project, retained history, aliases, resources and artifacts remain unchanged

### Requirement: Shared renderer and contract conformance
All render intents MUST use the retained source clocks for visual channels, legacy transforms/visibility and independent gain without double application. Independent expected samples at source interiors/seams/turns/exhaustion and inherited fractional times MUST be verified for frame, range, materialized draft and export. Canonical fixtures, typed consumers, headless/MCP schemas and guides MUST describe clock preservation, supported item restrictions, markers and schema31 accurately; required native tools MUST fail explicitly when unavailable.

#### Scenario: Compare edited output with independent source expectations
- **WHEN** the same edited composition is rendered by each supported intent at selected interior/boundary times
- **THEN** its visual placement/opacity/compound state and decoded audio match independent source expectations within established tolerances
