## ADDED Requirements

### Requirement: Complete typed group workflows
Headless and MCP MUST expose add_group, item_set_parent, item_set_z_index and group_ungroup as standalone edits and timeline_batch_edit variants with the existing project/revision envelope and mutation results. Transport adapters MUST delegate graph validation, ungroup behavior, atomicity and persistence to editor-core. Published documentation MUST explain local-preserving promotion, root-time timing, flat ordering, limits, errors and discovery.

#### Scenario: Execute real standalone group workflow
- **WHEN** a client creates a group, reparents a visual, changes its z-index and ungroups through each supported transport
- **THEN** typed results and subsequent project reads expose the expected core state and undo/redo/reopen restore the expected history

#### Scenario: Execute real batch workflow and failures
- **WHEN** the same workflow uses creation aliases in one batch, or encounters malformed input, a missing reference, a locked affected track, a stale revision or a later failed operation
- **THEN** real headless and MCP calls exhibit the specified single-commit or full-rollback behavior and canonical errors without adapter-owned domain mutation logic
