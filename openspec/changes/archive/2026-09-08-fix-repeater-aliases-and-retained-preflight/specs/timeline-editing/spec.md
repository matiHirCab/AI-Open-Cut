## ADDED Requirements

### Requirement: Aliased repeater descriptor replacement
Batch update_item MUST resolve itemId first and optional repeater.source.id second using existing earlier-creation aliases before semantic validation. source.scope MUST remain literal. Replacement MUST preserve the existing complete-descriptor semantics, protocol 1, schema 17, changed-ID behavior and one-revision/one-undo-step transaction. Missing or forward batch aliases MUST retain VALIDATION_FAILED; literal missing sources MUST retain ITEM_NOT_FOUND; stale revisions MUST retain retryable REVISION_CONFLICT. Failures MUST roll back project/history files and alias results. Drafts MUST preserve their existing literal-ID operation format and behavior without adding alias declarations or substitution.

#### Scenario: Retarget with two aliases
- **WHEN** a batch creates a source and repeater and updates the repeater using aliases for both itemId and source.id
- **THEN** the persisted descriptor contains the resolved source ID, one revision commits, and undo/redo and reopen preserve the replacement

#### Scenario: Roll back replacement failures
- **WHEN** a source alias is missing or forward-referenced, both item and source aliases are invalid, or a later operation fails after a valid retarget
- **THEN** existing alias error precedence is preserved and the entire batch publishes no state, history or aliases

#### Scenario: Preserve draft compatibility and revision conflicts
- **WHEN** a draft replaces a repeater descriptor using literal IDs obtained from a committed batch, or a batch has a stale expected revision
- **THEN** draft materialization preserves the supplied canonical IDs without mutating current state, missing draft references retain their existing errors, and the stale batch returns REVISION_CONFLICT without changes
