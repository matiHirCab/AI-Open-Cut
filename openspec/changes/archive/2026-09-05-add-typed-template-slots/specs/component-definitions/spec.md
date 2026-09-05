## ADDED Requirements

### Requirement: Compatible slot-aware component editing
Component creation MUST accept optional `slots`, defaulting to empty. Component update MUST accept optional `slots`; omission MUST preserve existing slots, while an explicit array MUST replace them. Existing required definition fields and full track replacement MUST retain their meaning. Nested component_instance requests MUST accept optional `slotValues`, defaulting to empty, with slot IDs as keys and typed values as entries. Every create/update MUST validate all resulting local bindings and incoming instance overrides against the evolving graph. Current locks, graph limits and timing checks MUST remain in force, with effective slot duration/asset values also validated. Draft materialization and direct renderer validation MUST reuse these core rules.

#### Scenario: Preserve older requests
- **WHEN** a pre-slot client creates or updates a definition without the new optional fields
- **THEN** creation yields empty slots, update retains existing slots, and unchanged tracks preserve all prior semantics

#### Scenario: Replace definitions with incoming slot references
- **WHEN** a definition update removes a bound item, removes a used slot, changes its type, or invalidates effective instance timing
- **THEN** the whole operation fails atomically without pruning references or clamping values

### Requirement: Stored slots preserve current rendered output
Slots and stored nested-instance values MUST NOT change root duration, ordering, pixels, audio or fallback selection. Root instance placement and component-instance rendering MUST remain deferred. Frame, range, draft preview and export MUST retain the shared evaluated behavior. Invalid slot content, including hidden/unused definitions, MUST fail before render process execution or output preparation.

#### Scenario: Compare root output with stored slots
- **WHEN** valid definitions receive slots and nested-instance values without root edits
- **THEN** all root render entry points retain equivalent evaluated output

#### Scenario: Reject malformed direct render data
- **WHEN** a direct render input contains invalid bindings or values in an unused definition
- **THEN** core rejects it before preparing or publishing artifacts
