## Why

The fixture-only motion-graphics validators still permit invalid scenario envelopes to contain unrelated duplicate definitions and collection-limit violations. Their component dependency classifiers also collapse multiple outgoing edges, which can hide a reachable cycle or undercount the longest reachable depth.

## What Changes

- Reject duplicate definitions and context identifiers across every invalid scenario envelope before semantic classification, with narrow fixture-ID-specific exemptions for declared ambiguity evidence.
- Apply every relevant named catalog collection limit to invalid scenario envelopes before deriving their intended failure.
- Preserve every component dependency edge and classify missing references, branching cycles, and longest reachable depth deterministically.
- Add mirrored Rust and TypeScript regressions and synchronize fixture documentation and the living requirement.
- Keep catalog version 1 `fixture_only`; this is not runtime activation.

Non-goals are project schema changes, production motion-graphics types, migrations, headless or MCP operations, capabilities, providers, renderer behavior, stable runtime errors, dependencies, generated schemas, ownership changes, or edits to archived OpenSpec packages.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-contracts`: Require complete invalid-envelope uniqueness and limit preflight plus lossless branching component-graph validation in both languages.

## Impact

The test-only Rust and TypeScript motion-graphics validators, focused contract regressions, fixture documentation, and living motion-graphics specification are affected. Runtime, persistence, transport, rendering, packaging, ownership, dependencies, and catalog version remain unchanged.
