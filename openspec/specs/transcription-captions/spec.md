# Transcription and Captions Specification

## Purpose

Define local provider discovery, estimation, preview retention, timestamp validation, and atomic caption insertion behavior.

## Requirements

### Requirement: Provider-neutral transcription status
The bridge SHALL report transcription provider identity, model, device, compute type, cache/load state, duration limits, queue state, and readiness without exposing project internals to the worker.

#### Scenario: Inspect transcription readiness
- **WHEN** a client requests transcription status
- **THEN** the response distinguishes provider availability and model readiness from the rest of the editor

### Requirement: Safe media resolution and estimation
Transcription MUST resolve project assets through the headless path policy and require probed audio with a positive duration. Estimation SHALL report that duration without starting inference, while preview creation MUST reject durations above the provider's advertised maximum.

#### Scenario: Estimate an asset transcription
- **WHEN** a client selects a valid project media asset
- **THEN** the bridge returns provider, duration, model-cache, cost, language, and queue information without exposing the provider-only absolute media path

#### Scenario: Reject an overlong preview
- **WHEN** a client starts transcription preview for media whose probed duration exceeds the provider maximum
- **THEN** preview creation fails with `VALIDATION_FAILED` before inference starts

#### Scenario: Reject disallowed transcription input
- **WHEN** selected media cannot be resolved through an allowed project asset
- **THEN** transcription fails before the worker receives the path

### Requirement: Timestamped preview retention
The transcription provider SHALL return timestamped segments with optional word timestamps, and the bridge SHALL validate their wire shape and retain them behind an expiring opaque preview token without modifying the project.

#### Scenario: Create a transcription preview
- **WHEN** local transcription completes with valid ordered timestamps
- **THEN** the client receives the language, segments, expiration, and token while the project revision remains unchanged

### Requirement: Atomic caption commit
Committing a transcription preview MUST validate the current project revision, target asset, caption track, ordered non-overlapping segment timing, word bounds, confidence values, and caption style, then SHALL publish all caption items with durable source-asset provenance as one project revision.

#### Scenario: Commit captions
- **WHEN** a caller commits a valid preview token for an existing asset at the current revision
- **THEN** the core inserts all resulting captions with the source asset identifier as one revision and consumes the preview

#### Scenario: Reject a stale caption commit
- **WHEN** the project revision changed before preview commit
- **THEN** commit fails with `REVISION_CONFLICT` and does not partially insert captions

#### Scenario: Reject a missing caption source
- **WHEN** a transcription commit names an asset absent from the current project
- **THEN** commit fails with `ASSET_NOT_FOUND` and no caption provenance is persisted

### Requirement: Durable caption-source lifecycle
Caption source provenance MUST remain resolvable while the caption is current or retained in undo/redo history, MUST block logical source-asset deletion while current, and MUST NOT be silently detached by deletion or garbage collection.

#### Scenario: Block deletion while captions are current
- **WHEN** one or more current captions retain provenance for a source asset
- **THEN** deleting that asset fails with `ASSET_IN_USE` and all captions and provenance remain unchanged

#### Scenario: Restore caption provenance through undo and redo
- **WHEN** an edit containing caption provenance is undone and subsequently redone within retained history
- **THEN** each restored caption resolves the same source asset and its managed content remains available

#### Scenario: Reopen caption provenance
- **WHEN** a valid captioned project is closed and reopened
- **THEN** every caption retains and resolves its original source asset identifier

#### Scenario: Reject legacy dangling caption provenance
- **WHEN** a reopened current or retained project snapshot contains caption provenance whose source asset is absent
- **THEN** open fails deterministically with `ASSET_INTEGRITY_FAILED` and identifies the caption-source reference

### Requirement: Bounded lifecycle and typed failures
Transcription work MUST use a bounded queue with timeout and cancellation, and previews MUST be explicitly discardable and automatically expire without mutating project state.

#### Scenario: Reject queue overload
- **WHEN** the transcription queue is full
- **THEN** new work fails with retryable `TRANSCRIPTION_QUEUE_FULL`

#### Scenario: Discard a transcription preview
- **WHEN** a caller discards an unexpired preview token
- **THEN** the preview is removed and the project remains unchanged

#### Scenario: Use an expired preview
- **WHEN** a caller addresses an expired or unknown preview token
- **THEN** the bridge returns `TRANSCRIPTION_PREVIEW_NOT_FOUND`
