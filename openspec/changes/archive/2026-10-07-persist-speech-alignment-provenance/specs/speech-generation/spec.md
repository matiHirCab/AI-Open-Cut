## ADDED Requirements

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
