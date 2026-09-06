## ADDED Requirements

### Requirement: Discoverable atomic component lifecycle workflows
Headless and MCP MUST expose component_instance_duplicate standalone and in timeline_batch_edit with existing project/revision envelopes and mutation results. Typed schemas MUST accept itemId, offsetMs and optional closed typed slotValues; batch creation MUST support resultAlias. Runtime status MUST advertise additive component_lifecycle under protocol 1 while retaining existing capabilities. Adapters MUST delegate domain validation and persistence to core. Documentation MUST explain using existing component_create, component_define_slots and add_component_instance for template creation/instantiation, complete override replacement, alias scope, retained coordinates/timing/order, bounds, errors and unchanged schema 13.

#### Scenario: Exercise real complete lifecycle clients
- **WHEN** source integration and packaged clients discover support and create, define, instantiate and duplicate templates standalone and in aliased batches
- **THEN** typed responses, history, reopen and representative preview/export match core behavior, including all supported override kinds

#### Scenario: Preserve failure atomicity through transports
- **WHEN** real clients submit invalid types, missing references, unsafe values, locked targets, stale revisions or a failing trailing edit
- **THEN** structural versus semantic rejection follows the canonical acceptance stage and failed requests preserve project/history bytes and revision
