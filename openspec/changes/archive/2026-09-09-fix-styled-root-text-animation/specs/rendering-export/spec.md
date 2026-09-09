## ADDED Requirements

### Requirement: Styled root text retains legacy animation
Ungrouped styled root text MUST retain identity ancestry throughout evaluation, measurement and rendering so position, scale and opacity keyframes follow existing timing/easing semantics. All render intents MUST match independently calculated static styled-text snapshots at equivalent timestamps within existing tolerances. Plain text MUST retain its legacy path, real parent ancestry MUST remain intact, and Transform2D MUST retain precedence. No public or persisted contract changes are introduced.

#### Scenario: Animate each supported legacy property
- **WHEN** ungrouped styled text has position, scale or opacity keyframes sampled at 0, 400 and 800 milliseconds
- **THEN** its displacement, visible bounds or brightness changes as expected and output matches static snapshots with independently calculated values

#### Scenario: Preserve ancestry and transform compatibility
- **WHEN** styled text has a non-default anchor, a real group/component parent, or an explicit Transform2D, or text is semantically plain
- **THEN** existing anchor, parent composition, Transform2D precedence and plain rendering semantics remain unchanged and evaluator ancestry is preserved through finalization

#### Scenario: Agree across render intents without freezing
- **WHEN** the animated project renders through frame, range, materialized draft and export at the same selections
- **THEN** outputs satisfy existing decoded-content tolerances, exhibit the independently expected animation and leave authoritative state unchanged
