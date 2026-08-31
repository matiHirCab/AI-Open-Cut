## Why

PR #93 establishes contract governance, but its parity gate can still pass when the Rust request enum or MCP input/output schemas drift from their declared canonical catalogs, and its standalone command skips TypeScript's compile-time parity constraint. These review gaps must be closed before the governance workflow can provide the evidence required by issue #84.

## What Changes

- Couple the canonical headless operation list directly to the Rust `Request` enum's derived serialized variant names.
- Make the MCP surface catalog own normalized client-visible input schemas, output schemas, and tool annotations for every registered tool, then compare live registrations against that catalog.
- Include TypeScript typechecking in the standalone contract parity command so type-only synchronization constraints cannot be skipped.
- Add regression coverage for Rust discriminant drift, MCP schema/annotation drift, and TypeScript-only request-union drift.
- Non-goals: changing protocol version 1, altering runtime request or response behavior, generating native declarations from schemas, or treating tool descriptions as compatibility surfaces.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `contract-governance`: Strengthens fixture-governed synchronization so the parity gate is structurally coupled to actual Rust request variants, complete client-visible MCP tool definitions, and TypeScript compile-time checks.

## Impact

- Affects the canonical MCP surface catalog, Rust headless request metadata and dependencies, the TypeScript contract harness, the parity command, tests, and contract-governance guidance.
- Adds a direct `strum` dependency already present transitively in the workspace lockfile.
- Does not change public runtime wire behavior or persisted data.
