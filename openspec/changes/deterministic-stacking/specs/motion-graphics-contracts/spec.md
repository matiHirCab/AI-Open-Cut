## ADDED Requirements

### Requirement: Governed runtime stacking contract
A versioned stacking-v1 runtime catalog MUST define exact fields, numeric bounds, default/migration semantics, ordering comparator, operations, errors, and valid/invalid fixtures. Ownership, typed headless request/response and batch unions, MCP Zod input/output schemas and registration, persisted consumers, and parity evidence MUST agree. Ready runtimes MUST advertise additive stacking support without changing protocol major version or removing existing capabilities. Remaining roadmap concepts MUST stay fixture-only.

#### Scenario: Discover and exercise stacking
- **WHEN** a client discovers stacking support
- **THEN** all three operations work through typed standalone and batch APIs and project responses expose ordering fields

#### Scenario: Verify strict parity and compatibility
- **WHEN** canonical valid, bounds, unknown-field, wrong-type, invalid-value, alias, and legacy-operation fixtures run through Rust and TypeScript consumers
- **THEN** acceptance, response shapes, stable errors and retryability agree and previously valid simple requests remain valid

#### Scenario: Reject unsafe fields
- **WHEN** a client submits a raw expression, path, URL, executable markup, non-finite token, or unknown field as stacking input
- **THEN** strict typed validation rejects it before renderer execution or publication
