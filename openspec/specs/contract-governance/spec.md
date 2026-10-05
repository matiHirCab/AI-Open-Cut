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
contracts/mask-models-v1.json MUST own active mask identifiers, exact required fields, finite inclusive limits, valid/invalid fixtures and model-authoring status with an explicit active-rendering-contract annotation, referencing existing canonical VectorPath/Paint/Transform2D semantics. Its governed Rust/Serde, TypeScript/Zod, headless and MCP consumers MUST agree. The additive mask_models_v1 capability MUST appear in editor status independently of renderer readiness; protocol major 1 and existing operation names/tool counts/annotations MUST remain. Project schema reporting MUST identify33. Existing active animation-channels, animation-presets, extended-visual-animation, inherited-animation-timing, motion-blur-sampling and initial-motion-preset-pack catalogs, and mask-models-v1, MUST report current projectSchemaVersion33, synchronized with current core and governed consumers. The five unchanged animation feature catalogs MUST change only their top-level32→33 marker. Animation-channels-v1 MAY additionally add only the separately approved fifteen mask properties and typed-mask target/applicability/activation declarations; mask-models-v1 MAY additionally update only the explicitly approved renderer annotation and stored expansion/paint-order description. Existing model fields/limits/fixtures and every unrelated feature/bound/fixture/historical source-version behavior MUST remain unchanged. Explicitly enumerated33→32 predecessor projections MUST reproduce independently pinned verified #50 semantic digests: marker-only for the five unchanged animation catalogs, marker plus exact approved additions/annotation changes for the animation and model authorities. For the six animation catalogs, the existing #50 top-level32→31 projection and original verified #49 predecessor pins MUST remain valid through the composed projection; no nonexistent pre-#50 mask-model catalog SHALL be invented. Unrelated drift MUST fail. Strict older consumers pinned to32 MAY reject schema33 responses under the existing persisted-version compatibility boundary; unchanged request acceptance SHALL NOT be misrepresented as compatibility with those old response validators. Fixture-only motion-graphics-v1 records SHALL NOT be reinterpreted as active runtime sources. Documentation MUST distinguish accepted/persisted/editable model support, editor mask_animation_v1 support and separately ready mask_rendering_v1 execution, including intentional32→33 masked-output activation and unchanged no-mask output.


The exact mask-model catalog activation whitelist SHALL be top-level projectSchemaVersion32→33, status authoring_metadata_only→authoring_with_active_rendering_contract, rendererStage identity_until_separately_approved_mask_rendering→contracts/mask-rendering-v1.json, semantics.order path_fill_paint, signed_expansion, feather, mask_affine_opacity, inversion, ordered_combination→geometric_fill, signed_expansion, original_coordinate_paint_channel, feather, mask_affine_opacity, inversion, ordered_combination, and semantics.timing static_metadata_no_animation_targets→static_and_mask_targeted_animation. The predecessor projection MUST first verify each exact current activation value, restore only these five paths to their exact schema32 values and reproduce the pinned complete predecessor digest. No other model field or fixture MAY change; missing/malformed activation annotations or unrelated drift MUST fail parity.

#### Scenario: Detect supported mask models through unavailable rendering
- **WHEN** status reports an editor with model support but renderer dependencies unavailable
- **THEN** mask_models_v1 and mask_animation_v1 remain discoverable while mask_rendering_v1 is absent and existing renderer readiness reporting stays accurate

#### Scenario: Reject cross-language catalog drift
- **WHEN** any fixture, field representation, nested bound, null/default rule, capability or project-version producer differs from its governed consumer
- **THEN** mandatory contract parity fails with the mismatched category before the change is considered complete

#### Scenario: Synchronize current catalog schema markers without feature drift
- **WHEN** schema33 rendering/channel activation advances current project-version reporting
- **THEN** all six active animation catalogs, the mask model authority and current consumers agree exactly on33, their exact approved projections reproduce verified #50 predecessor semantics, the six-animation composed32→31 evidence remains exact, and unrelated changes or weakened historical migration guards fail conformance

### Requirement: Reviewed additive mask MCP digest transition
The expanded canonical MCP surface digest MUST advance only after reviewed schema 33 mask property/target/capability synchronization. The exact immediately preceding verified #50 expanded digest `88b55ff7be147cb4aadc016dd92f92342c3dc366f49ac139b8116513d5854830` from commit `62c30eb224d36e0b0a73c88362e4423555a084f7` MUST remain pinned. Projection MUST remove only the two new capabilities and exact new mask property/target enum members at explicitly authorized schema locations, restoring only named projectSchemaVersion/schemaVersion literals33→32. It MUST reproduce verified #50 expanded structure/digest exactly. Prior #50→#49 approved mask metadata projection/digest evidence MUST remain valid through that intermediate projection rather than be removed. No broad matching-key/member deletion SHALL hide unrelated changes. Missing/cyclic/malformed/nonlocal/unused catalog reference rejection and deliberate manual expected catalogs/digests MUST remain.

#### Scenario: Prove both additive mask boundaries
- **WHEN** reviewed schema 33 expanded catalogs project back through exact authorized mask animation/readiness fields, then through the prior model-only projection
- **THEN** both verified predecessor digests match exactly and unrelated schema/capability/annotation changes fail parity

### Requirement: Canonical active mask renderer and animation contract
contracts/mask-rendering-v1.json MUST own algorithm, mask-subset property/target applicability, new work/reservation/precision limits, independent fixtures and linked mask_animation_v1/mask_rendering_v1 reporting, referencing existing model/vector/Transform2D/curve/clock contracts. animation-channels-v1 MUST remain the canonical global property/target enumeration/activation and value/curve owner, and existing headless/MCP catalogs MUST remain global capability/surface owners; no competing enumeration authority SHALL be introduced. Every governed native/Serde, TypeScript/Zod, headless and MCP consumer MUST agree. Editor animation support MUST be independent of renderer dependencies; rendering capability MUST require complete conforming backend readiness without degraded fallback. Protocol major 1, operation/tool counts/annotations and existing stable errors MUST remain. Raw duplicate-field assertions MUST apply only at boundaries preserving raw keys; parsed objects MUST retain strict typed/unknown-field validation without false original-wire duplicate claims.

#### Scenario: Detect algorithm channel and readiness drift
- **WHEN** any canonical fixture, typed property/target/value, bound, sampling rule or authoring/rendering capability diverges from its consumer
- **THEN** mandatory contract parity fails for the affected category and no readiness or completion claim is permitted

### Requirement: Bounded exact MCP predecessor conformance proof
The canonical MCP conformance suite MUST prove complete deterministic current expansion and the exact schema33→32→31→pre-linear predecessor digests within the existing single-test5000ms deadline on supported CI platforms. The proof MAY reuse its already independently expanded read-only module fixture and one already-checked compact predecessor source, but MUST independently expand the current source again, compare complete serialized JSON bytes, retain every nested field/scalar/array/key order and assert the unchanged78-tool count, all four pinned SHA256 values and exactly-one linear capability removal. The current serialized bytes MAY be reused for their digest assertion. Strict projection helpers, source/result isolation, all malformed/missing/cyclic/nonlocal/sibling/unused-reference and unrelated/schema/description/annotation drift controls, actual registered schema parity and public catalogs MUST remain unchanged. No timeout extension, test splitting, suppression, helper aliasing change or CI-policy weakening SHALL substitute for this proof.

#### Scenario: Preserve the complete current and predecessor proof
- **WHEN** the canonical schema33 MCP catalog is checked using two independent expansions and the checked33→32→31 chain
- **THEN** complete current bytes agree and all four existing pinned digests,78tools and linear-capability multiplicity pass within the unchanged5000ms deadline, with source/projection inputs unmodified

#### Scenario: Preserve malformed and unrelated drift rejection
- **WHEN** malformed references, missing/malformed/duplicated mask additions, misplaced matching keys, unrelated fields/descriptions/annotations or unexpected fixture mutation is introduced
- **THEN** unchanged expansion/projection/drift/current-registration controls and exact current-byte or predecessor-digest assertions reject it without narrowing compared content or weakening the deadline
