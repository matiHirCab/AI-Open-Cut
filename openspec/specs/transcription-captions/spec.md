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
Committing a transcription preview MUST validate the current project revision, target asset, caption track, ordered non-overlapping segment timing, word bounds, confidence values, and caption style, then SHALL publish all caption items as one project revision.

#### Scenario: Commit captions
- **WHEN** a caller commits a valid preview token at the current revision
- **THEN** the core inserts the resulting captions as one revision and consumes the preview

#### Scenario: Reject a stale caption commit
- **WHEN** the project revision changed before preview commit
- **THEN** commit fails with `REVISION_CONFLICT` and does not partially insert captions

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
