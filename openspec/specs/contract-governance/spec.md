# Contract Governance Specification

## Purpose

Define canonical ownership, compatibility, synchronized evidence, and review gates for public contracts spanning OpenCut's implementation languages and transports.

## Requirements

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
Every governed cross-language contract change MUST update mandatory canonical fixtures and pass automated parity checks for each affected Rust, TypeScript/Zod, and MCP consumer; those checks MUST verify that derived Rust operation names are accepted by the actual Serde request deserializer, enforce TypeScript types, and compare complete client-visible MCP tool schemas and annotations while excluding description-only schema copy.

#### Scenario: Detect implementation drift
- **WHEN** a Rust wire type, TypeScript validator, MCP declaration, capability or resource identifier, version rule, or stable error diverges from its canonical contract artifact
- **THEN** the standalone contract parity gate fails with the mismatched category and consumer

#### Scenario: Detect a Rust request variant mismatch
- **WHEN** the Rust headless request enum gains, removes, or renames a serialized operation without the canonical operation catalog changing identically
- **THEN** the Rust parity test fails using variant names derived from that enum

#### Scenario: Detect Serde and derived-name drift
- **WHEN** a Rust request variant's Serde wire tag differs from the corresponding derived canonical operation name
- **THEN** the Rust parity test fails even if the checked-in operation catalog still matches the derived name

#### Scenario: Detect an MCP tool definition mismatch
- **WHEN** a registered MCP tool's client-visible structural input schema, structural output schema, or annotations differ from the canonical MCP surface catalog
- **THEN** the TypeScript parity test fails for that named tool

#### Scenario: Ignore MCP schema documentation copy
- **WHEN** only a `description` keyword changes anywhere in a registered tool's input or output JSON Schema
- **THEN** the normalized MCP compatibility definition remains unchanged

#### Scenario: Enforce TypeScript parity in the standalone gate
- **WHEN** a TypeScript-only request union or type constraint diverges from the canonical contract
- **THEN** `bun run contracts:check` fails without relying on a later general CI typecheck step

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

### Requirement: Canonical rendering-semantics capability synchronization
The additive `evaluated_scene_rendering` capability identifier MUST have one canonical checked-in owner and MUST remain synchronized with Rust headless status production, TypeScript status typing and Zod validation, MCP status exposure, canonical headless and MCP catalogs, and standalone parity evidence.

#### Scenario: Add canonical capability support
- **WHEN** canonical evaluated-scene rendering becomes available to clients
- **THEN** the current protocol major version, canonical fixtures, Rust producer, TypeScript/Zod consumer, MCP surface, and parity tests all accept and report the identical `evaluated_scene_rendering` identifier

#### Scenario: Detect capability drift
- **WHEN** any governed producer, validator, MCP schema, or canonical catalog omits, renames, or reports the capability inconsistently
- **THEN** the standalone contract parity gate fails with the mismatched capability surface

#### Scenario: Preserve existing render contracts
- **WHEN** the new capability is added
- **THEN** all existing frame-preview, range-preview, draft-preview, export request and response shapes remain valid and no project schema, provider contract, stable error, or protocol major version changes

### Requirement: Independently visible contract-parity gate
Continuous integration MUST publish a dedicated contract-parity status that executes the repository's complete standalone cross-language contract command from its declared workspace with fail-closed setup and command steps, and fails when any canonical fixture, Rust/Serde declaration, TypeScript/Zod validator, MCP definition or annotation, capability identifier, version rule, or stable error diverges from its governed consumer. The gate MUST contain only its exact reviewed checkout, toolchain, installation, and parity steps in that order. Workflow-level and contract-job-level environment maps MUST be absent so no inherited process control can alter the reviewed execution model. The authoritative command MUST NOT be neutralized through ignored failures, additional shell control flow, inherited execution defaults, environment inheritance, custom step shells, job containers, or preceding repository-mutating steps.

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

### Requirement: Schema-v7 project evidence remains cross-language exact
The canonical project/protocol fixture, Rust declarations, TypeScript declarations, strict Zod schemas, headless serialization, MCP project responses, and parity tests governed for persisted project shape MUST agree on schema version 7 and the flattened common `transform` and `hidden` fields for every timeline-item variant. Existing protocol-version-1 edit request fields, response meanings, operation identifiers, capability identifiers, annotations, and stable errors MUST remain unchanged.

#### Scenario: Verify governed consumers
- **WHEN** the contract parity gate compares a schema-v7 project containing every timeline-item variant
- **THEN** every governed Rust and TypeScript/Zod consumer accepts and emits the same exact additive shape and the canonical catalog matches their structural schemas

#### Scenario: Preserve existing edit clients
- **WHEN** an existing client sends any valid schema-v6-era add, update-transform, visibility, standalone, draft, or batch request
- **THEN** the protocol accepts the unchanged request shape and returns the same operation-level meaning while project state reports schema version 7

#### Scenario: Reject ungoverned activation
- **WHEN** parity evidence detects a new operation, capability, error, motion-graphics runtime concept, or changed retryability not authorized by this change
- **THEN** validation fails rather than accepting the unreviewed contract expansion

### Requirement: Lossless canonical MCP catalog representation
The canonical MCP catalog MUST remain a single manually governed, version-1 checked-in artifact. Its stored representation MAY share exact repeated schema subtrees, but the contract parity gate MUST first expand the representation deterministically and compare every registered tool's complete structural input schema, complete structural output schema, and client-visible annotations, as well as the exact tool, prompt, resource, and capability identifier sets. Schema `description` copy alone SHALL remain excluded from structural comparison. A storage-only rewrite MUST preserve the expanded pre-change catalog exactly and MUST NOT change the public MCP surface or protocol version.

#### Scenario: Expand the compact catalog without changing the public contract
- **WHEN** the compact catalog is loaded and expanded twice
- **THEN** both expansions are identical to the approved pre-change canonical catalog, including every schema, annotation, identifier, resource template, and version

#### Scenario: Detect live MCP structural drift
- **WHEN** a registered tool's structural input or output schema or annotation differs from its expanded canonical definition, including a difference in a shared schema subtree
- **THEN** the standalone contract parity gate fails and identifies the affected tool

#### Scenario: Detect supporting-surface drift
- **WHEN** a registered tool, prompt, resource name or template, or capability identifier differs from the expanded canonical catalog
- **THEN** the standalone contract parity gate fails for the mismatched surface

#### Scenario: Reject an invalid compact representation
- **WHEN** a reference is missing, cyclic, malformed, or resolves outside declared local definitions
- **THEN** catalog expansion fails before any parity comparison and cannot silently omit or weaken a tool definition

#### Scenario: Preserve existing client compatibility
- **WHEN** a client discovers or invokes the version-1 MCP surface after the catalog storage change
- **THEN** the same tools, schemas, annotations, prompts, resources, capabilities, typed failures, and revision-conflict behavior remain available with unchanged identifiers and meanings

### Requirement: Active mask authoring contract and capability
contracts/mask-models-v1.json MUST own active mask identifiers, exact required fields, finite inclusive limits, valid/invalid fixtures and metadata-only status, referencing existing canonical VectorPath/Paint/Transform2D semantics. Its governed Rust/Serde, TypeScript/Zod, headless and MCP consumers MUST agree. The additive mask_models_v1 capability MUST appear in editor status independently of renderer readiness; protocol major 1 and existing operation names/tool counts/annotations MUST remain. Project schema reporting MUST identify 32. Existing active animation-channels, animation-presets, extended-visual-animation, inherited-animation-timing, motion-blur-sampling and initial-motion-preset-pack catalogs MUST report current projectSchemaVersion32, synchronized with the current core and their governed consumers. Only this top-level version marker may change in those six catalogs; every feature, bound, fixture and historical source-version behavior MUST remain unchanged. A narrowly scoped 32→31 marker projection MUST reproduce independently pinned predecessor semantic digests and reject unrelated drift. Strict older consumers pinned to 31 MAY reject schema 32 responses under the existing persisted-version compatibility boundary; unchanged request acceptance SHALL NOT be misrepresented as compatibility with those old response validators. Fixture-only motion-graphics-v1 records SHALL NOT be reinterpreted as active runtime sources. Documentation MUST explicitly state typed alpha/luma masks are accepted/persisted/editable but visually inactive until #51.

#### Scenario: Detect supported mask models through unavailable rendering
- **WHEN** status reports an editor with model support but renderer dependencies unavailable
- **THEN** mask_models_v1 remains discoverable while no mask-rendering support is claimed and existing renderer readiness reporting stays accurate

#### Scenario: Reject cross-language catalog drift
- **WHEN** any fixture, field representation, nested bound, null/default rule, capability or project-version producer differs from its governed consumer
- **THEN** mandatory contract parity fails with the mismatched category before the change is considered complete

#### Scenario: Synchronize current catalog schema markers without feature drift
- **WHEN** schema32 model activation advances current project-version reporting
- **THEN** all six active animation catalogs and current consumers agree exactly on32, their narrowly projected predecessor digests remain exact, and unrelated changes or weakened historical migration guards fail conformance

### Requirement: Reviewed additive mask MCP digest transition
The expanded canonical MCP surface digest MUST advance only after reviewed mask schema/capability synchronization. The exact immediately preceding verified #49 expanded digest MUST remain pinned as predecessor evidence. Removing only the new capability and the masks properties from explicitly approved input/output schema locations, and restoring only enumerated projectSchemaVersion/schemaVersion response/status literal locations from 32 to 31, MUST reproduce the predecessor exactly, including existing schemas and annotations. Projection SHALL NOT hide unrelated matching keys or changes. Canonical storage references MUST retain missing/cyclic/malformed/nonlocal/unused rejection; parity tests SHALL NOT regenerate catalogs or expected digests.

#### Scenario: Prove the additive contract boundary
- **WHEN** reviewed expanded mask catalogs are projected back through the exact authorized fields
- **THEN** the predecessor digest is identical and unrelated schema/capability/annotation mutations fail parity
