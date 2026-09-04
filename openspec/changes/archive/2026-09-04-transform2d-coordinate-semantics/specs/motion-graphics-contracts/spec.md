## ADDED Requirements

### Requirement: Governed runtime Transform2D contract
A versioned transform2d-v1 runtime catalog MUST define the complete closed Transform2D payload, defaults, bounds, switching rules, coordinate semantics, valid and invalid fixtures, and its mapping to existing fixture vocabulary. The remaining motion-graphics-v1 catalog MUST stay fixture-only. Contract ownership, typed headless requests/responses, MCP Zod schemas, existing update/batch surfaces, persisted consumers, and parity evidence MUST agree. Ready implementations MUST advertise the additive transform2d capability without removing existing capabilities or changing protocol major version. No new provider or stable-error contract SHALL be introduced.

#### Scenario: Discover and use support
- **WHEN** a client reads capabilities from a runtime with complete Transform2D support
- **THEN** it sees transform2d and can submit a complete typed update standalone or in a batch and read the resulting value

#### Scenario: Enforce cross-language parity
- **WHEN** Rust and TypeScript validate canonical success, boundary, unknown-field, invalid-number, unsupported-unit, and conflicting-update fixtures
- **THEN** both agree on the documented payload acceptance and failure semantics

#### Scenario: Keep roadmap concepts inactive
- **WHEN** the client inspects contracts after this milestone
- **THEN** only static Transform2D is newly addressable and other motion-graphics concepts remain fixture-only

#### Scenario: Preserve old request compatibility
- **WHEN** an existing valid legacy request is sent to the new runtime
- **THEN** its shape, transform meaning, and stable error/retryability behavior remain supported
