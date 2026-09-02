## Why

The fixture-only motion-graphics catalog still permits aggregate limit overflow, duplicate fixture identities in TypeScript, non-asset managed resources, and several Rust payloads that the mirrored Zod schemas reject. These gaps make the current green contract suites insufficient evidence for the living cross-language specification.

## What Changes

- Enforce catalog-wide component, layer, marker, slot, and audio-event limits by their project, root, or component owner while retaining the existing per-payload and graph limits.
- Require fixture IDs to be globally unique across valid and invalid fixtures before maps or failure results are constructed.
- Restrict managed resources to unique project-scoped asset references in both languages.
- Complete Rust/Zod parity for component identifiers and slot-value keys, animation-channel strings, marker identifiers and safe-integer timing, non-empty curve collections, and any adjacent field constraints found by the parity audit.
- Add mirrored mutation and boundary tests proving the corrected behavior, then clarify the fixture documentation and living specification.
- Keep catalog version 1 `fixture_only`; this correction does not activate or break a public or persisted runtime contract.

Non-goals are production motion-graphics types, project schema changes, migrations, headless or MCP operations, capabilities, provider protocols, renderer behavior, stable runtime errors, generated schemas, dependencies, or edits to archived OpenSpec changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-contracts`: Clarify aggregate limit ownership, global fixture identity, exact managed-resource tuples, and strict mirrored field validation for the fixture-only catalog.

## Impact

The checked-in motion-graphics catalog's test-only Rust and TypeScript validators, their focused tests, fixture documentation, and living motion-graphics specification are affected. No runtime, persisted, public, transport, provider, rendering, packaging, ownership, or dependency surface changes unless an implementation helper must move.
