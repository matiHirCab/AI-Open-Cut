## ADDED Requirements

### Requirement: Common visual properties activate as an isolated persisted milestone
Schema version 7 MUST establish common visual-property ownership for current transform and visibility state without activating the fixture-only Transform2D, layer, component, slot, marker, curve, mask, effect, or audio-event concepts. `contracts/motion-graphics-v1.json` MUST remain fixture-only, and no new public operation, capability identifier, provider surface, renderer expression, resource locator, or stable error SHALL be introduced by this milestone.

#### Scenario: Inspect milestone boundaries
- **WHEN** a schema-v7 project, public operation catalog, capability report, and motion-graphics fixture catalog are inspected
- **THEN** common transform and visibility ownership is present, existing operations retain their meanings, the motion-graphics catalog remains fixture-only, and all later concepts remain inactive

#### Scenario: Defer Transform2D behavior
- **WHEN** a common visual property is evaluated in this milestone
- **THEN** it uses the schema-v6 position, uniform scale, and opacity semantics with no units, anchor transform, independent scale, rotation, skew, or new transform ordering

## MODIFIED Requirements

### Requirement: EvaluatedScene foundation remains non-public and non-persisted
`EvaluatedScene` MUST remain an editor-core process-local derivation and MUST NOT itself persist state or add a public request, response, operation, MCP, provider, or stable-error contract. Separately approved persisted-schema and render-routing milestones MAY migrate a project before evaluation or advertise their own capability without making `EvaluatedScene` a persisted or public model.

#### Scenario: Evaluation does not mutate persisted state
- **WHEN** an already opened project revision is evaluated
- **THEN** evaluation itself does not rewrite the project, change its revision, or modify retained undo/redo history; any supported older-schema migration occurs during project opening under the separately specified persistence workflow

#### Scenario: Keep the scene model private
- **WHEN** clients inspect headless operations, MCP tools, provider contracts, or stable errors
- **THEN** they observe no serialized or directly addressable `EvaluatedScene` model
