## Why

PR #93's strengthened parity gates still permit a Serde wire-tag rename to drift from Strum's independently configured variant names, and the MCP schema normalizer does not enforce the documented exclusion of description-only copy. These gaps make the gate either miss a real Rust wire change or flag a non-compatibility documentation edit.

## What Changes

- Verify that every Strum-derived canonical Rust operation name is accepted as an actual Serde request tag, so Serde and Strum naming cannot drift silently.
- Add a focused regression proving a variant-specific Serde rename is detected without relying on the checked-in operation catalog changing.
- Recursively omit JSON Schema `description` fields from normalized MCP tool input/output schemas while preserving all other schema keywords and array order.
- Add focused normalization coverage proving nested descriptions are ignored but structural schema changes remain detectable.
- Keep runtime protocol version 1, request/response behavior, MCP registrations, and the manually governed catalog unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `contract-governance`: Clarifies that Rust parity evidence must verify the actual Serde wire tags and that description-only MCP schema metadata is excluded from compatibility comparison.

## Impact

- Affects only development-time contract tests, the contract-governance specification and documentation, and the follow-up OpenSpec archive.
- Does not change runtime wire behavior, dependencies, persisted data, or the canonical MCP catalog's current structural content.
- Non-goals: replacing fixture-governed manual synchronization, generating Rust/TypeScript declarations, or excluding structural JSON Schema metadata other than `description`.
