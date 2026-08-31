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
Committing a draft SHALL apply all draft operations as one project revision and one history entry and then remove the consumed draft file, while discarding SHALL remove only the draft without changing the project.

#### Scenario: Commit a valid draft
- **WHEN** a current draft is committed successfully
- **THEN** all candidate operations are published as one project revision and the consumed draft is removed

#### Scenario: Discard a draft
- **WHEN** a caller discards an existing draft
- **THEN** the draft is removed and the project state and revision remain unchanged
