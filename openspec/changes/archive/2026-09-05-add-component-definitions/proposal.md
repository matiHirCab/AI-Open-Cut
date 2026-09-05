## Why

Issue #22 requires reusable component definitions with local timelines, duration rules and bounded nested references. The current schema-10 model contains only root tracks and groups; component vocabulary is still fixture-only. This is the persistence and validation foundation for slots (#23) and instance evaluation (#24).

## What Changes

- Advance the current persisted schema to 11, migrating schemas 1 through 10 and all retained history atomically. The issue's schema-v8 label predates shipped schemas 8–10 and must not be reused.
- Add project-owned component definitions with dimensions, explicit duration, local tracks and typed nested component-instance records. Validate the complete definition graph, including unused definitions.
- Expose additive typed component_create, component_update and component_delete operations standalone and in batches, plus component_definitions discovery. Definition editing replaces a complete local timeline atomically; existing root item operations retain root-only meaning.
- Keep root placement and instance evaluation deferred to #24. Nested instance records are validated stored content inside definitions; they are not silently accepted as drawable root items.
- Govern the activated subset through a new component-definitions-v1 catalog and update native consumers, ownership, migration, media retention, docs and conformance evidence.

## Capabilities

### New Capabilities

- `component-definitions`: Stored local timelines, scoped references, finite timing, graph bounds and transactional definition editing.

### Modified Capabilities

- `project-persistence`: Schema-11 current/history migration and component media retention.
- `motion-graphics-contracts`: Runtime component definition contracts and compatibility evidence.
- `agent-bridge`: Typed definition operations, discovery and real standalone/batch workflows.

## Impact

Owning core model, validation, timeline, migration, asset/draft/persistence integration; headless and bridge typed consumers; canonical catalogs and CODEOWNERS; documentation and tests. Preserve ADR 0003 dependency direction and ADR 0004 root-track architecture.

Public operations and capability are additive under protocol major 1. **Persisted compatibility boundary:** schema 11 requires a schema-11 reader, with deterministic forward migration and no downgrade. Existing root operations, stable errors, renderer output and simple-project semantics remain unchanged.

## Non-goals

Slots/bindings, root component placement, component rendering or time-map evaluation, automatic duration inference, world-transform baking, recursive deletion, component-scoped variants of every existing root edit, provider changes and new external resource inputs.

## Approval

The user explicitly replied "Approve" on 2026-09-05, authorizing this proposal, design, all delta scenarios and tasks before implementation.

On 2026-09-05, the user replied "I approve" to the final designated CODEOWNER review request for the implemented schema-11 contracts, consumers and parity evidence, authorizing synchronization and archive.

## Approved review correction (2026-09-05)

The subsequent review reproduced unsafe nested timestamps, invalid caption metadata and non-media volume keyframes being persisted. The user explicitly requested "PLEASE IMPLEMENT THIS PLAN" with the complete correction plan, approving these correction requirements before implementation. Earlier final review evidence applies to the previous implementation and does not certify this correction.

Keep schema 11 and public shapes unchanged. Invalid existing component records fail closed without repair. Root operations and provider behavior remain unchanged.

## Final correction approval

On 2026-09-05, designated CODEOWNER @matiHirCab replied "I approve the change, revise if everything was implemented correctly" to the final correction review request. A subsequent review found no actionable defects; 12 component tests and 26 bridge contract/schema tests passed again. This approval authorizes synchronization and re-archive of the corrected implementation and evidence.
