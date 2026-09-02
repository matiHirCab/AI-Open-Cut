## Why

The fixture-only motion-graphics invalid-audio validators validate the shape of `soundDefinition.busId` but do not require it to resolve to a declared project bus. An unrelated missing sound-definition bus can therefore be hidden by every canonical audio fixture's expected failure.

## What Changes

- Require every invalid-audio sound definition's bus reference to resolve before resource or semantic failure classification.
- Preserve the one exact bus duplication permitted by `audio_event.ambiguous_bus` while still requiring the referenced bus to exist.
- Add mirrored Rust and TypeScript mutations across every canonical invalid audio fixture, including controls that restore the missing bus.
- Synchronize fixture documentation and the living motion-graphics requirement.
- Keep catalog version 1 `fixture_only`; this is not runtime activation.

Non-goals are catalog fixture changes, project schema changes, production motion-graphics types, migrations, headless or MCP operations, capabilities, providers, renderer behavior, stable runtime errors, dependencies, generated schemas, ownership changes, or edits to archived OpenSpec packages.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-contracts`: Require project-bus resolution for sound definitions in every invalid-audio envelope before its declared defect is classified.

## Impact

The test-only Rust and TypeScript motion-graphics validators, their malformed-payload regressions, fixture documentation, and living motion-graphics specification are affected. Runtime, persistence, transport, rendering, packaging, ownership, dependencies, catalog contents, and catalog version remain unchanged.
