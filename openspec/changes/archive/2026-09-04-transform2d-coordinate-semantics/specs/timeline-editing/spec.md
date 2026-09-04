## ADDED Requirements

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
