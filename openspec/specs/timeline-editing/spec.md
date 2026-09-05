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

### Requirement: Persisted canonical stacking values
Every timeline item MUST persist zIndex as an integer in [-2147483648,2147483647] and stackOrder as a u32 equal to its zero-based position in its owning track array. New items MUST default zIndex to zero. Mutations MUST synchronize ordinals while preserving existing insertion behavior; split, duplicate, and move MUST retain source zIndex. Existing scene complexity limits MUST apply before ordering work.

#### Scenario: Preserve ordering through lifecycle edits
- **WHEN** items are created, split, duplicated, moved, deleted, or generated by caption or speech workflows
- **THEN** each affected array has consecutive ordinals matching its array positions, retained items preserve zIndex, and existing timing/insertion behavior remains intact

#### Scenario: Reject malformed persisted ordering
- **WHEN** a schema-9 record omits ordering fields or has fractional, non-finite, out-of-range, duplicate, or array-inconsistent ordering values
- **THEN** core rejects the state with existing typed validation behavior before publication or rendering

### Requirement: Explicit item and track ordering operations
Core MUST expose item_set_z_index with itemId and zIndex, item_reorder with itemId and index, and track_reorder with trackId and index. Reorder index MUST be an integer final zero-based position in the current owning collection, between zero and collection length minus one. Item reorder MUST preserve track, timing, and zIndex; track reorder MUST reuse existing indexed track-update semantics. Z-index edits MUST accept visual media, text, solid, rectangle, and caption items and reject audio-only media and transition targets with INVALID_ARGUMENT. Item reorder MUST accept all item kinds.

#### Scenario: Reorder equal z-index items
- **WHEN** a valid item reorder moves an item to the first or last position
- **THEN** the array and ordinals reflect that position and visual order changes only according to the canonical track/z-index/stack-order comparator

#### Scenario: Set signed z-index boundaries
- **WHEN** a supported visual receives zero, a negative z-index, or either i32 bound
- **THEN** the edit succeeds without changing array position, timing, or transform

#### Scenario: Match legacy track reorder
- **WHEN** equivalent track_reorder and existing indexed update_track requests execute against equal initial states
- **THEN** resulting ordering, revision, errors, and history semantics agree

#### Scenario: Reject invalid targets and values
- **WHEN** an operation names a missing item or track, targets a locked track, supplies an invalid index or z-index, or sets z-index on an unsupported kind
- **THEN** it fails with ITEM_NOT_FOUND, TRACK_NOT_FOUND, TRACK_LOCKED, VALIDATION_FAILED for an out-of-range collection index, or INVALID_ARGUMENT for malformed/new z-index input respectively, preserving existing transport decoding errors for malformed envelopes and leaving state unchanged

### Requirement: Ordering mutations are transactional and alias-aware
New ordering operations MUST work standalone and in timeline_batch_edit, resolve creation aliases using existing rules, honor optimistic revisions, and commit once or roll back completely. Responses MUST preserve existing primary changed IDs and additionally include each item with a changed ordinal, deduplicated in deterministic traversal order. Successful same-position/same-value edits MUST follow existing successful mutation revision/history behavior.

#### Scenario: Create and reorder by alias
- **WHEN** a valid batch creates a track and visual items then sets z-index and reorders them by creation aliases
- **THEN** all edits commit as one revision and undo step, aliases identify the created entities, and changed IDs include ordinal changes

#### Scenario: Roll back a failing batch
- **WHEN** a later ordering operation fails validation or reference/lock checks, or the request has a stale expected revision
- **THEN** no state, history, aliases, or artifacts are published and stale revisions return retryable REVISION_CONFLICT

#### Scenario: Undo redo and reopen stacking
- **WHEN** successful ordering edits are undone, redone, and reopened
- **THEN** exact arrays, z-index, and ordinals are restored deterministically with existing revision semantics

### Requirement: Non-drawing group timeline nodes
Core MUST support GroupItem on overlay tracks with stable ID, integer-millisecond start/duration, common visual properties, and optional parent. Groups MUST default to identity Transform2D, visible, and zero zIndex; existing timing and ordering limits SHALL apply. Groups MUST accept static Transform2D, visibility, timing, move, reorder, and duplication edits. Group legacy transform updates, keyframes, split, audio, and transition-endpoint use MUST fail with INVALID_ARGUMENT. Group duplication MUST copy only the node and its parent, without changing children. Existing item behavior SHALL remain compatible.

#### Scenario: Create and edit a group
- **WHEN** a group is created on an unlocked overlay track and receives a valid static transform
- **THEN** it persists with canonical ordering and produces no independent drawable or audio instruction

#### Scenario: Reject unsupported group edits
- **WHEN** a group is placed on a non-overlay track or receives keyframes, legacy transform, split, audio, or transition-endpoint edits
- **THEN** core returns INVALID_ARGUMENT without changing state or history

#### Scenario: Duplicate a node
- **WHEN** a group with descendants is duplicated
- **THEN** only a new group node is created with the source properties and parent and existing descendants retain their original parent

### Requirement: Scoped bounded parent graph
Visual media, text, solids, rectangles, captions, and groups MUST support optional parent {scope,id}, where scope MUST be root in this milestone and id MUST name a group in the same project composition. Tracks SHALL NOT define composition scope. Audio-only and transition items MUST remain unparented. Core MUST reject missing parents with ITEM_NOT_FOUND and non-group targets, cross-scope references, self/indirect cycles, invalid values, and ancestor paths exceeding 32 edges with INVALID_ARGUMENT. A root item has depth zero. Graph validation MUST include hidden and inactive nodes, reject duplicate IDs before lookup normalization, apply existing project limits plus at most 4096 visual/group nodes in root, and validate before publication or render work.

#### Scenario: Resolve across tracks
- **WHEN** a visual item references a group on a different root-composition track
- **THEN** the valid reference resolves without changing either item's flat stacking or track

#### Scenario: Reject missing and invalid references
- **WHEN** parent assignment references an absent ID, a non-group, another scope, or creates a direct or indirect cycle including hidden nodes
- **THEN** core reports the specified typed failure and publishes nothing

#### Scenario: Enforce exact depth and count boundaries
- **WHEN** a graph has 32 versus 33 ancestor edges or 4096 versus 4097 visual/group nodes
- **THEN** the inclusive boundary is accepted and overflow fails with INVALID_ARGUMENT before traversal expansion or rendering

### Requirement: Transactional parenting lifecycle
Core MUST expose add_group with trackId/startMs/durationMs and optional complete transform2d and parent, and item_set_parent with itemId and required parent object or null. Null SHALL detach without changing local properties; reparenting SHALL preserve local rather than world transforms. Both operations MUST work standalone and in ordered timeline_batch_edit with aliases for created group/item IDs, existing revision and lock checks, one commit/undo step, and complete rollback. Parenting SHALL require the target item's track unlocked; reading a parent on a locked track SHALL remain legal. Deleting a referenced group MUST fail with INVALID_ARGUMENT. Track deletion MUST retain the existing empty-track-only rule and VALIDATION_FAILED for nonempty tracks. Callers MUST detach/reparent surviving children, delete items, then delete the empty track. Supported moves, splits, and duplicates of existing visual items MUST preserve their parent.

#### Scenario: Create and parent by alias
- **WHEN** a batch creates a group and visual item then assigns the group alias as parent to the item alias
- **THEN** the resolved graph commits once with existing deterministic changed-ID and alias response conventions and undo/redo restores the complete graph

#### Scenario: Preserve failures atomically
- **WHEN** a stale revision, locked target, missing item, invalid parent, or later failed batch operation occurs
- **THEN** existing REVISION_CONFLICT, TRACK_LOCKED, ITEM_NOT_FOUND, or specified invalid-input behavior leaves current state, history, files, and aliases unpublished

#### Scenario: Delete without dangling references
- **WHEN** a referenced group is deleted directly or a nonempty track is deleted
- **THEN** group deletion fails with INVALID_ARGUMENT until children are detached or reparented, and nonempty-track deletion retains VALIDATION_FAILED until its items are deleted

#### Scenario: Preserve local properties through lifecycle
- **WHEN** a child is reparented, detached, moved, duplicated, or split using supported existing operations and the project is reopened
- **THEN** local values and retained parent references remain deterministic and no automatic world-transform compensation occurs

### Requirement: Group transition endpoints fail closed
Core MUST reject either transition endpoint resolving to a group with INVALID_ARGUMENT in current state, retained history, drafts, mutations and direct renderer input, including hidden records. Existing missing-endpoint behavior SHALL remain unchanged. Invalid persisted endpoints MUST NOT be repaired or published.

#### Scenario: Reject persisted group endpoints atomically
- **WHEN** current state or an undo/redo snapshot contains a visible or hidden transition referencing a group from either endpoint
- **THEN** loading fails with INVALID_ARGUMENT and leaves authoritative files unchanged
