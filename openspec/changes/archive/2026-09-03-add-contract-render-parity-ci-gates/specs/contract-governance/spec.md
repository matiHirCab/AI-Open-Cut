## ADDED Requirements

### Requirement: Independently visible contract-parity gate
Continuous integration MUST publish a dedicated contract-parity status that runs the repository's complete standalone cross-language contract command and fails when any canonical fixture, Rust/Serde declaration, TypeScript/Zod validator, MCP definition or annotation, capability identifier, version rule, or stable error diverges from its governed consumer.

#### Scenario: Accept synchronized contracts
- **WHEN** canonical contract artifacts and every governed consumer remain synchronized
- **THEN** the dedicated contract-parity status succeeds using the same standalone command documented for local reproduction

#### Scenario: Reject fixture or consumer drift
- **WHEN** a canonical fixture or any governed Rust, TypeScript/Zod, or MCP consumer changes without the required synchronized evidence
- **THEN** the dedicated contract-parity status fails independently of general formatting, linting, unit, integration, or packaging results

#### Scenario: Preserve current contract compatibility
- **WHEN** the dedicated gate is introduced
- **THEN** existing protocol versions, requests, responses, capabilities, stable errors, persisted schemas, and fixture contents remain unchanged
