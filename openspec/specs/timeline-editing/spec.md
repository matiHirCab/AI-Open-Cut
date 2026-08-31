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
