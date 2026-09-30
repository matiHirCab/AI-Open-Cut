## ADDED Requirements

### Requirement: Transactional inherited timing edits
Typed AddGroup and AddComponentInstance requests MUST accept optional staggerMs; typed update_item on a group or component instance and component_instance_update MUST accept optional staggerMs; typed repeater create/replace requests MUST accept optional timeOffsetMs in the descriptor; set_animation_channels MUST accept active parent channels. These edits MUST work standalone and in ordered timeline_batch_edit, including earlier creation aliases and existing operation bounds. Omission MUST preserve values on updates with patch semantics; explicit zero MUST clear an offset. Generic update_item MUST reject staggerMs on other item kinds. Group and component-instance duplication MUST preserve staggerMs unless an operation explicitly supplies a replacement. Successful batches MUST commit once and undo/redo MUST restore exact typed values. Missing references, locked tracks, invalid values, unsupported targets, stale expected revisions and trailing invalid operations MUST retain stable error codes and retryability, with no partial revision, history, aliases or artifacts.

#### Scenario: Create and animate by alias
- **WHEN** one batch creates a group or component instance, sets its stagger and parent channels by alias, and creates a repeater with a copy offset
- **THEN** references resolve in order, one revision and undo step commit, and undo/redo restores exact timing and channels

#### Scenario: Roll back failed timing work
- **WHEN** a later timing edit is invalid, names a missing source, or the expected revision is stale
- **THEN** established INVALID_ARGUMENT, ITEM_NOT_FOUND or retryable REVISION_CONFLICT applies and project/history bytes remain unchanged


