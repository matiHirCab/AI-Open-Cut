## MODIFIED Requirements

### Requirement: Stored local component timelines
Projects MUST support reusable component definitions with stable project-unique ID, name, dimensions, explicit positive duration and local tracks. Existing ordinary item and track semantics MUST apply inside the composition, with IDs unique within that scope and parent/transition references confined to it. Component-local parents MUST use component:<id> scope. Typed component_instance items MUST be restricted to root or component overlay tracks and contain common static visual/order properties and componentId, startMs, trimStartMs, durationMs and finite positive timeScale. Root placement MUST follow the component-evaluation requirements. Instance transition endpoints and animated instance properties MUST continue to fail with INVALID_ARGUMENT.

#### Scenario: Persist an empty or populated definition
- **WHEN** a valid positive-duration definition is created with empty tracks or compatible local items
- **THEN** reads preserve its exact values independently of root tracks and root duration

#### Scenario: Confine local references
- **WHEN** local IDs match IDs in another definition or root, or a local parent or transition points outside its composition
- **THEN** valid independent IDs remain distinct and cross-scope references fail without mutation

### Requirement: Stored slots preserve current rendered output
Slots and stored nested-instance values in definitions unreachable from root instances MUST NOT change root duration, ordering, pixels, audio or fallback selection. Reachable instances MUST render their effective slot values under the component-evaluation requirements. Frame, range, draft preview and export MUST retain the shared evaluated behavior. Invalid slot content, including hidden/unused definitions, MUST fail before render process execution or output preparation.

#### Scenario: Compare root output with stored slots
- **WHEN** valid definitions unreachable from root instances receive slots and nested-instance values without root edits
- **THEN** all root render entry points retain equivalent evaluated output

#### Scenario: Reject malformed direct render data
- **WHEN** a direct render input contains invalid bindings or values in an unused definition
- **THEN** core rejects it before preparing or publishing artifacts
