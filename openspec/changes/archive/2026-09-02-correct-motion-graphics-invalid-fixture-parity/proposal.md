## Why

The fixture-only motion-graphics validators still permit four forms of cross-language or causal drift: invalid fixtures can be relabeled with another concept, TypeScript accepts duplicate definitions in several invalid envelopes that Rust rejects, Rust scans only the first mask and recognizes only one SVG event handler, and Rust accepts catalog limits beyond JavaScript's safe-integer range.

## What Changes

- Make the independent invalid-fixture expectation matrices include the exact concept as well as classification and reason.
- Reject duplicate invalid layer, mask, and renderer-expression effect definitions in TypeScript before classification.
- Make Rust scan every invalid mask and recognize the same executable SVG event-handler forms as TypeScript.
- Restrict every Rust catalog limit to the positive JavaScript-safe-integer range.
- Add mirrored causal regressions and synchronize fixture documentation and the living requirement.
- Keep catalog version 1 `fixture_only`; this is not runtime activation.

Non-goals are project schema changes, production motion-graphics types, migrations, headless or MCP operations, capabilities, providers, renderer behavior, stable runtime errors, dependencies, generated schemas, ownership changes, or edits to archived OpenSpec packages.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-contracts`: Require exact invalid-fixture concepts, invalid-envelope definition uniqueness, complete mask safety inspection, and JavaScript-safe catalog limits in both language validators.

## Impact

The test-only Rust and TypeScript motion-graphics validators, focused contract regressions, fixture documentation, and living motion-graphics specification are affected. Runtime, persistence, transport, rendering, packaging, ownership, dependencies, and catalog version remain unchanged.
