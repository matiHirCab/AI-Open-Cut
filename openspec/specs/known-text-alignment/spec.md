# Known-Text Alignment Specification

## Purpose

Define bounded provider-neutral alignment of supplied text to managed local audio through the existing transcription lifecycle.

## Requirements

### Requirement: Bounded core-owned known-text alignment validation
Core SHALL expose additive read-only validate_speech_alignment over an existing project audio asset and nonblank knownText of at most4096 UTF-8 bytes. It MUST resolve source media through existing path/integrity policy, require known positive probed duration, preserve text, validate optional expectedRevision and validate optional forced-quality alignment through the existing canonical intrinsic/asset-duration rules. It MUST NOT add timeline, history, asset-provenance or render mutations; existing ordinary project-open migration/recovery semantics remain unchanged.

#### Scenario: Validate a known-text source and alignment
- **WHEN** a valid existing probed audio source and bounded known text are supplied, with or without a valid forced record
- **THEN** core returns the existing resolved source descriptor and accepts the record without changing authoritative project/history/media bytes

#### Scenario: Reject invalid text or alignment
- **WHEN** known text is blank/over its UTF-8 limit or alignment is malformed, non-forced, unordered, excessive, unsafe or beyond asset duration
- **THEN** core returns the existing typed nonretryable validation/decoding failure without publication

#### Scenario: Reject missing or stale source
- **WHEN** project/asset is missing, media is not probed audio or expectedRevision is stale
- **THEN** existing PROJECT_NOT_FOUND, ASSET_NOT_FOUND, UNSUPPORTED_MEDIA, VALIDATION_FAILED or REVISION_CONFLICT semantics apply without partial edits

### Requirement: Truthful provider-neutral known-text support
Transcription status SHALL independently advertise known-text support, sentence/word/phoneme support and explicit duration/text limits. Legacy omission MUST normalize to unsupported, and explicit closed metadata MUST be validated without inferring support from ordinary transcription, readiness or synthesis. KnownText omission SHALL retain existing transcription behavior; unsupported known-text mode MUST fail before inference with TRANSCRIPTION_UNAVAILABLE.

#### Scenario: Discover a legacy provider
- **WHEN** an otherwise valid provider omits known-text metadata
- **THEN** ordinary transcription remains available and known-text support is false

#### Scenario: Inspect independent supported granularity
- **WHEN** the local known-token adapter reports support
- **THEN** word support is true, sentence/phoneme support false and limits are30 seconds/4096 UTF-8 bytes without changing speech-provider timestampSupport

#### Scenario: Reject malformed or unavailable support
- **WHEN** explicit metadata is malformed or a provider is unsupported/not ready or lacks its advertised align method
- **THEN** typed failure occurs before inference/token publication without changing the project

### Requirement: Actual bounded local known-token alignment
The local provider SHALL align caller-known text tokens against encoded local audio using the already pinned model alignment implementation, with forced quality and truthful producer/model identity. It MUST share the existing bounded FIFO/concurrency-one queue, timeout and cancellation with transcription. It MUST enforce advertised duration/text and model-token bounds and MUST NOT label recognized or evenly spaced estimated text as forced alignment or fabricate spans. Provider output MUST remain finite, ordered, positive and within actual/source duration, with no client-visible absolute media path. Known-text language omission SHALL select en without changing ordinary language detection.

#### Scenario: Align supplied text through the model boundary
- **WHEN** bounded local audio and known text are aligned
- **THEN** the model receives the supplied text tokens and audio encoding and returns word timing with forced producer provenance instead of recognizing alternate text

#### Scenario: Reject model or output limits
- **WHEN** decoded audio/text tokens exceed the advertised/model budget or alignment is empty/non-finite/degenerate/out of bounds
- **THEN** inference/retention fails with existing sanitized typed failure and no fabricated timing or project mutation

#### Scenario: Preserve queue lifecycle
- **WHEN** alignment and transcription compete, overflow, timeout, cancel or close
- **THEN** existing FIFO, concurrency-one, bounded overload, cancellation, timeout and shutdown semantics apply to both modes

### Requirement: Isolated inference worker recovery
The provider MUST retire cancelled or timed-out workers before dispatching queued or immediate successor inference, preserving concurrency-one FIFO and existing JOB_CANCELLED or TRANSCRIPTION_TIMEOUT codes/retryability. Output and lifecycle callbacks from a retired worker MUST NOT reject or corrupt another worker's requests. Alignment and ordinary transcription MUST retain their existing compatible output contracts.

#### Scenario: Cancel and reuse inference
- **WHEN** active alignment or transcription is cancelled with a valid successor already queued or immediately submitted after rejection
- **THEN** cancellation remains typed and the successor completes on a healthy worker without a caller delay or overlapping inference

#### Scenario: Time out and reuse inference
- **WHEN** active inference exceeds its deadline with a valid queued or immediate successor
- **THEN** only expired work fails with TRANSCRIPTION_TIMEOUT and the successor succeeds after terminated-worker cleanup

### Requirement: Graceful direct provider disposal
Direct provider close SHALL reject new work and gracefully drain active inference before terminating/reaping its worker. Bridge signal shutdown MUST cancel cancellable jobs before disposing the provider, retaining existing application shutdown order and typed cancellation.

#### Scenario: Close an active provider directly
- **WHEN** direct close begins during finite active inference
- **THEN** the active operation completes, new work is unavailable, and close waits for worker cleanup

#### Scenario: Shut down the bridge during inference
- **WHEN** the bridge receives its supported shutdown signal during cancellable inference
- **THEN** jobs are aborted before graceful provider disposal and the bridge and provider worker terminate without waiting for the inference deadline
