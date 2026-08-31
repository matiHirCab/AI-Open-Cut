## Context

PR #93 introduced canonical operation and MCP surface catalogs, but the first parity implementation compares the Rust operation catalog to a separate test-only constant, captures only MCP registration names, and runs Vitest without invoking TypeScript's compiler. All current checks pass while these structures still allow realistic future drift.

The fixes must preserve ADR 0002's fixture-governed manual synchronization strategy. They strengthen evidence without generating native Rust or TypeScript declarations and without changing runtime protocol behavior.

## Goals / Non-Goals

**Goals:**

- Couple Rust operation parity to the actual serialized `Request` variants.
- Treat each registered tool's client-visible input schema, output schema, and annotations as part of the canonical MCP surface.
- Make the documented standalone parity command enforce TypeScript type constraints.
- Keep canonical artifacts checked in and manually reviewed.

**Non-Goals:**

- Change protocol version 1 or any runtime request/response behavior.
- Generate native declarations from JSON Schema.
- Make descriptions or implementation-only handler details compatibility surfaces.
- Encode custom Zod refinements that MCP's JSON Schema exposure cannot represent; focused behavioral tests continue to own those rules.

## Decisions

### Derive Rust operation names with Strum

Add `strum` 0.27.2 with its derive feature as a workspace dependency and consume it from `opencut-headless`. Derive `VariantNames` on `Request` with snake-case serialization, remove the independent `HEADLESS_OPERATIONS` constant, sort `Request::VARIANTS` in the test, and compare it directly with the sorted canonical operation list.

This is preferred over parsing Rust source text, which is brittle, and over a large macro that would need to reproduce the existing enum declaration. Strum is already present transitively in the lockfile, but it becomes an explicit dependency because production source derives the metadata under test.

### Canonicalize complete MCP tool exposure

Keep the readable `tools` name array in `mcp-surface-v1.json` and add a `toolDefinitions` object keyed by tool name. Each entry contains:

- `inputSchema`: `z.toJSONSchema` with `io: "input"`, `target: "draft-2020-12"`, and `unrepresentable: "throw"`.
- `outputSchema`: the same conversion with `io: "output"`.
- `annotations`: the exact client-visible annotation object supplied at registration.

The contract harness captures the definition argument passed to every `registerTool` call. A normalizer converts schemas to JSON-compatible values, recursively sorts object keys, preserves array order, and omits descriptions. The test compares normalized live definitions with the checked-in catalog and also requires the name list, definition keys, and registered names to be identical.

The catalog remains manually governed: tests only read it and never rewrite it. A one-time temporary inspection may be used while authoring the initial checked-in definitions, but no artifact-update generator becomes part of the supported workflow.

### Typecheck inside the parity command

Prepend `bun run typecheck` to `contracts:check`, followed by the existing Rust and focused Vitest parity suites. This makes `satisfies Record<HeadlessRequest["operation"], true>` effective when contributors run the documented gate locally, while the separate CI typecheck remains defense in depth.

## Risks / Trade-offs

- [The MCP catalog becomes substantially larger] -> Keep tool names separate for readability and use deterministic key normalization so review diffs remain stable.
- [Zod JSON Schema output may change after dependency upgrades] -> Treat dependency-driven output changes as explicit contract reviews and update the catalog only with CODEOWNER approval.
- [Custom refinements may not appear in JSON Schema] -> Preserve existing focused validator tests and limit this catalog to the client-visible MCP schema.
- [Strum naming could diverge from Serde naming] -> Configure both derives explicitly for snake case and retain canonical request deserialization tests.

## Migration Plan

1. Add the Rust derived variant metadata and dependency, then remove the duplicate constant.
2. Populate canonical MCP definitions from the current registrations and make the harness compare normalized definitions.
3. Add TypeScript checking to the parity command and run all contract, workspace, integration, and smoke checks.
4. Roll back all three development-time changes together if necessary; no runtime data or protocol migration is involved.

## Open Questions

None.
