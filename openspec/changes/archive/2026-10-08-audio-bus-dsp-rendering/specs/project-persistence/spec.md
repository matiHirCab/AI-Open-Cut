## ADDED Requirements

### Requirement: Atomic schema42 normalized bus DSP adoption
Core SHALL adopt schema42 from supported1..41 current and retained undo/redo generations together under the existing project lock/recoverable transaction. Genuine old buses MUST retain omission of DSP;41 audio-event provenance/captured roots,40 sound libraries,39 routes and all existing assets/fonts/markers/IDs/revisions/timestamps/roles/history resources MUST remain exact. Pre42 actual bus dsp presence including null SHALL fail before defaults; dynamically named values outside bus envelopes remain valid. Malformed/null42 DSP and unknown future schemas MUST fail before authoritative rewrite. Source-matched drafts and every original fault phase MUST retain precommit prior bytes or recover the exact complete postcommit target; valid42 reopen MUST be byte-stable.

#### Scenario: Adopt all sources with mixed retained generations
- **WHEN** genuine1..41 current/undo/redo snapshots and41 event-bearing drafts are opened or validly edited
- **THEN** all generations reach42 atomically while prior content/roots/routing and neutral render output remain unchanged

#### Scenario: Reject premature malformed future and failed adoption
- **WHEN** any generation has premature DSP, malformed42 fields, future version or rejected mutation/publication fault
- **THEN** established errors and commit-point recovery preserve one authoritative complete generation without speculative resources/defaults
