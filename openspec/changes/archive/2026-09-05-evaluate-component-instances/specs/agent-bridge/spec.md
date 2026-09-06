## ADDED Requirements

### Requirement: Typed root component instance workflows
Headless and MCP MUST expose add_component_instance and component_instance_update standalone and inside timeline_batch_edit with existing project/revision envelopes and mutation results. Create MUST accept resultAlias; update MUST NOT produce an alias. Earlier aliases MUST resolve in trackId, componentId, itemId and parent.id where present; local IDs and slot-map keys MUST remain literal. Transport adapters MUST delegate timing, graph, effective slots, locks, persistence and rendering validation to core. Documentation MUST describe schema 13, half-open fractional derived timing, coordinates, ordering, visibility/audio, limits, errors and unsupported behaviors, superseding earlier deferred-rendering documentation only for component_instance_evaluation-capable runtimes.

#### Scenario: Exercise real clients
- **WHEN** source integration and packaged clients create, update, batch, preview and export component instances then undo, redo and reopen
- **THEN** typed requests/results agree with native state, aliases resolve correctly and invalid input, missing references, locks and revision conflicts preserve atomicity
