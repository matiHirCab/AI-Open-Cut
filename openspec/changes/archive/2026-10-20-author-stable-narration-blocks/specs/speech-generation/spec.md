## ADDED Requirements

### Requirement: Complete retained narration preview consumption
The speech application SHALL support explicitly approved bounded complete preview groups through its existing retained ownership mechanism. Group publication MUST preserve every preview's exact effective request/profile/alignment and original retention through conflict/validation failure, lease tokens exclusively while awaiting one core transaction, and consume them only after authoritative acknowledgement. Ordinary single-preview commit, discard, regeneration, insertion, queue, cleanup and worker contracts MUST retain their existing behavior outside active group leases.

#### Scenario: Preserve ordinary speech clients
- **WHEN** a legacy client previews, inserts, discards or regenerates speech without a block group
- **THEN** existing payloads, errors, revisions, retained conflict retry and cleanup behavior remain unchanged

#### Scenario: Commit one complete group without inference
- **WHEN** a caller approves all valid retained block previews
- **THEN** exactly one native transaction consumes the original generated artifacts without resynthesis and existing postcommit warnings remain truthful
