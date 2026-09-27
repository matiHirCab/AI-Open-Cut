## Why

The canonical MCP catalog is 19,998,586 bytes and 360,953 lines. Repeated structural schemas obscure meaningful changes in review and make manual synchronization under ADR 0002 unnecessarily costly.

## What Changes

- Compact exact repeated schema subtrees into readable, named, catalog-local definitions and references while retaining `contracts/mcp-surface-v1.json` as the single manually governed canonical artifact.
- Expand and validate the stored representation before the existing complete MCP parity comparison. Add deterministic expansion, baseline-equivalence, malformed-reference, and structural-drift tests.
- Record before-and-after byte and line counts and keep the standalone contract gate and CODEOWNER review authoritative.
- Non-goals: changing any client-visible MCP tool, input or output schema, annotation, prompt, resource, capability identifier, catalog version, protocol version, or runtime validator; generating native declarations or the canonical catalog from code; changing persisted data or migrations.
- Compatibility: this is an internal canonical-storage change. No public contract changes are intended, so no additive or breaking protocol change or migration is required. Any discovered public difference blocks implementation until separately specified and approved.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `contract-governance`: Permit a losslessly compact canonical MCP catalog while requiring fail-closed expansion and unchanged complete parity evidence.

## Impact

- Canonical MCP catalog, its TypeScript contract-parity consumer and focused tests, and contract-governance guidance. Ownership remains with `contracts/mcp-surface-v1.json` and its existing governed consumers under ADR 0002.
- No Rust, Python, desktop, provider, or public wire declarations are expected to change. Designated `@matiHirCab` CODEOWNER review and all required repository checks still apply.
