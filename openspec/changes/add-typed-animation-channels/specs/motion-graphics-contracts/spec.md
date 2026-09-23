## ADDED Requirements

### Requirement: Governed runtime channel contract
The canonical contract artifacts MUST define the versioned channel names, value tags, prospective target kinds, activation state, limits, and additive operation/capability surface. Active entries MUST declare finite numeric bounds; inactive entries MUST explicitly mark numeric bounds as deferred until activation and MUST remain unwritable. Rust and TypeScript parity tests MUST consume the checked-in fixtures, compare every entry's metadata, and agree on accepted and rejected envelopes. Existing public identifiers and error retryability MUST remain stable.

#### Scenario: Compare language consumers
- **WHEN** canonical valid, invalid, and inactive-channel examples are checked by Rust and TypeScript
- **THEN** both consumers agree on tags, target kinds, activation and bound states, active numeric limits, compatibility, and expected stable errors

#### Scenario: Discover an inactive channel without premature bounds
- **WHEN** a client reads a cataloged channel whose evaluator is inactive
- **THEN** its value tag and prospective target kind are declared, its numeric bounds are explicitly deferred, and editing it returns `INVALID_ARGUMENT` without persistence

#### Scenario: Discover capability
- **WHEN** a client reads capability/version reporting
- **THEN** it can distinguish supported typed-channel edits from later unimplemented curve, marker, loop, and rendering features
