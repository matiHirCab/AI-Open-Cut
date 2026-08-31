# Edit Drafts Specification

## Purpose

Define isolated, revision-aware composition of proposed edits before they are committed to the durable project timeline.

## Requirements

### Requirement: Revision-bound draft creation
The editor core SHALL create a draft containing between one and 100 valid ordered edits against the current project revision, retain its base revision and optional validated label, and SHALL reject creation once the project already retains 100 draft files.

#### Scenario: Create a draft
- **WHEN** a caller submits valid operations at the current project revision
- **THEN** the core returns a stable draft identifier without modifying the persisted project timeline

#### Scenario: Reject excess retained drafts
- **WHEN** a project already contains 100 retained drafts and a caller creates another
- **THEN** creation fails with `DRAFT_LIMIT_REACHED` without modifying the project or existing drafts

### Requirement: Isolated draft updates
Draft updates MUST validate their complete operation set against the draft base state and SHALL remain isolated from project history until commit.

#### Scenario: Update a draft
- **WHEN** a caller replaces a draft's operations while its base revision remains current
- **THEN** subsequent draft reads and previews use the updated operations while the project revision is unchanged

### Requirement: Draft state and preview
The system SHALL materialize a draft state and render a draft preview by applying the draft operations to its base project without publishing that candidate state.

#### Scenario: Preview a draft
- **WHEN** a caller requests a frame from a valid draft
- **THEN** the preview reflects the candidate edits and the durable project remains unchanged

### Requirement: Explicit stale-draft handling
Updating or committing a draft MUST reject a base revision that no longer matches the current project, while rebase SHALL explicitly move the draft to a caller-supplied current revision.

#### Scenario: Reject a stale draft commit
- **WHEN** the project revision advanced after draft creation and the caller attempts to commit the old base
- **THEN** commit fails with `REVISION_CONFLICT` and the draft remains available for rebase or discard

#### Scenario: Rebase a draft
- **WHEN** a caller rebases a draft against the current revision and its operations remain valid
- **THEN** the draft records the new base revision without modifying the project

### Requirement: Single-revision commit and independent discard
Committing a draft SHALL transactionally apply all draft operations as one project revision and one history entry and consume the draft as part of the committed outcome, while discarding SHALL remove only the draft without changing the project. A repeated commit attempt for the same consumed draft MUST NOT apply its operations as another revision. If the project and history generation commits but draft cleanup remains pending, the commit SHALL return the committed revision with stable `DRAFT_CLEANUP_FAILED` and `PERSISTENCE_RECOVERY_PENDING` warnings rather than report rejection.

#### Scenario: Commit a valid draft
- **WHEN** a current draft is committed successfully
- **THEN** all candidate operations are published as one project revision, one history entry is retained, and the consumed draft is removed

#### Scenario: Retry an interrupted draft commit
- **WHEN** a caller retries a draft commit after termination or an I/O failure at any persistence phase
- **THEN** recovery either completes the single committed revision or preserves the uncommitted draft, and the draft operations are never applied twice

#### Scenario: Report committed draft cleanup failure
- **WHEN** project and history publication commits but the consumed draft cannot yet be removed
- **THEN** the commit returns the committed revision with `DRAFT_CLEANUP_FAILED` and `PERSISTENCE_RECOVERY_PENDING`, and later project open deterministically completes cleanup

#### Scenario: Discard a draft
- **WHEN** a caller discards an existing draft
- **THEN** the draft is removed and the project state and revision remain unchanged
