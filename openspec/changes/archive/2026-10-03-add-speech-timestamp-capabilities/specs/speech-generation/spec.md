## ADDED Requirements

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
