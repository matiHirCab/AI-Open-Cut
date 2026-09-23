## ADDED Requirements

### Requirement: Owned animation channel modules
Serialized animation channel records MUST belong to the editor-core model owner, and compatibility, bounds, target, and legacy-collision rules MUST belong to the validation owner. Timeline edits and scene evaluation MUST consume those owners without a new cyclic private-owner dependency. ADR 0003 and the architecture test MUST cover every top-level private editor-core module so an undeclared owner cannot evade the allowed-dependency matrix.

#### Scenario: Trace a channel rule to one owner
- **WHEN** a contributor changes a channel's persisted shape or edit validity
- **THEN** the model owns the shape, validation owns the rule, and timeline and scene code delegate without a parallel validator

#### Scenario: Reject an unlisted private owner
- **WHEN** a new top-level private editor-core module is declared without an allowed-dependency entry
- **THEN** the architecture check fails before accepting its production code
