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

### Requirement: Independent speech timestamp support
Speech status SHALL expose `timestampSupport` containing independent boolean `sentence`, `word`, and `phoneme` fields describing timestamp support actually exposed by the provider integration. The bridge MUST preserve every valid combination without inferring one granularity from another, readiness, or text segmentation. The current Kokoro integration MUST report all three false, including its unavailable fallback.

#### Scenario: Independent provider declarations
- **WHEN** a provider declares any of the eight boolean combinations
- **THEN** bridge status preserves that combination without changing synthesis or project state

#### Scenario: Kokoro support remains truthful
- **WHEN** Kokoro is cold, cached and ready, loaded, or cannot start
- **THEN** status reports sentence, word, and phoneme timestamp support as false

### Requirement: Compatible strict timestamp metadata
The provider-v1 status contract SHALL accept omission of the whole `timestampSupport` object for legacy providers and normalize omission to all false. Explicit metadata MUST contain exactly the three boolean fields; invalid, partial, null, or unknown-key metadata MUST fail with non-retryable `TTS_INVALID_CAPABILITIES` through bridge discovery. Existing worker version, requests, synthesis outputs, and persisted provenance MUST retain their meaning.

#### Scenario: Legacy provider status
- **WHEN** an otherwise valid legacy provider omits `timestampSupport`
- **THEN** the adapter and bridge accept its status and expose all three fields as false

#### Scenario: Reject malformed capability metadata
- **WHEN** explicit timestamp metadata has a missing key, non-boolean value, null, wrong container type, or unknown key
- **THEN** bridge discovery returns non-retryable `TTS_INVALID_CAPABILITIES` without synthesis or project mutation

### Requirement: Governed timestamp capability parity
The canonical speech-provider fixture and MCP status output catalog SHALL describe the additive metadata, legacy fallback, and strict negative cases. Native provider, bridge, and MCP consumers MUST agree with these declarations without changing unrelated public or persisted contracts.

#### Scenario: Status contract parity
- **WHEN** cross-language contract checks inspect speech status and registered MCP status output
- **THEN** they agree with the canonical timestamp metadata and retain unchanged synthesis and provenance fixtures
### Requirement: Preserve returned alignment through retained speech workflows
The bridge SHALL carry optional provider-neutral returned alignment into existing generated-asset commit and replacement provenance without inference, changing timestamp-support declarations or changing worker-v1 synthesis. Preview retention, conflicted commit retry and regeneration MUST retain exact alignment quality and producer identity in an owned nested snapshot taken inside existing provider-result cleanup ownership. Later provider mutation MUST NOT alter retained metadata. Malformed provider alignment shape SHALL use existing TTS_INVALID_OUTPUT and existing cleanup; semantic validation remains core-owned. Core SHALL remain the owner of timing/duration semantics. No new timeline batch operation SHALL be added for this issue; generated speech commit/replacement remain the existing dedicated operations.

#### Scenario: Retry aligned preview without synthesis
- **WHEN** an aligned preview commit conflicts and is retried within retention at the current revision
- **THEN** the original audio and alignment are committed without rerunning synthesis or rewriting quality

#### Scenario: Regenerate aligned speech
- **WHEN** a speech item is regenerated with a valid aligned result
- **THEN** one atomic revision preserves item identity, replaces the asset, and retains the new alignment producer and quality

#### Scenario: Isolate retained alignment from provider mutation
- **WHEN** a provider later mutates the object originally returned for a retained preview
- **THEN** commit/retry preserves the observed original alignment arrays and producer/quality snapshot without another synthesis

#### Scenario: Reject malformed provider shape with owned cleanup
- **WHEN** a provider returns null/unknown/malformed alignment fields
- **THEN** existing TTS_INVALID_OUTPUT and owned cleanup apply without publishing provenance or changing the worker-v1 protocol
