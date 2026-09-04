## ADDED Requirements

### Requirement: Common visual properties own existing visual state
Every media, text, solid-color, rectangle, caption, and transition timeline item MUST own one canonical common visual-properties value containing an identity-compatible transform and visibility flag. The serialized item SHALL retain the existing flattened `transform` and `hidden` field names, and this milestone MUST NOT add transform units, anchors, independent axes, rotation, skew, z-index, parenting, masks, effects, or replacement animation semantics.

#### Scenario: Create an item through an existing operation
- **WHEN** a client creates media, text, solid-color, rectangle, caption, or transition content through an existing operation
- **THEN** editor-core constructs common visual properties using the supplied legacy transform where applicable, identity transform otherwise, and visible-by-default state without changing the operation payload or result semantics

#### Scenario: Serialize common fields compatibly
- **WHEN** a schema-v7 timeline item is serialized in a project response or persisted document
- **THEN** its common transform and visibility values use the existing flattened `transform` and `hidden` keys and no nested compatibility-breaking visual object is required

### Requirement: Existing visual mutations remain compatible
Existing transform updates and visibility mutations MUST update the canonical common visual-properties value with the same validation, optimistic revision, atomic batch, changed-ID, undo, redo, and error behavior as schema version 6.

#### Scenario: Update a transform standalone
- **WHEN** a client submits a valid existing `update_item` transform request at the expected revision
- **THEN** the common transform is updated in one revision and the response preserves existing changed-ID and serialization behavior

#### Scenario: Set visibility inside a batch
- **WHEN** a valid `set_item_visibility` operation targets an earlier creation alias inside `timeline_batch_edit`
- **THEN** the common visibility value is updated within the one atomic batch revision and undo restores the entire prior batch state

#### Scenario: Reject an invalid transform or revision
- **WHEN** an existing visual mutation contains a non-finite or out-of-bounds transform, names a missing item, or supplies a stale expected revision
- **THEN** it fails with the existing stable typed error and does not change current state, history, aliases, or persisted files
