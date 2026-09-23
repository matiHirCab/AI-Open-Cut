## ADDED Requirements

### Requirement: Atomic typed channel editing
The timeline MUST expose a typed replace-channels operation for a compatible item as a standalone edit and within `timeline_batch_edit`. Omission MUST preserve channels; an explicit empty list MUST clear them. The operation MUST use existing expected-revision, track-lock, changed-ID, undo/redo, and batch alias semantics, with one revision and history step per successful standalone edit or batch. Legacy `set_keyframes` MUST preserve its existing names, wire values, and meaning; conflicting legacy and new channels addressing the same effective property MUST be rejected instead of applying undocumented precedence.

#### Scenario: Create and animate by alias
- **WHEN** a batch creates an item with a result alias and a later operation replaces its channels through that alias
- **THEN** both edits commit as one revision and the result reports the resolved item ID

#### Scenario: Roll back invalid or stale edit
- **WHEN** a channel edit targets a missing or locked item, contains an invalid value, or uses a stale expected revision
- **THEN** the existing stable error is returned and no item, alias mapping, revision, or history entry is published

#### Scenario: Preserve existing simple edits
- **WHEN** an older client uses `set_keyframes` or a non-animation item edit
- **THEN** the operation retains its existing behavior and serialized result

#### Scenario: Reject conflicting later edits
- **WHEN** a caller sets a legacy position, scale, opacity, or volume keyframe that overlaps an active typed channel, or sets `transform2d` while visual typed channels exist
- **THEN** core rejects the complete edit with `INVALID_ARGUMENT` without changing state or history
