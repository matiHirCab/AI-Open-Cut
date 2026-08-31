## ADDED Requirements

### Requirement: Canonical contract ownership
The project MUST assign exactly one canonical checked-in owner to each public contract category, including headless requests, headless responses and events, stable errors, capability identifiers, MCP tools and resources, provider protocols, persisted project documents, and protocol-version negotiation.

#### Scenario: Locate a contract authority
- **WHEN** a contributor changes a public contract category
- **THEN** contributor guidance identifies one canonical artifact and the synchronized Rust, TypeScript, MCP, fixture, or documentation consumers

#### Scenario: Preserve layer ownership
- **WHEN** a contract spans editor-core, headless transport, and agent bridge
- **THEN** its canonical owner and synchronization workflow preserve the repository's inward dependency direction and do not move domain rules into a transport or presentation layer

### Requirement: Compatibility policy
Public contract changes MUST follow documented compatibility rules that distinguish additive, breaking, and version-negotiated changes, preserve stable error semantics, and reject unsupported future versions with a typed failure.

#### Scenario: Make an additive change
- **WHEN** a producer adds an optional request field or a response field that existing consumers can ignore
- **THEN** the change retains the current major protocol version and all governed consumers and fixtures are updated together

#### Scenario: Propose a breaking change
- **WHEN** a change removes, renames, narrows, or changes the meaning of a public field, operation, capability, resource, or error
- **THEN** the change requires a new major contract version and an explicit migration path before implementation

#### Scenario: Reject an unsupported version
- **WHEN** a client explicitly requests a protocol version the endpoint does not support
- **THEN** the endpoint rejects the request before mutation with stable non-retryable `INVALID_ARGUMENT`

### Requirement: Fixture-governed synchronization evidence
Every governed cross-language contract change MUST update mandatory canonical fixtures and pass automated parity checks for each affected Rust, TypeScript/Zod, and MCP consumer.

#### Scenario: Detect implementation drift
- **WHEN** a Rust wire type, TypeScript validator, MCP declaration, capability or resource identifier, version rule, or stable error diverges from its canonical contract artifact
- **THEN** the contract parity gate fails with the mismatched category and consumer

#### Scenario: Prove an additive workflow
- **WHEN** protocol-version negotiation is added to the status request and response
- **THEN** the same canonical examples are accepted and emitted by Rust, validated by TypeScript, exposed through MCP, and exercised by integration tests

### Requirement: Contract review and CI gate
Changes to canonical contracts or governed consumers MUST require review from the designated contract owner and MUST run the contract parity gate in continuous integration.

#### Scenario: Review a contract change
- **WHEN** a pull request changes a canonical contract artifact or one of its governed consumers
- **THEN** repository ownership rules request the designated contract reviewer and contributor guidance requires synchronized evidence

#### Scenario: Run repository CI
- **WHEN** continuous integration evaluates a change
- **THEN** it runs the contract parity gate before accepting Rust, TypeScript, or MCP contract changes
