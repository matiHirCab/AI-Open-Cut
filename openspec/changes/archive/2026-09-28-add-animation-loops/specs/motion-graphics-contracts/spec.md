## MODIFIED Requirements

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
- **THEN** it can distinguish supported typed-channel edits and loops from later unimplemented inactive-property and rendering features

### Requirement: Governed active parameterized curve contract
Canonical checked-in channel and motion-graphics catalogs MUST describe the active `cubic_bezier` and `spring` tagged records, named finite parameter limits, existing string `hold`/`linear` compatibility, and an additive capability/version marker. The governed Rust, TypeScript/Zod, headless, and MCP consumers MUST accept and reject the same canonical valid and invalid fixtures. The existing `set_animation_channels` and batch operations MUST remain typed and additive; unknown variants and fields MUST fail without coercion. Stable error codes and retryability MUST remain unchanged.

#### Scenario: Compare cross-language curve acceptance
- **WHEN** each canonical boundary, malformed, non-finite, unknown-field, and unknown-variant fixture is checked in Rust and TypeScript
- **THEN** both languages agree on its wire shape, named bounds, activation, and stable result

#### Scenario: Discover runtime support
- **WHEN** a client reads capability and version reporting
- **THEN** it can distinguish active Bézier/spring channels and loops from deferred inactive-property behavior

#### Scenario: Retain old clients
- **WHEN** a client sends a previously valid channel or legacy keyframe request
- **THEN** its wire format, result shape, and error behavior remain compatible

## ADDED Requirements

### Requirement: Governed additive loop contract
The checked-in versioned channel, persisted-project, headless, MCP, capability, and motion-graphics catalogs MUST describe the exact optional loop record, two modes, finite count bound, infinite literal, endpoint rule, schema 25, and support capability. Rust, TypeScript/Zod, headless, and MCP consumers MUST agree with canonical accepted, boundary, malformed, and legacy fixtures. Existing request/response fields, operation identifiers, protocol major 1, stable error codes, and retryability MUST remain compatible. Inactive channels MUST remain unwritable even when a loop is supplied.

#### Scenario: Compare cross-language loop fixtures
- **WHEN** each canonical valid, boundary, unknown-field, unknown-variant, invalid-count, endpoint, and legacy fixture is checked by governed consumers
- **THEN** every consumer agrees on accepted shape, bounds, capability, and stable result

#### Scenario: Discover support without changing old clients
- **WHEN** a client reads capability/version reporting or sends an existing unlooped channel request
- **THEN** it can distinguish active loop support and its old request retains the same result and representation
