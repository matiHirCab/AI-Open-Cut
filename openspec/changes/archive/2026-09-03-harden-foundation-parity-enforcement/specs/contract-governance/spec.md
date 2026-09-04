## MODIFIED Requirements

### Requirement: Independently visible contract-parity gate
Continuous integration MUST publish a dedicated contract-parity status that executes the repository's complete standalone cross-language contract command from its declared workspace with fail-closed setup and command steps, and fails when any canonical fixture, Rust/Serde declaration, TypeScript/Zod validator, MCP definition or annotation, capability identifier, version rule, or stable error diverges from its governed consumer. The authoritative command MUST NOT be neutralized through ignored failures or additional shell control flow.

#### Scenario: Accept synchronized contracts
- **WHEN** canonical contract artifacts and every governed consumer remain synchronized
- **THEN** the dedicated contract-parity status succeeds using the same standalone command documented for local reproduction

#### Scenario: Reject fixture or consumer drift
- **WHEN** a canonical fixture or any governed Rust, TypeScript/Zod, or MCP consumer changes without the required synchronized evidence
- **THEN** the dedicated contract-parity status fails independently of general formatting, linting, unit, integration, or packaging results

#### Scenario: Reject a neutralized contract command
- **WHEN** the authoritative contract step ignores its exit status, changes its command body, or runs outside its declared workspace
- **THEN** repository policy validation fails before the weakened gate can be accepted

#### Scenario: Preserve current contract compatibility
- **WHEN** the dedicated gate is hardened
- **THEN** existing protocol versions, requests, responses, capabilities, stable errors, persisted schemas, and fixture contents remain unchanged
