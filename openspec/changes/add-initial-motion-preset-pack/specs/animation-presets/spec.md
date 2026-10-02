## MODIFIED Requirements

### Requirement: Versioned primitive compilation
Editor-core MUST own a pure, bounded compiler addressed by mandatory `presetId` and positive integer `presetVersion`. The compilation catalog MUST contain `scalar_tween` version 1 with compiler version 1 and the five version-1 initial motion pack entries specified by initial-motion-preset-pack with compiler version 2. Unknown identifiers, unsupported versions, missing versions and `latest` selection MUST fail with non-retryable `INVALID_ARGUMENT` before publication. Compilation MUST use no filesystem, network, executable content, renderer expression or transport-specific expansion. The compiler MUST produce the existing canonical channel primitives, with fully materialized effective parameters for provenance; it MUST NOT introduce new interpolation or rendering rules.

#### Scenario: Compile an explicitly versioned seed
- **WHEN** a caller applies supported `scalar_tween@1` with valid typed parameters to a compatible item
- **THEN** core produces the specified editable channel and effective source record with compiler version 1

#### Scenario: Reject unsupported compilation identity
- **WHEN** a request omits the version, uses `latest`, names an unknown preset, or requests an unsupported version
- **THEN** it fails with `INVALID_ARGUMENT` without changing project, revision, history, aliases or managed bytes

### Requirement: Explicit channel collision policy
Preset application MUST accept `collisionPolicy` equal to `reject` or `replace`, defaulting to `reject`. Collision MUST mean equality of canonical channel property and target identity, regardless of keyframe interval overlap. `reject` MUST fail with `INVALID_ARGUMENT` if any generated identity has an existing identical typed channel. Explicit `replace` MUST replace that complete channel, including its prior loop, in its existing collection position. A new identity MUST append one channel in compiler output order. Multi-channel expansion MUST be atomic across every generated identity; failure MUST publish no accepted prefix. Unrelated channels and their ordering, provenance, keyframes, targets and loops, static properties, effects and legacy keys MUST remain unchanged. Either policy MUST reject overlapping legacy keys using the existing canonical collision rule; preset application MUST NOT remove or convert legacy animation.

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
A successful preset edit MUST persist its resolved channel in `animationChannels` and one source record per generated identity in optional item `animationPresetProvenance`, keyed by the generated targetless property. Each record MUST contain `presetId`, `presetVersion`, `compilerVersion` and fully materialized parameters. Scalar-tween parameters MUST include the effective curve; tagged pack shapes MUST retain their complete explicit endpoints and timing, with fixed curves determined by the descriptive shape and stored channels. Omission MUST mean an empty map; `null` MUST be invalid; an empty map MUST serialize as omitted. Entries MUST correspond to existing targetless channel identities and matching scalar parameter properties or output-property membership for tagged pack parameters. Provenance MUST be descriptive rather than executable or an integrity signature.

Persisted identifiers MUST match `[a-z][a-z0-9_]{0,63}` and versions MUST be positive u32. The parameter record MUST obey the closed finite scalar-tween shape or one of the five strict tagged pack shapes defined by initial-motion-preset-pack and existing primitive bounds. Tagged shapes MUST require schema 30; scalar-tween sources MUST remain valid and unchanged. Each generated channel MUST receive the complete effective parameters. Persisted looped pack parameters MUST include effective iterations explicitly, and persisted impact parameters MUST include explicit motionBlur; absence MUST fail rather than invoke request defaults. Descriptive validation MUST NOT require all sibling output channels to exist after subsequent edits. Valid historical identifiers and versions fitting that descriptive shape MUST NOT require live compiler support. Project reading, undo/redo, evaluation and rendering MUST NOT compile the provenance, substitute current catalog output, or require equality to today's expansion. Further descriptive parameter shapes MUST require explicit schema compatibility work.

#### Scenario: Save effective parameters and primitives together
- **WHEN** a successful request omits optional curve and collision policy
- **THEN** its stored primitives and source record are published together, parameters contain `curve: "linear"`, and existing channels remain the authoritative output

#### Scenario: Reopen a retired source version
- **WHEN** a schema-29 project and retained snapshots have valid resolved channels and a structurally valid source identifier/version absent from the live compiler catalog
- **THEN** open, undo/redo and rendering use the saved channels and preserve source records without compiler lookup or resource loading

#### Scenario: Reject malformed descriptive state
- **WHEN** source data is null, has unknown fields, invalid identifiers or versions, non-finite parameters, a mismatched property, or an orphan identity
- **THEN** core fails validation using its established typed persisted-input failure path and publishes no repaired/defaulted generation

### Requirement: Bounded safe preset expansion
Core MUST reject non-finite values, wrong tags, unknown fields, out-of-range parameters, unsafe or overflowing timing and violations of canonical candidate bounds with `INVALID_ARGUMENT`. Each scalar seed application MUST expand to at most one channel and two keys; each pack application MUST expand to at most five channels and twenty-five total keys plus at most one existing MotionBlur assignment; source IDs MUST be at most 64 ASCII bytes; each item MUST hold at most 64 source entries, each for one existing channel identity. All closed parameter shapes MUST accept no arbitrary maps, expressions, external resources, paths or executable SVG. The existing 100-operation batch, 64-channel, 1,000-keyframe, retained-history, scene, raster and 65,536-node extended-certification limits MUST remain unchanged. Reapplication MUST replace each generated identity's source record rather than grow history outside retained project snapshots.

#### Scenario: Reject each numeric and shape boundary
- **WHEN** parameters are non-finite, outside a property/curve bound, have an unknown field, exceed safe integer timing, or fail checked addition
- **THEN** core returns `INVALID_ARGUMENT` with unchanged authoritative state, including direct native/headless callers

#### Scenario: Enforce merged and retained candidate limits
- **WHEN** a valid seed would add a sixty-fifth channel, the source map is excessive/orphaned, or complete retained/expanded scene work violates existing bounds
- **THEN** core rejects the final candidate before durable or resource publication rather than bypassing the existing budgets

## ADDED Requirements

### Requirement: Motion-blur-aware impact provenance lifecycle
Accepted changes/removal through existing edit paths (including raw component replacement) of an item's MotionBlur MUST clear all source records using the impact_slam parameter shape on that item. Raw component replacements MUST preserve those known source records only when both the same identity's channel and associated blur remain byte-equivalent. Read/retirement validation MUST remain descriptive without requiring equality to current runtime blur or live catalog identity/version. Exact copies, undo/redo and unrelated edits MUST retain existing source behavior.

#### Scenario: Change blur and undo
- **WHEN** a committed slam's MotionBlur is changed or removed through an accepted low-level edit, then undo/redo
- **THEN** impact-shaped source labels clear, undo restores exact channels/blur/sources, and redo restores the edited unlabeled output

#### Scenario: Reconcile blur-dependent raw sources
- **WHEN** raw component replacement preserves a channel but changes its associated blur
- **THEN** its known impact-shaped label is cleared while unrelated known source records remain governed by their existing unchanged-primitive rules
