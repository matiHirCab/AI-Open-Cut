# Timeline Editing Specification

## Purpose

Define canonical timeline item, track, validation, atomic editing, and history behavior shared by all transports and presentations.

## Requirements

### Requirement: Typed tracks and timeline items
The editor core SHALL represent video, overlay, audio, and caption tracks and SHALL support media, text, solid-color, rectangle, caption, and transition timeline items with canonical track compatibility rules.

#### Scenario: Add a compatible item
- **WHEN** a caller adds a supported item to a compatible unlocked track with valid timing and properties
- **THEN** the item is persisted with a stable identifier in the resulting revision

#### Scenario: Reject an incompatible item
- **WHEN** an item or media type is assigned to an incompatible track type
- **THEN** validation fails and the timeline remains unchanged

#### Scenario: Enforce specialized placement
- **WHEN** a caller places text outside an overlay track, captions outside a caption track, or audio media outside an audio track
- **THEN** the core rejects the operation as incompatible

### Requirement: Canonical edit validation
All transports MUST rely on editor-core validation for durations, transforms, colors, text styling, audio settings, keyframes, track locks, and item references.

#### Scenario: Reject invalid edit properties
- **WHEN** an edit includes an invalid duration, transform, style, audio value, keyframe sequence, or missing identifier
- **THEN** the core returns a typed validation or not-found error without committing the edit

### Requirement: Complete editing surface
The editor core SHALL support adding, updating, moving, trimming, splitting, duplicating, hiding, and deleting items; setting keyframes and audio; adding transitions; and creating, updating, and deleting tracks.

#### Scenario: Apply a supported edit
- **WHEN** a caller submits one supported operation against the current revision
- **THEN** the core applies that operation, records history, and returns the new revision and changed identifiers

### Requirement: Atomic batch editing
A batch edit MUST contain between one and 100 operations, validate every operation against one evolving candidate state, and SHALL commit all operations as one revision or none of them.

#### Scenario: Commit a valid batch
- **WHEN** every operation in an ordered batch is valid against the state produced by preceding operations
- **THEN** all operations are published together as one revision and one undo step

#### Scenario: Reject an invalid batch
- **WHEN** any operation in a batch is invalid
- **THEN** no operation from the batch is persisted

#### Scenario: Reference a created identifier by alias
- **WHEN** a batch assigns a valid unique result alias to a single-ID creation and a later operation references that alias
- **THEN** the core resolves the alias to the created identifier and returns the alias mapping with the committed batch

### Requirement: Reversible timeline changes
Committed timeline and track edits SHALL participate in project undo and redo while preserving revision ordering and media ownership invariants.

#### Scenario: Undo a batch edit
- **WHEN** a caller undoes a committed batch at the current revision
- **THEN** the complete pre-batch state is restored rather than a partially reversed timeline

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

### Requirement: Typed static Transform2D updates
Common visual properties MUST support optional transform2d with complete position {x,y,unit}, anchor {x,y}, scaleX, scaleY, rotationDeg, skewXDeg, skewYDeg, and opacity. Units SHALL be pixels or normalized. Existing update_item and batch updates MUST accept this value for visual media, text, solids, rectangles, and captions; transition and audio-only targets MUST fail with INVALID_ARGUMENT. Omission MUST preserve the active transform; null MUST restore the retained legacy transform; a legacy transform update MUST clear transform2d. Simultaneous legacy and Transform2D updates MUST fail with INVALID_ARGUMENT. Existing transform and hidden serialization MUST remain compatible. The schema-v7 milestone restrictions remain specific to that historical milestone.

#### Scenario: Set and clear a static transform
- **WHEN** a valid complete Transform2D is set and subsequently cleared with null at current revisions
- **THEN** the first edit activates Transform2D and the second restores the retained legacy value, each as one undoable revision

#### Scenario: Preserve legacy clients
- **WHEN** an old client updates transform on an item with active Transform2D
- **THEN** the submitted legacy transform becomes active and transform2d is cleared without changing legacy operation semantics

#### Scenario: Reject conflicting or unsupported updates
- **WHEN** an update supplies both transform representations, an incomplete Transform2D, or targets a transition or audio-only item
- **THEN** core rejects it with INVALID_ARGUMENT and leaves state and history unchanged

### Requirement: Bounded Transform2D values
Core MUST require finite numbers, anchor in [0,1], independent scales in (0,100], opacity in [0,1], rotation in [-36000,36000], skews in [-80,80], and position absolute magnitude <= 1000000 pixels or <= 100 normalized units. Unknown fields and units MUST be rejected. Active Transform2D with legacy position, scale, or opacity keyframes MUST fail with INVALID_ARGUMENT on either transform or keyframe mutation. Legacy transform validation MUST remain unchanged.

#### Scenario: Test every numeric boundary
- **WHEN** each new numeric field is at an inclusive bound, outside that bound, or non-finite, including scale zero
- **THEN** valid boundaries succeed and invalid values fail with INVALID_ARGUMENT without mutation

#### Scenario: Reject incompatible animation
- **WHEN** Transform2D is assigned to an item with legacy transform keyframes or those keyframes are assigned to an item with Transform2D
- **THEN** the mutation fails atomically with INVALID_ARGUMENT

### Requirement: Transform2D transactional semantics
Transform2D edits MUST preserve existing lock, missing-item, revision-conflict, changed-ID, alias resolution, undo/redo, duplication, split, and persistence behavior.

#### Scenario: Create and transform by alias
- **WHEN** a valid batch creates a visual item and updates its Transform2D through the creation alias
- **THEN** both edits commit as one revision with resolved changed IDs and one undo step

#### Scenario: Roll back a failed batch
- **WHEN** any batch operation has invalid Transform2D, a missing item, a locked target, or a stale expected revision
- **THEN** the existing typed error is returned and no state, history, alias, or artifact is published

#### Scenario: Retain transform through lifecycle edits
- **WHEN** a transformed item is duplicated or split and the edits are undone, redone, and reopened
- **THEN** its complete active transform is preserved deterministically in each resulting state
