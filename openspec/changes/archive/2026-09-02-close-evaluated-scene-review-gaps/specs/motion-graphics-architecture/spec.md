## MODIFIED Requirements

### Requirement: Deterministic flat scene timing and ordering
Flat scene evaluation MUST use integer-millisecond half-open intervals, MUST preserve the current top-left pixel coordinate behavior, MUST omit hidden tracks and hidden items, and MUST order current visual instructions bottom-to-top by ascending track array index and then ascending item array index. Logical resource requests and audio instructions MUST use deterministic first-use order. Transition endpoint facts MUST preserve transition declaration order, MUST emit an `Out` fact for a source endpoint and an `In` fact for a target endpoint including both facts when the endpoints are the same item, and transition, mute, fade, automation, and ducking facts MUST resolve without consulting renderer-specific behavior. Merged voiceover activity intervals MUST be stored once at scene scope in deterministic order, and each ducked audio layer MUST retain only its gain, attack, and release settings while consuming that shared table.

#### Scenario: Order equal flat layers
- **WHEN** visible flat items occur across multiple tracks and item positions without a future explicit z-index
- **THEN** evaluated visual instructions are ordered by track array index and item array index and retain a stable item identifier as identity without reordering by hash-map or filesystem iteration

#### Scenario: Exclude hidden content
- **WHEN** a track or item is hidden, or an audio-bearing track or media item is muted
- **THEN** the hidden visual contributes no visual instruction and the muted source contributes no audio instruction while other visible instructions retain their relative order

#### Scenario: Resolve flat audio behavior
- **WHEN** visible media has audio, volume keyframes, fades, track roles, or ducking settings
- **THEN** evaluation produces deterministic audio instructions and a single scene-level table of resolved voiceover intervals using the same timing and gain semantics as the current renderer

#### Scenario: Share voiceover intervals across ducked layers
- **WHEN** multiple music layers enable ducking while voiceover activity exists
- **THEN** every ducked layer retains its own gain, attack, and release settings without owning a cloned interval vector, and all use the one scene-level merged interval table

#### Scenario: Index transition endpoint facts deterministically
- **WHEN** visible items reference transitions including a transition whose source and target are the same item
- **THEN** evaluation indexes the transition list once, preserves declaration order for each endpoint, and emits both the source `Out` and target `In` facts for that item

### Requirement: Bounded and fail-closed scene evaluation
Editor-core MUST reject evaluated values that are non-finite, intervals that are not valid non-empty half-open ranges, missing logical asset references, non-image media source intervals whose checked end overflows or exceeds a known asset duration, and work exceeding the named inclusive limits of 4,096 visual layers, 4,096 logical media resources, 4,096 audio layers, 4,096 emitted transition endpoint facts, 10,000 keyframes per property channel, or 10,000 positive voiceover activity ranges before merging. Evaluation MUST return `ASSET_NOT_FOUND` for a missing media asset and `INVALID_ARGUMENT` for invalid values, timing, or complexity. Referenced assets MUST be checked before source timing and complexity rejection; source timing and all named complexity limits MUST be checked before output collection allocation, scene-level voiceover interval allocation, path resolution, graphics rasterization, FFmpeg execution, or artifact publication.

#### Scenario: Reject a missing media reference
- **WHEN** a visible media item references an asset absent from the evaluated project revision, including when its source timing is invalid or other evaluated work exceeds a complexity limit
- **THEN** evaluation fails with `ASSET_NOT_FOUND`, produces no partial scene, performs no renderer or filesystem I/O, and leaves project state and history unchanged

#### Scenario: Reject a non-finite evaluated value
- **WHEN** any transform, opacity, audio gain, automation value, or derived evaluated numeric value is non-finite
- **THEN** evaluation fails with `INVALID_ARGUMENT` before producing a scene or invoking any downstream adapter

#### Scenario: Enforce each scene complexity boundary
- **WHEN** evaluation encounters exactly a named inclusive limit it succeeds, and when it encounters one additional visual layer, logical resource, audio layer, emitted transition endpoint fact, per-property-channel keyframe, or positive pre-merge voiceover activity range
- **THEN** the overflow fails with `INVALID_ARGUMENT` before allocating or publishing a partial downstream render result

#### Scenario: Preflight voiceover activity
- **WHEN** voiceover volume automation produces 10,000 positive activity ranges before merging
- **THEN** evaluation succeeds and creates one deterministically merged scene-level interval table
- **WHEN** it produces 10,001 positive activity ranges before merging
- **THEN** evaluation fails with `INVALID_ARGUMENT` before allocating the scene-level interval table

#### Scenario: Validate non-image source timing
- **WHEN** a video or audio source end is representable and equals its known asset duration, or equals `u64::MAX` when asset duration is unknown
- **THEN** evaluation accepts the source interval
- **WHEN** the checked source end overflows or exceeds a known asset duration by any amount
- **THEN** evaluation fails with `INVALID_ARGUMENT` before complexity checks or output allocation

#### Scenario: Preserve image source-offset behavior
- **WHEN** an image media item has a source offset that would overflow if added to its timeline duration
- **THEN** evaluation preserves current image behavior and does not reject it on source timing because image rendering ignores the offset

#### Scenario: Preflight voiceover keyframes
- **WHEN** a voiceover media item contains more than 10,000 keyframes in one property channel
- **THEN** evaluation fails with `INVALID_ARGUMENT` before deriving voiceover intervals or allocating an output keyframe collection

#### Scenario: Reject invalid evaluated timing
- **WHEN** an evaluated flat instruction would have an empty, reversed, or overflowing integer-millisecond interval
- **THEN** evaluation fails with `INVALID_ARGUMENT` instead of saturating or passing ambiguous timing to a renderer
