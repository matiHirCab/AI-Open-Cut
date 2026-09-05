## Why

Issue #23 requires reusable components to expose validated text, rich-text, color, number, Boolean, enum, duration and managed-asset inputs. Schema 11 stores component timelines but has no runtime slot definitions, bindings or instance values; the foundation catalog only demonstrates preparatory text-slot fixtures.

## What Changes

- Add closed typed slot definitions, constrained defaults, stable local property bindings and typed nested-instance overrides, validated by editor-core.
- Add `component_define_slots` as a standalone and ordered batch operation. Extend component creation/replacement compatibly and validate incoming instance values whenever a definition changes.
- Introduce schema 12 with atomic migration of current state and retained history, and protect managed assets referenced only by slots or overrides.
- Publish canonical runtime fixtures, mirrored headless/MCP contracts, capability `typed_template_slots`, documentation and conformance tests.
- Preserve root rendering and existing revision, lock, rollback, draft, undo/redo and reopen guarantees.

## Capabilities

### New Capabilities
- `template-slots`: Eight typed input kinds, constraints, local bindings, default/override resolution and atomic slot editing.

### Modified Capabilities
- `component-definitions`: Compatible slot-aware definition creation/replacement and nested-instance value storage.
- `project-persistence`: Schema-12 migration and slot asset retention.
- `agent-bridge`: Typed slot operation and discovery through headless and MCP.
- `motion-graphics-contracts`: Canonical runtime slot evidence and explicit separation from preparatory fixtures.

## Impact

Affected owners are editor-core model/validation/timeline/migrations/assets, thin headless transport, bridge contracts/schemas/registration, canonical catalogs and their listed consumers. No provider or desktop workflow changes are intended. Public protocol version 1 gains additive operations and optional request fields; old requests retain their meaning. **BREAKING persisted format:** schema 12 requires migration, is unreadable by older binaries and has no downgrade path. No existing stable error is renamed or reclassified. Contract review belongs to @matiHirCab.

## Non-goals

Root component instantiation, instance rendering/time evaluation (#24), a general rich-text renderer, arbitrary JSON-path binding, expressions, network resources, filesystem asset inputs, and dynamic slot-to-slot binding are excluded. Rich text is validated and stored as a typed document for later component evaluation; no plain-text flattening is introduced as a rendering fallback.

## Approval status

Approved by the user in this task on 2026-09-05 ("Approve"), covering proposal, delta specs, design and tasks. Implementation proceeds through tasks.md; approval does not claim implementation or verification is complete.

Final implementation and public contract review approved by the user in this task on 2026-09-05 (second approval of the completed diff), satisfying the designated CODEOWNER review and authorizing specification synchronization, archival and the final gate.
