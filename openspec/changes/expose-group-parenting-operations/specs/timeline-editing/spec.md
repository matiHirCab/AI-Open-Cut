## ADDED Requirements

### Requirement: Atomic local-preserving ungroup
Core MUST expose `group_ungroup` with required `groupId`, remove only that group, and assign its immediate children the removed group's parent or null. Children MUST retain local transform, visibility, root-millisecond timing, zIndex, track and relative array ordering; nested descendants MUST retain their direct parents. Ordinals MUST be normalized after deletion. Existing direct group deletion restrictions SHALL remain unchanged.

#### Scenario: Ungroup root and nested groups
- **WHEN** a root or parented group containing visual items and nested groups across tracks is ungrouped
- **THEN** only that group disappears, immediate children inherit its parent, deeper descendants retain their parents, and all surviving local properties remain unchanged apart from required ordinal normalization

#### Scenario: Ungroup an empty group
- **WHEN** a valid unlocked group has no children
- **THEN** ungroup removes it in one revision and undo step

#### Scenario: Preserve evaluated semantics
- **WHEN** a group with non-identity transform, opacity, visibility and interval contributions is ungrouped
- **THEN** preview and export evaluate the same result as explicitly reparenting its immediate children to its parent and deleting it, without world-transform compensation or changes to renderer algorithms

### Requirement: Ungroup failures preserve the complete transaction
Ungroup MUST require the group's track and every immediate child's track unlocked, including hidden/inactive children. Read-only ancestors and deeper descendants MUST NOT require unlocked tracks. Missing group MUST return ITEM_NOT_FOUND, a non-group target INVALID_ARGUMENT, affected locked tracks TRACK_LOCKED, and stale revision retryable REVISION_CONFLICT under existing revision precedence. Core MUST retain existing finite-value, graph, project and batch complexity limits. Failure MUST publish no state, history, aliases or managed artifacts.

#### Scenario: Reject missing or invalid targets
- **WHEN** ungroup targets an absent ID or a non-group timeline item
- **THEN** it returns the specified stable typed error without changing current or retained state

#### Scenario: Respect exactly the affected locks
- **WHEN** the group or any immediate child is on a locked track
- **THEN** ungroup fails atomically with TRACK_LOCKED, while a locked track containing only a read-only ancestor or deeper descendant does not independently block the operation

#### Scenario: Preserve bounded validation and revision checks
- **WHEN** a group workflow has a stale revision, invalid finite transform values, a parent cycle, or exceeds existing node/depth/batch limits
- **THEN** existing typed failures and inclusive boundaries remain enforced in the owning layer with no publication

### Requirement: Ungroup batch aliases and reversible results
Ungroup MUST work standalone and within the existing 1-to-100 operation ordered atomic batch and resolve groupId aliases to earlier creations. It MUST NOT accept a resultAlias because it creates no ID. Creation alias mappings MUST retain their existing reporting semantics even if the created group is subsequently removed; a later item reference to that removed ID MUST fail and roll back the batch. Results MUST report the removed ID first, immediate children in pre-edit traversal order next, and additional ordinal-changed IDs through existing ordering conventions without duplicates. Success MUST preserve deterministic undo/redo, reopen, media and provenance behavior using schema 10.

#### Scenario: Create parent order and ungroup in one batch
- **WHEN** a batch creates a group and children with aliases, reparents and sets z-index through those aliases, then ungroups by the group alias
- **THEN** the final graph commits once, returns deterministic changed IDs and creation mappings, and undo restores the entire pre-batch state

#### Scenario: Roll back after ungroup
- **WHEN** a later operation fails, including a reference to the removed group's creation alias, or ungroup uses an unresolved or forward alias
- **THEN** the entire batch fails through existing alias/reference errors without publishing partial state, history or aliases

#### Scenario: Restore exact history on reopen
- **WHEN** a successful standalone or batched ungroup is undone, redone and reopened
- **THEN** each state preserves exact expected groups, parent links, ordering and local properties with existing revision semantics and unchanged media/provenance ownership

#### Scenario: Reject explicit null result aliases before batch execution
- **WHEN** standalone or batch group_ungroup input contains resultAlias with null, a string or any other value
- **THEN** decoding rejects the request before mutation, headless returns non-retryable INVALID_ARGUMENT, and even a preceding valid batch edit leaves revision, project and history unchanged; omitted aliases and other operations retain existing behavior
