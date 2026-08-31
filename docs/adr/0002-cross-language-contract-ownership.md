# ADR 0002: Fixture-governed cross-language contract ownership

## Status

Accepted

## Context

OpenCut repeats public wire behavior across Rust Serde types, TypeScript types and Zod validators, MCP declarations, Python providers, tests, and JSON catalogs. A change can compile in one layer while drifting from another. The contract surface includes headless requests and events, stable errors, capabilities, MCP tools and resources, provider protocols, persisted projects, and version negotiation.

A single generated schema was considered, but the current systems do not share a lossless type vocabulary. Serde uses internally tagged enums and strict unknown-field handling; the editor supports recursive and aliased batch references and fixed-width integers; Zod adds refinements, transforms, defaults, and optional-versus-nullable distinctions; MCP adds tool annotations and resource templates. Migrating the existing surface would require a custom intermediate representation and generators before providing parity evidence.

## Decision

OpenCut uses fixture-governed manual synchronization. Native Rust, TypeScript/Zod, MCP, and Python declarations remain hand-authored. Versioned checked-in artifacts under `contracts/` own public wire examples and identifier sets, and every governed consumer reads those artifacts in parity tests.

`contracts/contract-ownership-v1.json` is the authoritative ownership index:

| Category | Canonical owner |
| --- | --- |
| Headless requests, responses/events, capabilities, and version negotiation | `contracts/headless-protocol-v1.json` |
| Stable errors and retryability | `contracts/error-codes-v1.json` |
| MCP tools, resources, prompts, and exposed schemas | `contracts/mcp-surface-v1.json` |
| Speech provider protocol | `contracts/speech-provider-v1.json` |
| Transcription provider protocol | `contracts/transcription-provider-v1.json` |
| Persisted project document and migrations | `crates/editor-core/src/model.rs` |

The parity gate is `bun run contracts:check` from `apps/agent-bridge`. It runs focused Rust and TypeScript tests against the same canonical artifacts. CI runs this gate explicitly, and `.github/CODEOWNERS` requests `@matiHirCab` for canonical artifacts and governed consumers.

Protocol major version 1 is negotiated by status. Omitting `protocolVersion` selects the current version for compatibility; explicitly requesting an unsupported version fails before mutation with non-retryable `INVALID_ARGUMENT`.

Additive changes include optional request fields, ignorable response fields, new operations, capabilities, and resources whose identifiers do not change existing meaning. Removing or renaming fields or operations, narrowing values, changing semantics or retryability, or reusing identifiers is breaking and requires a new major contract plus a migration path.

## Rejected alternatives

### Generate every language from JSON Schema

Rejected because JSON Schema does not preserve all current Serde, Zod, MCP, and provider semantics without custom extensions and generators. The migration would create a large generated diff and risk weaker runtime validation.

### Generate TypeScript interfaces only

Rejected because static interfaces do not prove Zod runtime validation, Serde behavior, MCP exposure, resource registration, or error retryability. This would offer incomplete drift evidence.

### Keep code-only synchronization

Rejected because isolated native tests do not identify one canonical owner and cannot demonstrate cross-language parity in CI.

## Consequences

Contract changes require synchronized native declarations, fixtures/catalogs, parity tests, and owner review in one pull request. This retains some manual maintenance but makes drift observable and keeps each language's full validation expressiveness. Generation can be reconsidered if measured synchronization cost justifies a lossless intermediate representation.
