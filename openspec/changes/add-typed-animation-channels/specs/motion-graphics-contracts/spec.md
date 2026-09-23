## ADDED Requirements

### Requirement: Governed runtime channel contract
The canonical contract artifacts MUST define the versioned channel names, value tags, target kinds, activation state, limits, and additive operation/capability surface. Rust and TypeScript parity tests MUST consume the checked-in fixtures and agree on accepted and rejected envelopes. Existing public identifiers and error retryability MUST remain stable.

#### Scenario: Compare language consumers
- **WHEN** canonical valid, invalid, and inactive-channel examples are checked by Rust and TypeScript
- **THEN** both consumers agree on tags, limits, compatibility, and expected stable errors

#### Scenario: Discover capability
- **WHEN** a client reads capability/version reporting
- **THEN** it can distinguish supported typed-channel edits from later unimplemented curve, marker, loop, and rendering features
