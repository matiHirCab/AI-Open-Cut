## ADDED Requirements

### Requirement: Production EvaluatedScene consumption
The production renderer MUST consume the complete owned `EvaluatedScene` and its structurally separate process-local resource bindings, MUST keep raw requested and resolved filesystem paths outside `EvaluatedScene`, and MUST derive backend syntax, input indexes, prepared resources, output clipping, and artifact destinations only after canonical evaluation succeeds.

#### Scenario: Prepare a logical media resource
- **WHEN** an evaluated visual or audio layer references a logical asset identifier
- **THEN** path-safe preparation resolves that identifier through the separate binding collection without consulting persisted asset or timeline records and without adding a path to `EvaluatedScene`

#### Scenario: Prepare an evaluated text layer
- **WHEN** an evaluated text instruction references a logical font resource identifier
- **THEN** path-safe preparation resolves its requested path or family through the separate font binding and preserves the evaluated text semantics without consulting the persisted text item

#### Scenario: Reject an inconsistent binding envelope
- **WHEN** an evaluated instruction references a logical media or font resource absent from its binding collection
- **THEN** rendering fails deterministically before backend execution or artifact publication and does not attempt to reconstruct the reference from project records

### Requirement: Intent-independent scene semantics
Frame, range, draft, and export output intents MUST clip and encode one common evaluated scene without changing its resolved coordinate system, half-open timing, layer order, transforms, animation, transition, text, media, or audio facts.

#### Scenario: Select a frame and a range
- **WHEN** frame and range intents select the same timestamp from an equivalent evaluated scene
- **THEN** both consume the same ordered active instructions and differ only in intent-required seeking, duration, audio inclusion, and encoding behavior

#### Scenario: Preserve repeated logical assets
- **WHEN** several evaluated layers reference one logical media asset with different source intervals or audio settings
- **THEN** backend preparation creates deterministic input instances that preserve each layer's evaluated timing and settings without duplicating or reordering canonical scene semantics
