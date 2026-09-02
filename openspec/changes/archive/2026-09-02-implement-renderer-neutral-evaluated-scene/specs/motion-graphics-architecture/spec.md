## ADDED Requirements

### Requirement: Concrete flat EvaluatedScene representation
Editor-core MUST provide an owned renderer-neutral `EvaluatedScene` whose header contains explicit canvas dimensions, frame rate, and integer-millisecond project duration; whose logical media resources contain no filesystem path or network locator; and whose closed typed instructions represent the current flat media, text, solid-color, rectangle, and associated audio behavior without containing persisted-record references, raw FFmpeg expressions, executable SVG, backend types, prepared files, or artifact destinations.

#### Scenario: Evaluate current flat items
- **WHEN** editor-core evaluates a validated project containing visible media, text, solid-color, and rectangle items
- **THEN** it produces owned typed resource and layer instructions containing the timing, transform, animation, styling, media, and audio facts needed to reproduce the existing output semantics

#### Scenario: Keep evaluation renderer-neutral
- **WHEN** a caller inspects any evaluated resource or instruction
- **THEN** it contains logical identifiers and typed finite values and contains no arbitrary path, network resource, renderer expression, executable content, prepared temporary file, backend command, or artifact destination

#### Scenario: Preserve the project snapshot
- **WHEN** evaluation succeeds or fails for an immutable project revision
- **THEN** the source project, revision, current state, retained undo/redo history, and persisted schema remain unchanged and repeated evaluation of the same inputs produces an equal scene or equal typed error

### Requirement: Deterministic flat scene timing and ordering
Flat scene evaluation MUST use integer-millisecond half-open intervals, MUST preserve the current top-left pixel coordinate behavior, MUST omit hidden tracks and hidden items, and MUST order current visual instructions bottom-to-top by ascending track array index and then ascending item array index. Logical resource requests and audio instructions MUST use deterministic first-use order, and transition, mute, fade, automation, and ducking facts MUST resolve without consulting renderer-specific behavior.

#### Scenario: Order equal flat layers
- **WHEN** visible flat items occur across multiple tracks and item positions without a future explicit z-index
- **THEN** evaluated visual instructions are ordered by track array index and item array index and retain a stable item identifier as identity without reordering by hash-map or filesystem iteration

#### Scenario: Exclude hidden content
- **WHEN** a track or item is hidden, or an audio-bearing track or media item is muted
- **THEN** the hidden visual contributes no visual instruction and the muted source contributes no audio instruction while other visible instructions retain their relative order

#### Scenario: Resolve flat audio behavior
- **WHEN** visible media has audio, volume keyframes, fades, track roles, or ducking settings
- **THEN** evaluation produces deterministic audio instructions and resolved voiceover intervals using the same timing and gain semantics as the current renderer

### Requirement: Bounded and fail-closed scene evaluation
Editor-core MUST reject evaluated values that are non-finite, intervals that are not valid non-empty half-open ranges, missing logical asset references, and work exceeding the named inclusive limits of 4,096 visual layers, 4,096 logical media resources, 4,096 audio layers, or 10,000 keyframes per property channel. Evaluation MUST return `ASSET_NOT_FOUND` for a missing media asset and `INVALID_ARGUMENT` for invalid values, timing, or complexity before path resolution, graphics rasterization, FFmpeg execution, or artifact publication.

#### Scenario: Reject a missing media reference
- **WHEN** a visible media item references an asset absent from the evaluated project revision
- **THEN** evaluation fails with `ASSET_NOT_FOUND`, produces no partial scene, performs no renderer or filesystem I/O, and leaves project state and history unchanged

#### Scenario: Reject a non-finite evaluated value
- **WHEN** any transform, opacity, audio gain, automation value, or derived evaluated numeric value is non-finite
- **THEN** evaluation fails with `INVALID_ARGUMENT` before producing a scene or invoking any downstream adapter

#### Scenario: Enforce each scene complexity boundary
- **WHEN** evaluation encounters exactly a named inclusive limit it succeeds, and when it encounters one additional visual layer, logical resource, audio layer, or per-property-channel keyframe
- **THEN** the overflow fails with `INVALID_ARGUMENT` before allocating or publishing a partial downstream render result

#### Scenario: Reject invalid evaluated timing
- **WHEN** an evaluated flat instruction would have an empty, reversed, or overflowing integer-millisecond interval
- **THEN** evaluation fails with `INVALID_ARGUMENT` instead of saturating or passing ambiguous timing to a renderer

### Requirement: EvaluatedScene foundation remains non-public and non-persisted
The initial `EvaluatedScene` implementation MUST remain an editor-core process-local derivation, MUST NOT change project schema version 6 or any public request, response, operation, capability, MCP, provider, or stable-error catalog, and MUST leave production render-entry routing and preview/export parity migration to the separately approved routing milestone.

#### Scenario: Reopen and history remain compatible
- **WHEN** a schema-version-6 project with retained undo and redo snapshots is evaluated and later reopened
- **THEN** no migration or rewrite occurs and current state plus every retained history snapshot reopens byte-for-byte under the existing persistence behavior

#### Scenario: Keep clients unchanged
- **WHEN** clients inspect headless operations, MCP tools, capabilities, canonical public fixtures, or error identifiers after this change
- **THEN** they observe no added or changed public surface for `EvaluatedScene`

#### Scenario: Defer render-entry routing
- **WHEN** this representation milestone is completed before the routing milestone
- **THEN** the new evaluator is independently testable while existing frame, range, draft, and export call sites retain their current behavior and output
