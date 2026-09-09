## Why

Issue [#31](https://github.com/matiHirCab/AI-Open-Cut/issues/31) requires a compact way to repeat shapes, groups, and component instances without persisting expanded copies or relying on renderer-specific workarounds. Component-instance evaluation (#24), vector primitives (#27), and shape rendering (#28) are already implemented, so repeaters can now be added at the canonical editor-core evaluation boundary.

## What Changes

- Introduce a bounded persisted `RepeaterItem` that references a shape, group, or component instance and describes an exact number of additional copies, a finite per-copy `Transform2D` offset, and a finite per-copy opacity offset.
- Add headless `add_repeater` and MCP `timeline_add_repeater`, complete repeater replacement through `update_item`, atomic batch aliases, and the applicable non-drawing-item lifecycle in root and component scopes.
- Expand repeater occurrences lazily in editor-core evaluation, with deterministic copy identity, timing, ordering, transform composition, opacity clamping, hierarchy traversal, and explicit per-repeater and aggregate occurrence limits.
- Route the same expanded `EvaluatedScene` through frame preview, audiovisual range preview, materialized drafts, and final export; downstream renderers do not inspect persisted repeater records.
- Advance persisted schema 16 to 17 and atomically migrate supported current state and retained undo/redo history without altering existing scene output.
- Publish strict Rust/TypeScript contracts, canonical fixtures, capability evidence, documentation, and automated success/failure, transaction, migration, evaluation, and render-parity coverage.

This is additive to protocol 1, but schema-17 projects containing the new item variant require repeater-aware readers. No existing request or simple item behavior changes meaning.

## Capabilities

### New Capabilities

- `repeaters`: Strict repeater descriptors, reference and cycle validation, bounded lazy occurrence expansion, and deterministic transform/opacity behavior.

### Modified Capabilities

- `timeline-editing`: Transactional repeater creation, replacement, aliases, lifecycle restrictions, and reference-safe deletion.
- `project-persistence`: Atomic schema-17 repeater activation and fail-closed source-version validation across current and retained history.
- `agent-bridge`: Typed discoverable repeater operations, capabilities, and governed cross-language parity.
- `rendering-export`: Shared complete repeater evaluation and rendering for every preview/export intent.

## Impact

The core model, strict decoding, validation, timeline mutations, migrations, component definitions, hierarchy checks, evaluated scene traversal, occurrence budgeting, and exhaustive item consumers require repeater support. Headless unions, bridge Zod schemas/tool registration, capability and operation catalogs, contract ownership/fixtures, desktop generic item handling, documentation, and integration/smoke fixtures must remain synchronized.

The public operation is additive under protocol 1. Persisted schema 17 is forward-only: older binaries reject it and no downgrade is provided. Existing stable errors and retryability remain unchanged; invalid values, targets, cycles, or complexity return non-retryable `INVALID_ARGUMENT`, missing targets return `ITEM_NOT_FOUND`, locked edits return `TRACK_LOCKED`, and stale revisions return retryable `REVISION_CONFLICT` according to existing precedence. No new dependency edge, provider protocol, network access, arbitrary path, executable SVG, raw FFmpeg expression, or rendering backend is introduced.

## Non-goals

Repeater time offsets, stagger/inherited animation, repeating media/text/SVG/grid/caption/transition items, arbitrary expressions, renderer-specific modifiers, source mutation, destructive expansion into persisted items, desktop repeater authoring controls, cross-composition references, and schema downgrade are excluded. Time offsets remain assigned to later milestone MG-M3-05 (#42).

## Approval status

Approved by the user in this task on 2026-09-07 with “Approve,” covering this proposal, the design, all five delta specifications, and the implementation tasks. At that approval point, the separate designated review of resulting canonical contracts and governed consumers remained pending; it was completed later on 2026-09-07 as recorded in `tasks.md` task 5.5 and the verification evidence.
