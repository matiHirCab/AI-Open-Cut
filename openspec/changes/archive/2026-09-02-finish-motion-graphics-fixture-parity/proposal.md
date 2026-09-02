## Why

The fixture-only motion-graphics validators still diverge after the latest correction: Rust normalizes duplicate payload definitions before rejecting them, accepts malformed negative scenario fields that Zod rejects, and the corrective tests accept unrelated failures instead of proving the named invariant. These gaps allow the contract gate to remain green without establishing the strict causal parity promised by the living specification.

## What Changes

- Reject duplicate layer, component-layer, mask, and effect definitions in Rust before `BTreeSet` normalization and aggregate counting.
- Add mirrored semantic preflight validation for every invalid scenario family, validating all fields except the fixture ID's one intentional defect.
- Align negative-scenario identifiers, scopes, safe integers, finite ranges, collections, constraints, and Unicode-scalar name lengths across Rust and Zod.
- Make mutation helpers assert the exact expected invariant, including named aggregate limits and duplicate/resource/envelope failures.
- Add cross-language regressions for duplicate payload definitions and malformed negative envelopes while preserving every canonical exact classification/reason result.
- Keep catalog version 1 `fixture_only`; this is not a public or persisted runtime activation.

Non-goals are project schema changes, production motion-graphics types, migrations, headless or MCP operations, capabilities, providers, renderer behavior, stable runtime errors, dependencies, generated schemas, or changes to archived OpenSpec packages.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-contracts`: Require pre-normalization duplicate rejection, strict causal validation of negative scenario envelopes, and exact mutation failure assertions in both language suites.

## Impact

The test-only Rust and TypeScript motion-graphics validators, focused contract regressions, fixture documentation, and living motion-graphics specification are affected. No runtime, persisted, transport, provider, rendering, packaging, ownership, dependency, or catalog-version surface changes unless a test helper must move.
