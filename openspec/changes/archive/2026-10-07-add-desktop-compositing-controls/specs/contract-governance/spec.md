## ADDED Requirements

### Requirement: Governed desktop compositing catalog and fresh API parity
A canonical desktop-compositing-controls-v1 catalog MUST govern exact defaults, complete parameter/choice matrix and desktop consumers with Rust/TypeScript parity tests. Ownership/CODEOWNERS MUST identify the desktop consumer of mask/matte/blend/parameterized/group catalogs. Existing core/headless/MCP operations MUST expose fresh standalone and alias-batch compositing lifecycle/failure parity without adding operations/tools/schema/capabilities or changing schema37/protocol1/78 tools/current and predecessor MCP pins. Catalog tests MUST consume checked-in authorities and MUST NOT regenerate them from live declarations. The required contracts:check/contract-parity gate MUST execute hermetic Rust desktop canonical tests with the existing signed desktop native link prerequisites installed; policy MUST enforce that coverage and preserve every prior command/control/timeout/pin. The actual regular bridge package source and exact complete canonical contracts:check command MUST be required in protected preflight before Moon execution/attestation and tracked in Moon inputs; missing, malformed, substituted, failure-masked or omitted consumer/input evidence MUST reject before success attestation. Protected policy MUST consume a required regular package source before Moon execution/attestation, validate the complete canonical contracts:check command preserving every predecessor consumer, track that package in Moon inputs and reject missing/substituted/failure-masked coverage additively. Actual GPUI acceptance remains a separate required gate.

#### Scenario: C25 Match controls defaults and unchanged public surfaces
- **WHEN** Rust desktop and TypeScript/headless/core parity consume canonical catalogs
- **THEN** all defaults/fields/choices agree and every current/predecessor surface pin remains exact

#### Scenario: C26 Exercise standalone alias batch and rollback parity
- **WHEN** compositing edits use standalone APIs and alias batches with omission/clear/order, duplicate/missing/reference/lock/stale or later-operation failure
- **THEN** shared semantics, undo/redo/reopen and complete unchanged failure inventories agree across core/headless/MCP

## MODIFIED Requirements

### Requirement: Independently visible contract-parity gate
Continuous integration MUST publish a dedicated contract-parity status that executes the repository's complete standalone cross-language contract command from its declared workspace with fail-closed setup and command steps, and fails when any canonical fixture, Rust/Serde declaration, TypeScript/Zod validator, MCP definition or annotation, capability identifier, version rule, or stable error diverges from its governed consumer. The gate MUST contain only its exact reviewed checkout, signed desktop native prerequisite installation, toolchain, JavaScript installation, and parity steps in that order. Workflow-level and contract-job-level environment maps MUST be absent so no inherited process control can alter the reviewed execution model. The authoritative command MUST NOT be neutralized through ignored failures, additional shell control flow, inherited execution defaults, environment inheritance, custom step shells, job containers, or preceding repository-mutating steps.

#### Scenario: Accept synchronized contracts
- **WHEN** canonical contract artifacts and every governed consumer remain synchronized under the exact reviewed leaf sequence without inherited workflow or contract-job environment
- **THEN** the dedicated contract-parity status succeeds using the same standalone command documented for local reproduction

#### Scenario: Reject fixture or consumer drift
- **WHEN** a canonical fixture or any governed Rust, TypeScript/Zod, or MCP consumer changes without the required synchronized evidence
- **THEN** the dedicated contract-parity status fails independently of general formatting, linting, unit, integration, or packaging results

#### Scenario: Reject an injected contract preparation step
- **WHEN** a step is added, duplicated, replaced, or reordered so code can rewrite a governed fixture or consumer before contract parity executes
- **THEN** repository policy validation fails before the altered evidence can be accepted

#### Scenario: Reject a neutralized contract command
- **WHEN** the authoritative contract step ignores its exit status, changes its command body, runs outside its declared workspace, uses a custom shell, inherits workflow or job environment or execution defaults, or runs in a job container
- **THEN** repository policy validation fails before the weakened gate can be accepted

#### Scenario: Preserve current contract compatibility
- **WHEN** the dedicated gate's isolated closed sequence is enforced
- **THEN** existing protocol versions, requests, responses, capabilities, stable errors, persisted schemas, and fixture contents remain unchanged

