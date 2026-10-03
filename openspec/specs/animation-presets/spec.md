# Animation Presets Specification

## Purpose

Define editor-core's versioned animation-preset compilation, descriptive provenance and deterministic primitive-only persistence and evaluation.

## Requirements

### Requirement: Versioned primitive compilation
Editor-core MUST own a pure, bounded compiler addressed by mandatory `presetId` and positive integer `presetVersion`. The initial compilation catalog MUST contain exactly `scalar_tween` version 1, with compiler version 1. Unknown identifiers, unsupported versions, missing versions and `latest` selection MUST fail with non-retryable `INVALID_ARGUMENT` before publication. Compilation MUST use no filesystem, network, executable content, renderer expression or transport-specific expansion. The compiler MUST produce the existing canonical channel primitives, with fully materialized effective parameters for provenance; it MUST NOT introduce new interpolation or rendering rules.

#### Scenario: Compile an explicitly versioned seed
- **WHEN** a caller applies supported `scalar_tween@1` with valid typed parameters to a compatible item
- **THEN** core produces the specified editable channel and effective source record with compiler version 1

#### Scenario: Reject unsupported compilation identity
- **WHEN** a request omits the version, uses `latest`, names another preset, or requests an unsupported version
- **THEN** it fails with `INVALID_ARGUMENT` without changing project, revision, history, aliases or managed bytes

### Requirement: Scalar tween seed semantics
The seed's parameters MUST be a closed record with `property`, `startMs`, `durationMs`, `from`, `to` and optional `curve`. Supported properties MUST be exactly `transform.position_x`, `transform.position_y`, `transform.scale_x`, `transform.scale_y`, `transform.opacity` and `audio.gain_db`, using their existing active scalar endpoint bounds and target compatibility. `from` and `to` MUST be explicit absolute finite endpoint values. Visual properties MUST retain the current legacy-transform exclusion of transform2d, audio-only media, captions, transitions and repeaters; audio gain MUST require media with audio. Root group and root component-instance visual channels MUST retain their current behavior without editing a definition.

Timing MUST use losslessly represented item-local integer milliseconds, with nonnegative `startMs`, positive `durationMs`, and each value and their checked sum no greater than 9,007,199,254,740,991. The generated times MUST both be within the item's half-open duration. The compiler MUST generate exactly one targetless channel with two scalar keyframes at `startMs` and `startMs + durationMs`, holding `from` before the first and `to` after the last. The starting keyframe MUST use the supplied existing `hold`, `linear`, `cubic_bezier` or `spring` curve, defaulting to `linear`; the terminal keyframe MUST use `hold`; the channel MUST have no loop. Curve validation and sampling MUST retain their existing bounds, endpoints and clamping semantics.

#### Scenario: Resolve each supported property and curve
- **WHEN** each of the six properties is applied with each valid curve to a compatible target
- **THEN** independently specified keys, effective parameters and sampled values match the existing channel semantics without reading an implicit static baseline

#### Scenario: Respect half-open timing
- **WHEN** an item has duration 1,000 and the request uses start 0 with duration 999 or duration 1,000
- **THEN** duration 999 succeeds with its last key at 999 and duration 1,000 fails with `INVALID_ARGUMENT`, without retiming or clamping

#### Scenario: Reject incompatible or deferred output
- **WHEN** a request targets an incompatible item, uses a scoped target, loop or marker parameter, or names a property outside the six-property seed
- **THEN** core returns `INVALID_ARGUMENT` without activating a deferred channel or modifying a component definition

### Requirement: Explicit channel collision policy
Preset application MUST accept `collisionPolicy` equal to `reject` or `replace`, defaulting to `reject`. Collision MUST mean equality of canonical channel property and target identity, regardless of keyframe interval overlap. `reject` MUST fail with `INVALID_ARGUMENT` for an existing identical typed channel. Explicit `replace` MUST replace that complete channel, including its prior loop, in its existing collection position. A new identity MUST append one channel. Unrelated channels and their ordering, provenance, keyframes, targets and loops, static properties, effects and legacy keys MUST remain unchanged. Either policy MUST reject overlapping legacy keys using the existing canonical collision rule; preset application MUST NOT remove or convert legacy animation.

#### Scenario: Reject a disjoint-time collision by default
- **WHEN** the item already has the generated property even with a disjoint interval or identical generated keys and no policy is provided
- **THEN** the call fails with `INVALID_ARGUMENT` and preserves the original channel and source label

#### Scenario: Replace only the explicitly selected identity
- **WHEN** `replace` is applied to an existing identity among other channels
- **THEN** only that channel and its source record change, its position stays fixed, unrelated state is equal, and undo restores its entire prior channel including its loop

#### Scenario: Preserve legacy animation on either policy
- **WHEN** overlapping legacy position, scale, opacity or volume keys exist and a preset requests the conflicting typed property under either policy
- **THEN** core returns `INVALID_ARGUMENT` and preserves every legacy key and project/history byte

### Requirement: Descriptive persisted provenance
A successful preset edit MUST persist its resolved channel in `animationChannels` and one source record in optional item `animationPresetProvenance`, keyed by the generated targetless property. Each record MUST contain `presetId`, `presetVersion`, `compilerVersion` and fully materialized parameters, including the effective curve. Omission MUST mean an empty map; `null` MUST be invalid; an empty map MUST serialize as omitted. Entries MUST correspond to existing targetless channel identities and matching parameter properties. Provenance MUST be descriptive rather than executable or an integrity signature.

Persisted identifiers MUST match `[a-z][a-z0-9_]{0,63}` and versions MUST be positive u32. The parameter record MUST obey the closed finite scalar-tween shape and existing primitive bounds. Valid historical identifiers and versions fitting that descriptive shape MUST NOT require live compiler support. Project reading, undo/redo, evaluation and rendering MUST NOT compile the provenance, substitute current catalog output, or require equality to today's expansion. A new descriptive parameter shape MUST require explicit schema compatibility work.

#### Scenario: Save effective parameters and primitives together
- **WHEN** a successful request omits optional curve and collision policy
- **THEN** its stored primitives and source record are published together, parameters contain `curve: "linear"`, and existing channels remain the authoritative output

#### Scenario: Reopen a retired source version
- **WHEN** a schema-29 project and retained snapshots have valid resolved channels and a structurally valid source identifier/version absent from the live compiler catalog
- **THEN** open, undo/redo and rendering use the saved channels and preserve source records without compiler lookup or resource loading

#### Scenario: Reject malformed descriptive state
- **WHEN** source data is null, has unknown fields, invalid identifiers or versions, non-finite parameters, a mismatched property, or an orphan identity
- **THEN** core fails validation using its established typed persisted-input failure path and publishes no repaired/defaulted generation

### Requirement: Provenance follows primitive lifecycle
Successful preset replacement MUST replace that identity's source record without accumulating an application log. `set_animation_channels` MUST clear all source records on its item, including when its replacement is byte-equivalent or empty. Other accepted operations changing/removing/retiming an authored channel MUST clear that identity's source record; operations preserving its source primitives and effective source clock under animation-edit-semantics MUST preserve its known source record. Raw full component track/document inputs MUST NOT introduce trusted source labels: new definitions MUST start without them, and replacements MUST retain only previously known records for the same scope/item/identity with unchanged complete channels, including any retained clock. Exact core copies MUST retain source records with equivalent source primitives and effective source clocks; materializing an implicit zero clock on a duplicate is equivalent and MUST NOT change original compilation attribution. Deleting an item MUST delete its records. Undo/redo MUST restore primitives and records together. Ordinary unclocked authored-channel updates MUST retain existing item-duration key bounds and compiler half-open timing constraints; only approved split, trim and duplicate operations may implicitly materialize validated retained clocks under animation-edit-semantics. Split and trim MUST follow animation-edit-semantics by preserving exact source keys, curves, loops and descriptive original compilation attribution while composing validated retained source clocks; retained channel keys MUST satisfy their effective source-duration bounds even when outside the shorter edited item window. This MUST NOT automatically retime source keys, validate malformed source records, relax edit boundaries or bypass candidate safety constraints. Newly compiled or replaced preset channels MUST use fresh item-local timing without retained clocks; unrelated typed channels and legacy animation MUST retain their existing independent clocks.

#### Scenario: Clear labels through the raw setter and restore by undo
- **WHEN** an item with preset source records receives `set_animation_channels`, including an identical or empty replacement, then undo and redo
- **THEN** the setter clears all its labels, undo restores exact prior primitives/labels, and redo returns to the unlabeled replacement

#### Scenario: Preserve copies and unrelated edits
- **WHEN** core makes an exact duplicate or a move, static/base-volume, visibility or parenting edit leaves source channels and effective source clocks unchanged
- **THEN** known source records remain associated with equivalent source primitives and effective source clocks without recompilation

#### Scenario: Reconcile raw component replacement
- **WHEN** a raw component document changes one previously labeled channel, preserves another, or submits labels on new items
- **THEN** only previously known labels for unchanged scoped identities survive and submitted labels cannot label newly authored output

#### Scenario: Preserve existing duration and split outcomes
- **WHEN** duration or split violates edit boundaries, ordinary unclocked authored-channel update bounds, retained source-clock/source-key bounds or candidate safety rules, or an accepted operation changes an authored primitive
- **THEN** the invalid edit fails atomically or the changed primitive's label is cleared, without automatic retiming

#### Scenario: Preserve preset source attribution through retained edits
- **WHEN** a supported split or trim shortens a preset-generated item window while its original source keys remain valid against the retained source duration, or core duplicates that animated item
- **THEN** the edit preserves exact source keys, curves, loops and original descriptive compilation parameters, composes or copies equivalent effective source clocks, and samples preserved source values without recompilation or clearing attribution

#### Scenario: Compile one replacement on its fresh local clock
- **WHEN** a preset replaces one property on an item with retained typed and legacy animation clocks
- **THEN** only the replaced property receives newly compiled item-local primitives and source attribution with no retained clock, while unrelated typed channels and legacy keys retain their exact clocks and records

### Requirement: Transactional preset edits
`apply_animation_preset` MUST work through existing standalone edit and ordered batch paths with optimistic revision checks, project locking, track locks, stable missing-reference failures and one atomic project/history publication. Batch application MUST resolve an earlier item's `@alias` and MUST NOT be an ID-creation operation. The final candidate MUST undergo existing channel/reference and inherited/extended safety preflight before commit. Any failure MUST publish none of the batch, return no committed alias mapping and preserve current state, history, revision, existing drafts and managed resources. Existing interrupted-transaction recovery and post-commit warning semantics MUST remain unchanged.

#### Scenario: Apply to an earlier creation alias
- **WHEN** a batch creates a compatible item with `resultAlias`, then applies a valid preset to `@alias`
- **THEN** the alias resolves to the created ID, all edits publish as one revision and one undo entry, and undo/redo restore the complete state including provenance

#### Scenario: Roll back after earlier valid operations
- **WHEN** an ordered batch has valid earlier edits followed by a colliding or invalid preset, or later candidate safety fails
- **THEN** the complete batch fails with the established typed error and no project/history/draft/resource change

#### Scenario: Preserve canonical stale, missing and lock failures
- **WHEN** the expected revision is stale, item/asset is missing, track is locked, an alias is referenced before creation, or the preset declares `resultAlias`
- **THEN** core preserves the existing respective `REVISION_CONFLICT`, `ITEM_NOT_FOUND`/`ASSET_NOT_FOUND`, `TRACK_LOCKED` or `VALIDATION_FAILED` code and retryability without a mutation

#### Scenario: Reject before publishing legacy migration
- **WHEN** a supported legacy project and retained history require migration and a preset or preset-containing ordered batch is rejected for invalid input, collision, alias, track/target, or complete-candidate safety
- **THEN** the established typed error is returned and this request changes no current/history/draft bytes, revision, or managed resource bytes

#### Scenario: Accept legacy migration and edit together
- **WHEN** a valid preset or preset-containing ordered batch edits supported legacy current/history/drafts
- **THEN** migration and the complete edit publish through the existing journal as one revision and undo entry, and undo/redo/reopen preserve migrated snapshots and provenance

#### Scenario: Preserve preset publication fault semantics
- **WHEN** resource publication or transaction persistence fails before journal commit, or a checkpoint fails after the journal commits
- **THEN** pre-commit failure preserves authoritative documents and preexisting managed bytes and removes only new uncommitted resources, while committed recovery-pending behavior and reopen recovery remain unchanged

#### Scenario: Preserve preexisting resource destination entries
- **WHEN** a preset transaction reaches migration asset staging or font publication and the respective planned managed destination has a dangling symlink or other invalid preexisting entry
- **THEN** the request fails with the established integrity error before overwriting that entry, and rollback preserves its link target and all preexisting project/history/draft/resource bytes

#### Scenario: Preserve migration asset font selection behavior
- **WHEN** a preset-containing request selects an extensionless content-addressed migration asset as an explicit or configured-default font source
- **THEN** the existing font owner returns nonretryable DEPENDENCY_UNAVAILABLE rather than accepting a pinned fallback, and all project/history/draft/resource bytes remain unchanged

### Requirement: Bounded safe preset expansion
Core MUST reject non-finite values, wrong tags, unknown fields, out-of-range parameters, unsafe or overflowing timing and violations of canonical candidate bounds with `INVALID_ARGUMENT`. Each seed application MUST expand to at most one channel and two keys; source IDs MUST be at most 64 ASCII bytes; each item MUST hold at most 64 source entries, each for one existing channel identity. The closed parameter shape MUST accept no arbitrary maps, expressions, external resources, paths or executable SVG. The existing 100-operation batch, 64-channel, 1,000-keyframe, retained-history, scene, raster and 65,536-node extended-certification limits MUST remain unchanged. Reapplication MUST replace one source record rather than grow history outside retained project snapshots.

#### Scenario: Reject each numeric and shape boundary
- **WHEN** parameters are non-finite, outside a property/curve bound, have an unknown field, exceed safe integer timing, or fail checked addition
- **THEN** core returns `INVALID_ARGUMENT` with unchanged authoritative state, including direct native/headless callers

#### Scenario: Enforce merged and retained candidate limits
- **WHEN** a valid seed would add a sixty-fifth channel, the source map is excessive/orphaned, or complete retained/expanded scene work violates existing bounds
- **THEN** core rejects the final candidate before durable or resource publication rather than bypassing the existing budgets

### Requirement: Primitive-only shared evaluated behavior
All render intents MUST evaluate saved canonical primitives independently of provenance or the live preset catalog. Equivalent manually authored channels and preset-compiled channels MUST produce the same evaluated values and retain the existing visual/audio tolerances in frame preview, range preview, ordinary draft preview and export. Projects without source records MUST retain their current output, including inherited fractional timing, loop sampling, extended channels and the exact motion-blur midpoint correction. Evaluation MUST remain read-only with respect to project/history.

#### Scenario: Compare a preset with independently authored primitives
- **WHEN** frame, range, draft and export sample a preset-created project and one with independently fixed equivalent channel keys at boundary/interior/inherited times
- **THEN** sampled values agree within existing scalar tolerance and rendered visual/audio outputs retain documented parity without project/history writes

#### Scenario: Preserve legacy and midpoint behavior
- **WHEN** an untagged project or the existing 500 ms / 25 FPS / 360-degree / five-sample motion-blur case is evaluated on the feature branch
- **THEN** old animation/render behavior remains equal and exact midpoint samples remain `[484, 492, 500, 508, 516]`
