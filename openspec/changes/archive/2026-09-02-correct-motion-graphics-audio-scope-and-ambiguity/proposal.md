## Why

The fixture-only motion-graphics validators still treat composition-scoped audio-event and marker identities and limits as global within an invalid envelope. Their ambiguity exemptions also disable uniqueness for entire collections, allowing unrelated duplicate keys to coexist with the fixture's declared defect.

## What Changes

- Key invalid audio-event and marker definitions by their exact composition scope and ID.
- Enforce audio-event and marker limits independently for each `root` or `component:<id>` owner.
- Permit only the one lookup key responsible for a fixture's declared marker, bus, sound-definition, or variant ambiguity and reject every unrelated duplicate.
- Add mirrored Rust and TypeScript regressions and synchronize fixture documentation and the living requirement.
- Keep catalog version 1 `fixture_only`; this is not runtime activation.

Non-goals are project schema changes, production motion-graphics types, migrations, headless or MCP operations, capabilities, providers, renderer behavior, stable runtime errors, dependencies, generated schemas, ownership changes, or edits to archived OpenSpec packages.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-contracts`: Require exact composition ownership for invalid audio identities and limits and exact-key ambiguity exemptions in both language validators.

## Impact

The test-only Rust and TypeScript motion-graphics validators, focused contract regressions, fixture documentation, and living motion-graphics specification are affected. Runtime, persistence, transport, rendering, packaging, ownership, dependencies, and catalog version remain unchanged.
