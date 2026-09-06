## ADDED Requirements

### Requirement: Reusable slotted rule-card lifecycle fixture
The suite MUST include a deterministic local rule-card fixture using one shared definition with at least six visual child layers and exactly three root instances with different text, displayed rule numbers and managed icon assets. It MUST use existing typed slot contracts: displayed numbers are text content, with a separate numeric opacity slot exercising the number kind. A shared root parent MUST permit one transform edit to move all three cards; each instance MUST retain independent overrides and the definition MUST remain unchanged. Synthetic assets, font identity, dimensions, timing, sample timestamps and expected transforms/order MUST be recorded reproducibly. The fixture MUST exercise existing standalone operations and an atomic batch with creation aliases, without adding transport or persisted fields.

#### Scenario: Construct three independently slotted cards
- **WHEN** the fixture creates its definition, slots, parent and three instances through existing typed APIs
- **THEN** it produces six or more visual children per occurrence with independently verified text, number/opacity and icon values, stable scoped identities and unchanged shared base tracks

#### Scenario: Move parent through the complete lifecycle
- **WHEN** one parent transform moves all cards, followed by undo, redo and reopening from a fresh core instance
- **THEN** independently expected child transforms, slots and stacking match each lifecycle state and retained history restores the complete edit

#### Scenario: Reject invalid rule-card mutations atomically
- **WHEN** a fixture variant supplies invalid slot content, a missing component or asset, a cyclic parent, a locked target, a stale revision or a later failing batch operation
- **THEN** the documented INVALID_ARGUMENT, ITEM_NOT_FOUND, ASSET_NOT_FOUND, TRACK_LOCKED or retryable REVISION_CONFLICT applies and authoritative project/history files and revision remain unchanged

### Requirement: Rule-card preview and export conformance
Rule-card still preview, audiovisual range preview and export MUST consume production EvaluatedScene semantics at common timestamps before and after parent movement and after undo, redo and reopen. The suite MUST compare independent transform/order/slot expectations and reviewed visual references, with SSIM at least 0.99, bidirectionally aligned decoded float-PCM RMS error at most 0.0001 and timing deviation at most one output video frame. Synthetic local audio MUST make audiovisual checks non-vacuous. Repeated immutable evaluations MUST have identical semantic plans; comparing encoded container bytes MUST NOT be the conformance oracle. Missing configured rendering tools or deterministic fonts MUST fail the required native gate. The existing flat-scene baseline, tolerances and report work counts MUST remain intact; rule-card references MUST be separately identified and validated, with deliberate updates only.

#### Scenario: Compare every lifecycle state across render intents
- **WHEN** the required native suite renders common timestamps for original, moved, undone, redone and reopened rule-card states
- **THEN** all three intents meet independent expectations and reviewed visual/audio/timing tolerances, including expected overlap and parent displacement

#### Scenario: Detect drift and missing dependencies
- **WHEN** a child fails to inherit movement, instances share wrong overrides, output order drifts, all intents drift together or a required dependency is absent
- **THEN** conformance fails rather than accepting cross-intent agreement or skipping native verification
