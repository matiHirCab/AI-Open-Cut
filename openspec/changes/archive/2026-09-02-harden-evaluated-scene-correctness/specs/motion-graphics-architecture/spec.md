## ADDED Requirements

### Requirement: Process-local scene resource bindings
Editor-core MUST return evaluated scene semantics with a separate process-local resource-binding collection that maps logical media resources to project-relative media requests and configured logical font resources to their original path and family selection request. Raw requested paths MUST remain outside `EvaluatedScene`, resolved filesystem paths MUST remain owned by the existing path-safe preparation layer, and neither collection SHALL become a public or persisted contract in this milestone.

#### Scenario: Preserve configured font selection
- **WHEN** text configures a font path, a font family, or both
- **THEN** evaluation assigns a logical font resource identifier and records the original selection in the separate resource-binding collection without placing a path or URL in `EvaluatedScene`

#### Scenario: Keep default font renderer-configured
- **WHEN** text configures neither a font path nor a font family
- **THEN** evaluation emits no font resource binding and downstream preparation remains responsible for selecting its configured default font

#### Scenario: Preserve path-safe preparation
- **WHEN** a later render-routing milestone consumes a requested font path from the resource-binding collection
- **THEN** the request is canonicalized and checked against configured font roots before any resolved filesystem path reaches a backend

## MODIFIED Requirements

### Requirement: Deterministic flat scene timing and ordering
Flat scene evaluation MUST use integer-millisecond half-open intervals, MUST preserve the current top-left pixel coordinate behavior, MUST omit hidden tracks and hidden items, and MUST order current visual instructions bottom-to-top by ascending track array index and then ascending item array index. Logical resource requests and audio instructions MUST use deterministic first-use order. Transition endpoint facts MUST preserve transition declaration order, MUST emit an `Out` fact for a source endpoint and an `In` fact for a target endpoint including both facts when the endpoints are the same item, and transition, mute, fade, automation, and ducking facts MUST resolve without consulting renderer-specific behavior.

#### Scenario: Order equal flat layers
- **WHEN** visible flat items occur across multiple tracks and item positions without a future explicit z-index
- **THEN** evaluated visual instructions are ordered by track array index and item array index and retain a stable item identifier as identity without reordering by hash-map or filesystem iteration

#### Scenario: Exclude hidden content
- **WHEN** a track or item is hidden, or an audio-bearing track or media item is muted
- **THEN** the hidden visual contributes no visual instruction and the muted source contributes no audio instruction while other visible instructions retain their relative order

#### Scenario: Resolve flat audio behavior
- **WHEN** visible media has audio, volume keyframes, fades, track roles, or ducking settings
- **THEN** evaluation produces deterministic audio instructions and resolved voiceover intervals using the same timing and gain semantics as the current renderer

#### Scenario: Index transition endpoint facts deterministically
- **WHEN** visible items reference transitions including a transition whose source and target are the same item
- **THEN** evaluation indexes the transition list once, preserves declaration order for each endpoint, and emits both the source `Out` and target `In` facts for that item

### Requirement: Bounded and fail-closed scene evaluation
Editor-core MUST reject evaluated values that are non-finite, intervals that are not valid non-empty half-open ranges, missing logical asset references, and work exceeding the named inclusive limits of 4,096 visual layers, 4,096 logical media resources, 4,096 audio layers, 4,096 emitted transition endpoint facts, or 10,000 keyframes per property channel. Evaluation MUST return `ASSET_NOT_FOUND` for a missing media asset and `INVALID_ARGUMENT` for invalid values, timing, or complexity. Referenced assets MUST be checked before complexity rejection, and all named complexity limits MUST be checked before output collection allocation, voiceover interval derivation, path resolution, graphics rasterization, FFmpeg execution, or artifact publication.

#### Scenario: Reject a missing media reference
- **WHEN** a visible media item references an asset absent from the evaluated project revision, including when other evaluated work exceeds a complexity limit
- **THEN** evaluation fails with `ASSET_NOT_FOUND`, produces no partial scene, performs no renderer or filesystem I/O, and leaves project state and history unchanged

#### Scenario: Reject a non-finite evaluated value
- **WHEN** any transform, opacity, audio gain, automation value, or derived evaluated numeric value is non-finite
- **THEN** evaluation fails with `INVALID_ARGUMENT` before producing a scene or invoking any downstream adapter

#### Scenario: Enforce each scene complexity boundary
- **WHEN** evaluation encounters exactly a named inclusive limit it succeeds, and when it encounters one additional visual layer, logical resource, audio layer, emitted transition endpoint fact, or per-property-channel keyframe
- **THEN** the overflow fails with `INVALID_ARGUMENT` before allocating or publishing a partial downstream render result

#### Scenario: Preflight voiceover keyframes
- **WHEN** a voiceover media item contains more than 10,000 keyframes in one property channel
- **THEN** evaluation fails with `INVALID_ARGUMENT` before deriving voiceover intervals or allocating an output keyframe collection

#### Scenario: Reject invalid evaluated timing
- **WHEN** an evaluated flat instruction would have an empty, reversed, or overflowing integer-millisecond interval
- **THEN** evaluation fails with `INVALID_ARGUMENT` instead of saturating or passing ambiguous timing to a renderer
