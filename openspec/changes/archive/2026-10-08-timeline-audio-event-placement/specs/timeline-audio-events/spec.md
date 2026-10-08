## ADDED Requirements

### Requirement: Bounded deterministic semantic audio placement
Core SHALL expose timeline_add_audio_event with scope, trackId, event and closed at TimeExpression; optional durationMs SHALL default to the selected asset's known positive safe duration, gainDb SHALL default0 and variantSeed SHALL default the definition seed. It MUST select existing definition variants by seed modulo ordered count and place source0 on an unlocked audio track in root or component:<existing ID>. A schema41 type:media item MUST retain selected asset plus optional nonnull closed audioEvent provenance requiring event, gainDb, captured defaultGainDb, busId, variantSeed, variantIndex and contentHash. Gain values MUST independently be finite[-120,24], seed/intervals JavaScript-safe, index0..31, duration positive and source range within recorded asset duration; content MUST be eligible managed audio or video-with-audio with the canonical sha256. Complexity SHALL retain existing item/composition and definition limits. Missing event or malformed inputs MUST use INVALID_ARGUMENT, missing track TRACK_NOT_FOUND, locked track TRACK_LOCKED, missing scoped component/marker ITEM_NOT_FOUND. Extra nested fields, null provenance, executable/path/network inputs MUST fail closed.

#### Scenario: Place a deterministic absolute event
- **WHEN** a registered event is placed at a valid absolute time with a saved or explicit seed and optional truncated duration
- **THEN** the selected asset, immutable gain/bus/content provenance and exact numeric interval are returned as one ordinary inspectable media item

#### Scenario: Reject bounds and missing references
- **WHEN** definition, scope, track, lock, asset duration, gains, seed, fields or interval are invalid
- **THEN** the specified nonretryable error occurs before publication and all previous generation/resource bytes remain exact

#### Scenario: Preserve placed selection after replacement
- **WHEN** a placed event's definition is replaced with different variants/default gain/bus/seed
- **THEN** existing items retain captured content/settings and future placements use the replacement

### Requirement: Scoped live marker timing for audio events
Audio placement SHALL accept the existing closed milliseconds and marker variants, retain markerName/offsetMs through existing startTime and synchronize numeric startMs by exact containing-scope lookup. No cross-scope fallback SHALL occur. Missing/ambiguous names, unsafe signed offsets, negative starts or out-of-bounds component intervals MUST fail with existing marker errors. Marker moves, rename/delete, numeric time edits, duplicate/split and undo/redo SHALL use the same canonical timing rules as other items.

#### Scenario: Move a root and component cue
- **WHEN** an event references one same-scope marker with a signed offset and that marker moves
- **THEN** effective event timing updates deterministically without altering selected content/gain and another scope's same name is ignored

#### Scenario: Reject ambiguous missing and unsafe cues
- **WHEN** no unique local marker exists or its offset/start/end violates existing safe/component bounds
- **THEN** the entire candidate fails unchanged using stable ITEM_NOT_FOUND or INVALID_ARGUMENT

### Requirement: Atomic alias-aware audio-event edits
Standalone, ordered timeline_batch_edit and durable-draft placement SHALL use the existing prepared core transaction, support preceding scope/track/event aliases and resultAlias for created item IDs, one logical revision/history entry, whole-batch rollback, optimistic retryable REVISION_CONFLICT precedence and existing VALIDATION_FAILED aliases. Draft validation/preview/rebase/commit, undo/redo, reopen and publication-fault recovery MUST preserve exact metadata/timing/managed resources.

#### Scenario: Place and address a created event through aliases
- **WHEN** a batch registers a definition, creates a scoped audio track and places an event then addresses its result alias
- **THEN** existing ordered alias resolution commits all edits as one revision and history entry

#### Scenario: Conflict late failure and publication fault
- **WHEN** a stale request, invalid later batch operation or injected precommit publication phase rejects the candidate, or an existing publication fault occurs after the durable commit point
- **THEN** rejected precommit candidates retain all prior project/history/draft/resource bytes and original error precedence, while postcommit faults preserve and recover the exact complete target generation under the existing warning/recovery rules without treating it as rejected

#### Scenario: Commit and restore a materialized draft
- **WHEN** a valid placement draft is previewed/committed then undone/redone/reopened
- **THEN** all surfaces retain the exact variant snapshot and marker expression under shared core validation

### Requirement: Shared deterministic audio-only event evaluation
The canonical scene SHALL evaluate event-bearing media as audio-only, including video-bearing variants, and apply audio.volume * 10^((captured defaultGainDb+gainDb)/20) once while retaining existing fades/mute/volume animation/source timing and role ducking. Preview/draft/export MUST share that behavior and existing SSIM>=0.99, aligned float-PCM RMS<=0.0001 and one-frame bounds. Captured bus identity SHALL remain metadata until later approved DSP activation. Projects without audioEvent metadata MUST retain exact evaluated plans, filter graphs, RGB and PCM.

#### Scenario: Compare native event and equivalent media
- **WHEN** fixed audio and audio-bearing video are placed via semantic events and equivalent ordinary audio-only media with the same effective gain/timing
- **THEN** native preview/export/draft match original RGB/PCM/timing oracles and video events do not add visual layers

#### Scenario: Preserve legacy role and render behavior
- **WHEN** a project contains only ordinary media or a definition is registered/replaced without placement
- **THEN** every existing plan/filter/ducking and native pixel/audio result remains exact without claiming bus DSP support

### Requirement: Independently governed additive placement contracts
Protocol1 SHALL retain every old contract while adding timeline_add_audio_event, timeline_audio_events_v1 support and schema41 reporting. Canonical timeline-audio-events-v1, Rust, Zod/MCP, headless, drafts and persisted responses MUST agree. Seven active current-schema headers MUST advance exactly40→41; all frozen catalogs/counts/original assertions SHALL remain independently pinned. Projection MUST permit only exact independently captured new surfaces, rejecting unrelated schema/annotation/error/capability drift. Every original full contract/integration/package/native consumer MUST remain mandatory.

#### Scenario: Exercise real public placement transports
- **WHEN** real MCP/headless clients place events standalone and in aliased batches/drafts
- **THEN** canonical contracts agree with stable core errors, success, revision/history and reopen behavior

#### Scenario: Detect unrelated contract mutations
- **WHEN** any old operation/error/schema/annotation or frozen feature proof is changed beyond approved additions
- **THEN** independent predecessor and ownership negative controls reject the mutation without updating old oracles
