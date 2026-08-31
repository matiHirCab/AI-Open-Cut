## Context

OpenCut currently maintains public wire behavior in Serde types, TypeScript types and Zod schemas, MCP registrations, tests, and JSON catalogs. Some categories already have a natural canonical artifact (`contracts/error-codes-v1.json`, provider protocol files, and the persisted Rust project model), while headless operations, responses, capability names, MCP tools/resources, and version behavior are implicit in code. Existing tests prove isolated fragments but no single gate demonstrates parity across all consumers.

Issue #84 asks for an explicit choice between schema-driven generation and fixture-governed manual synchronization. The decision must cover types that do not translate losslessly among Rust, TypeScript/Zod, and MCP JSON Schema, and must be usable by the motion-graphics work in issues #11 and #16.

## Goals / Non-Goals

**Goals:**

- Assign one canonical owner per public contract category.
- Make additive, breaking, error, capability, resource, and version rules unambiguous.
- Fail CI on Rust/TypeScript/MCP drift with actionable parity tests.
- Prove the workflow with an additive status protocol-version negotiation field.
- Require a designated contract review for both canonical artifacts and synchronized consumers.

**Non-Goals:**

- Generate editor-core domain or persisted project models from JSON Schema.
- Replace Serde, Zod, or the MCP SDK's native schema declarations.
- Change the process-per-request headless architecture or MCP transport.
- Introduce a breaking protocol version or migrate persisted projects.

## Decisions

### Retain fixture-governed manual synchronization

OpenCut will retain hand-authored native Rust, TypeScript/Zod, and MCP declarations. Checked-in JSON contract artifacts and examples under `contracts/` are the canonical wire evidence, and a parity suite consumes those same artifacts in every language.

Full schema/code generation is rejected for this version. Serde's internally tagged enums and strict unknown-field behavior, recursive/aliased batch-edit references, Rust integer widths, Zod refinements and transforms, optional-versus-nullable fields, and MCP SDK annotations do not share one lossless schema vocabulary. Adopting generation now would either erase validation semantics or require a custom intermediate representation and generator toolchain. Migrating all existing contracts at once would also create a large generated diff before parity is established. Generation can be reconsidered when the contract surface or repeated maintenance cost justifies owning that compiler.

Generating only interfaces from JSON Schema is also rejected because it would create false confidence: generated static types would not prove runtime validation, Serde behavior, MCP exposure, stable errors, or resources.

### Canonical ownership matrix

The ADR will record this ownership matrix:

| Contract category | Canonical owner | Governed consumers |
| --- | --- | --- |
| Headless requests, responses/events, capability identifiers, and protocol negotiation | Versioned headless public-contract fixture/catalog in `contracts/` | Rust Serde boundary, TypeScript/Zod bridge schemas, MCP status mapping, protocol tests |
| Stable errors and retryability | `contracts/error-codes-v1.json` | editor-core error enum/tests, headless envelopes, bridge error mapping |
| MCP tool names, input/output exposure, resources, and prompts | Versioned MCP surface catalog in `contracts/` | capability registrars, SDK schemas, architecture/integration tests |
| Speech and transcription provider protocols | Existing versioned provider contracts in `contracts/` | Python workers, TypeScript providers, Rust provenance fields where applicable |
| Persisted project document | Versioned editor-core model and migration rules | editor-core store/migration tests, headless/bridge project responses |

The ownership manifest will also enumerate required consumers so adding a new category cannot silently omit parity evidence. `AGENTS.md`, the contributor OpenSpec guide, and an ADR will point contributors to the matrix rather than duplicating its details.

### Compatibility and version negotiation

The public headless protocol starts at integer major version `1`. Status accepts an optional requested protocol version for backward compatibility; omission selects the current version. A supported request returns `protocolVersion: 1`. An unsupported explicit version fails before dispatch with non-retryable `INVALID_ARGUMENT`.

Adding optional request fields, ignorable response fields, new operations, new capability identifiers, and new resources is additive when older clients remain valid. Removing or renaming fields/operations, narrowing accepted values, changing field or error meaning, changing retryability, or reusing identifiers is breaking and requires a new major fixture/catalog plus a documented migration. Stable error codes remain governed by their catalog and are never inferred from free-form messages.

The additive `protocolVersion` path is the representative workflow: canonical request/response examples drive Rust protocol tests, TypeScript/Zod parsing tests, MCP registrar tests, and an integration path through `editor_get_status`.

### Parity gate and required review

A repository-level contract parity command will run all focused Rust and TypeScript parity suites. CI will invoke it explicitly in the correctness job. Tests will identify the category and consumer when they fail, and will cover requests, responses/events, errors, capabilities, MCP tools/resources, provider versions, and version negotiation.

`.github/CODEOWNERS` will designate `@matiHirCab` for `contracts/`, headless protocol types, bridge schemas/registrars, and contract tests. Contributor guidance will require that reviewer for canonical or consumer changes, plus synchronized fixtures and tests in the same pull request. CODEOWNERS requests review on GitHub; branch protection remains repository administration rather than code behavior.

## Risks / Trade-offs

- [Manual declarations can still drift before tests run] -> Use shared fixtures in every consumer, one parity command, explicit CI execution, and CODEOWNERS coverage.
- [Example fixtures may not prove every valid value] -> Combine representative examples with catalog/set parity for identifiers and targeted negative cases for strictness and unsupported versions.
- [The ownership manifest can become another stale document] -> Validate its known categories, canonical paths, and required consumers in the parity suite.
- [Adding a response field can affect strict legacy consumers] -> Keep request omission compatible, document response fields as additive/ignorable, and exercise existing bridge consumers.
- [Manual synchronization costs grow with the API] -> Revisit generation if parity maintenance becomes repetitive enough to justify a lossless intermediate representation.

## Migration Plan

1. Add the ADR, canonical ownership manifest/catalogs, fixtures, CODEOWNERS rules, and contributor guidance.
2. Add the parity command and failing tests for current Rust, TypeScript/Zod, MCP, catalog, and resource surfaces.
3. Add protocol-version negotiation to the fixture, Rust status boundary, TypeScript schemas, and MCP status tool until all parity tests pass.
4. Run the parity gate plus the repository's Rust, TypeScript, integration, smoke, and OpenSpec checks.
5. Roll back by removing the additive field and new gate/artifacts together; no persisted data migration or external service rollback is required.

## Open Questions

None. A future proposal may reconsider generation after the fixture-governed baseline measures its maintenance cost.
