## ADDED Requirements

### Requirement: Bounded historical MCP verification without weaker evidence
Diagnostic contract verification MUST avoid duplicated identical DSP/audio-event predecessor computation within the affected test while retaining all count/hash assertions, independent current-catalog expansion, input immutability, negative controls and historical evidence. Its existing single-case5s deadline, profiles and required gates MUST remain unchanged.

#### Scenario: Reuse one predecessor within one case
- **WHEN** count and pinned-hash assertions inspect the identical DSP or audio-event predecessor
- **THEN** one case-local computed result serves both unchanged assertions, with no cross-case cache or altered resolver/projection behavior

#### Scenario: Preserve independent and invalid-input evidence
- **WHEN** current catalog determinism, historical predecessors or invalid/drift controls execute
- **THEN** independent current expansion, all expected counts/digests/raw pins and rejection assertions remain unchanged and inputs remain intact

#### Scenario: Supported Windows verification
- **WHEN** the changed test runs on the exact published Windows head
- **THEN** it retains the same5s guard; timeout or any missing assertion remains a failure, and passing Linux timings alone are not acceptance
