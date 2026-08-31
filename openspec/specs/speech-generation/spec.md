# Speech Generation Specification

## Purpose

Define provider-neutral speech discovery, estimation, generation, preview, commit, regeneration, provenance, and lifecycle behavior.

## Requirements

### Requirement: Provider-neutral speech capabilities
The bridge SHALL discover provider, model, language, voice, device, limit, resource, and availability metadata dynamically through the speech provider contract.

#### Scenario: List available voices
- **WHEN** a client requests speech voices
- **THEN** the bridge returns provider-described labels, locale, model, preview support, availability, and default status without a core-owned provider enum

### Requirement: Validated estimation and synthesis
Speech requests MUST validate text, language, voice, speed, normalization, pronunciation, chunking, and provider limits before synthesis, and SHALL expose duration, resource, and queue estimates.

#### Scenario: Estimate a valid request
- **WHEN** a client submits valid speech intent
- **THEN** the bridge returns bounded duration estimates, chunk count, provider resource information, and current queue state without generating media

#### Scenario: Reject unsupported speech intent
- **WHEN** a request exceeds provider limits or selects unsupported capabilities
- **THEN** the bridge returns a typed non-mutating failure before queueing inference

### Requirement: Preview before insertion
The bridge SHALL allow speech to be synthesized into an expiring opaque preview token without modifying the project and SHALL allow that preview to be explicitly committed or discarded.

#### Scenario: Preview and commit speech
- **WHEN** synthesis succeeds and the caller commits its token with a current revision and valid placement
- **THEN** the core atomically creates the generated asset and timeline item without rerunning inference

#### Scenario: Discard speech preview
- **WHEN** a caller discards a valid preview token
- **THEN** the retained audio is cleaned up and no project mutation occurs

### Requirement: Revision-conflict reuse
Completed synthesis MUST remain reusable for its retention period when commit encounters a revision conflict so the client can retry against refreshed state without repeating inference.

#### Scenario: Retry a conflicted commit
- **WHEN** preview commit fails with `REVISION_CONFLICT` and the client retries the retained token at the current revision
- **THEN** the bridge commits the existing generated artifact rather than synthesizing the text again

### Requirement: Persisted speech intent and regeneration
Committed speech assets MUST preserve provider-neutral request intent and generation provenance, and regeneration SHALL create a newly identified generated asset while preserving the existing timeline item identifier and replacing its asset reference in one project revision.

#### Scenario: Regenerate committed speech
- **WHEN** a caller regenerates a speech-backed item with valid updated intent
- **THEN** the replacement asset retains new provenance and the project publishes one atomic revision

### Requirement: Bounded queue and owned cleanup
The provider adapter MUST process synthesis with concurrency one and FIFO fairness, bound queued work, support timeout and cancellation, and attempt to clean only process-owned temporary outputs on success, failure, expiry, discard, or shutdown.

#### Scenario: Reject queue overload
- **WHEN** the configured speech queue is full
- **THEN** new synthesis fails with retryable `TTS_QUEUE_FULL` without disturbing active work

#### Scenario: Cleanup failure after commit
- **WHEN** project commit succeeds but temporary output cleanup fails
- **THEN** the committed result remains authoritative and includes a cleanup warning for later shutdown retry
