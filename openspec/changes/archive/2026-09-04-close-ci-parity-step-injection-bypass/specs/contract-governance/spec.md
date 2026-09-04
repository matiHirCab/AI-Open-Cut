## MODIFIED Requirements

### Requirement: Independently visible contract-parity gate
Continuous integration MUST publish a dedicated contract-parity status that executes the repository's complete standalone cross-language contract command from its declared workspace with fail-closed setup and command steps, and fails when any canonical fixture, Rust/Serde declaration, TypeScript/Zod validator, MCP definition or annotation, capability identifier, version rule, or stable error diverges from its governed consumer. The gate MUST contain only its exact reviewed checkout, toolchain, installation, and parity steps in that order. The authoritative command MUST NOT be neutralized through ignored failures, additional shell control flow, inherited execution defaults, custom step shells, job containers, or preceding repository-mutating steps.

#### Scenario: Accept synchronized contracts
- **WHEN** canonical contract artifacts and every governed consumer remain synchronized under the exact reviewed leaf sequence
- **THEN** the dedicated contract-parity status succeeds using the same standalone command documented for local reproduction

#### Scenario: Reject fixture or consumer drift
- **WHEN** a canonical fixture or any governed Rust, TypeScript/Zod, or MCP consumer changes without the required synchronized evidence
- **THEN** the dedicated contract-parity status fails independently of general formatting, linting, unit, integration, or packaging results

#### Scenario: Reject an injected contract preparation step
- **WHEN** a step is added, duplicated, replaced, or reordered so code can rewrite a governed fixture or consumer before contract parity executes
- **THEN** repository policy validation fails before the altered evidence can be accepted

#### Scenario: Reject a neutralized contract command
- **WHEN** the authoritative contract step ignores its exit status, changes its command body, runs outside its declared workspace, uses a custom shell, inherits execution defaults, or runs in a job container
- **THEN** repository policy validation fails before the weakened gate can be accepted

#### Scenario: Preserve current contract compatibility
- **WHEN** the dedicated gate's closed sequence is enforced
- **THEN** existing protocol versions, requests, responses, capabilities, stable errors, persisted schemas, and fixture contents remain unchanged
