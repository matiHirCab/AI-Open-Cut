## Why

The fixture-only motion-graphics catalog currently reports exact cross-language failure evidence without deriving the observed failure, leaves most declared complexity limits unenforced, and permits Rust/TypeScript validation drift. These gaps let materially incorrect fixture payloads pass the contract gate even though the living specification promises strict deterministic validation.

## What Changes

- Make invalid fixtures complete except for one intentional defect and require both validators to derive the exact declared classification and reason.
- Expand fixture scenario envelopes where graphs, supplied values, or duplicate resolution candidates are required to exercise hierarchy, slot, marker, and audio failures.
- Enforce every represented catalog limit, remove unsupported inline-SVG complexity limits, and reject illegal kind/scope combinations and duplicate definitions or metadata.
- Align closed Rust Serde and TypeScript Zod catalog/payload validation, including safe-integer bounds, Unicode scalar length, identifiers, dimensions, ranges, collections, enums, and required fields.
- Add cross-language mutation and boundary evidence, including payload swaps, all limit boundaries, cycles, reference failures, resource safety, and ordinary-text opacity.
- Keep version 1 `fixture_only`; this is a corrective fixture contract change, not a runtime activation or breaking public contract.

Non-goals are production motion-graphics types, project migrations, headless or MCP operations, capabilities, provider protocols, renderer behavior, stable runtime errors, schema generation, or new dependencies. The archived `add-motion-graphics-contract-fixtures` and `harden-motion-graphics-contract-fixtures` changes remain immutable.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-contracts`: Require failure-derived negative evidence, complete limit and duplicate enforcement, and identical Rust/TypeScript boundary semantics for the fixture-only catalog.

## Impact

The canonical `contracts/motion-graphics-v1.json` fixture catalog, its test-only Rust and TypeScript validators, focused contract tests, fixture documentation, ownership metadata, and the living motion-graphics contract specification are affected. No persisted, public, provider, transport, capability, evaluation, renderer, or packaging surface changes, and no migration or dependency is introduced.
