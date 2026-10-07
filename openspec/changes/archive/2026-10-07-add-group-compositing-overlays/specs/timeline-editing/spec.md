## MODIFIED Requirements

### Requirement: Non-drawing group timeline nodes
Core MUST support GroupItem on overlay tracks with stable ID, integer-millisecond start/duration, common visual properties, and optional parent. Groups MUST default to identity Transform2D, visible, and zero zIndex; existing timing and ordering limits SHALL apply. Groups MUST accept static Transform2D, visibility, timing, move, reorder, duplication and schema37 approved clip/effects edits. Group legacy transform updates, keyframes, split, audio, and transition-endpoint use MUST fail with INVALID_ARGUMENT. Group duplication MUST copy only the node and its parent, without changing children. Existing item behavior SHALL remain compatible.


Groups MUST remain non-source/non-audio controller nodes. Only explicit approved schema37 clip/nonempty effects MUST create an evaluated drawable aggregate over descendants or bounded emitted particles; no controls MUST preserve the existing non-drawing path. Duplication MUST copy approved node controls without implicitly reparenting/cloning descendants; ungroup MUST retain local-preserving node deletion semantics and remove only its owner clip/effects, with no baked pixels or child effect duplication. Existing interval/lock/missing/stale/alias/revision/rollback/history requirements MUST remain.

#### Scenario: Create and edit a group
- **WHEN** a group is created on an unlocked overlay track and receives a valid static transform without explicit clip/nonempty effects
- **THEN** it persists with canonical ordering and produces no independent drawable or audio instruction

#### Scenario: Reject unsupported group edits
- **WHEN** a group is placed on a non-overlay track or receives keyframes, legacy transform, split, audio, or transition-endpoint edits
- **THEN** core returns INVALID_ARGUMENT without changing state or history

#### Scenario: Duplicate a node
- **WHEN** a group with descendants is duplicated
- **THEN** only a new group node is created with the source properties and parent and existing descendants retain their original parent

#### Scenario: Duplicate and ungroup controlled nodes reversibly
- **WHEN** a controlled group is duplicated or ungrouped through existing standalone/alias workflow
- **THEN** controls copy only with the duplicate node or disappear with the removed node, surviving child local properties stay unchanged and native undo/redo/reopen prove both generations
