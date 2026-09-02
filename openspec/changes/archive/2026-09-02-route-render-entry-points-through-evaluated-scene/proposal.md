## Why

OpenCut currently evaluates a renderer-neutral `EvaluatedScene` but discards it, while production frame preview, range preview, draft preview, and export still build plans by traversing persisted project records. Routing every entry point through the canonical evaluated representation is required to prevent preview/export semantic drift and complete the issue #12 foundation for issue #13.

## What Changes

- Replace the borrowed project-backed render-planning input with `EvaluatedScene` plus its process-local resource-binding sidecar for frame preview, audiovisual range preview, draft preview, and final export.
- Adapt FFmpeg planning and resource preparation to consume only evaluated instructions and explicit logical bindings; remove the duplicate production traversal of tracks, items, transitions, animation, and audio semantics.
- Preserve immutable-revision checks, draft isolation, managed path policy, atomic artifact publication, stable errors, and existing simple rendering behavior.
- Add deterministic visual/audio parity coverage across all render intents, including invalid input, missing references, revision conflict, undo/redo, and deterministic reopen scenarios where they apply.
- Add an additive renderer capability identifier so headless and MCP clients can distinguish canonical evaluated-scene rendering support, and synchronize the typed status surface, Zod schemas, canonical fixtures, and parity tests.
- Document observable timing, coordinate, ordering, fallback, validation, and preview/export tolerance semantics.

**Non-goals:** This change does not add persisted motion-graphics fields, reusable components, hierarchy, masks, effects, executable SVG, network resources, arbitrary paths, new timeline mutations, batch aliases, schema migration, or new stable error codes. It does not activate future fixture-only motion-graphics primitives beyond current flat media, text, solid-color, rectangle, and audio behavior.

**Compatibility:** Existing render requests and responses remain valid and retain their meaning. The new capability identifier is additive; project schema version 6, retained history, simple operations, and existing error-code meaning remain unchanged. There are no breaking changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rendering-export`: Require every preview and export entry point to consume the canonical evaluated scene, define parity tolerance and failure-before-publication behavior, and expose additive capability detection.
- `motion-graphics-architecture`: Complete the production substitution from the temporary borrowed planner to `EvaluatedScene` and its separate path-safe bindings without weakening renderer neutrality.
- `editor-core-architecture`: Enforce that render planning no longer traverses persisted project/timeline records outside the canonical evaluator.
- `contract-governance`: Govern the additive capability/status change across Rust, TypeScript/Zod, MCP, and canonical cross-language fixtures.

## Impact

- `crates/editor-core`: evaluated-scene consumption, resource preparation, render planning, renderer entry points, architecture and parity tests.
- `apps/headless`: additive renderer capability reporting and protocol evidence; existing render request operations remain compatible.
- `apps/agent-bridge`: status schema/capability propagation, MCP schema/catalog parity, and integration tests.
- `contracts`: headless and MCP canonical catalogs plus ownership-governed parity evidence for the additive capability.
- `docs` and ADR 0004: canonical routing, deterministic tolerance, and observable semantics.
- No new runtime dependency, provider protocol, persisted schema, migration, or backend-specific public contract.
