## ADDED Requirements

### Requirement: Archive-only OpenSpec merge readiness
The protected repository policy MUST reject every unarchived entry under `openspec/changes` and MUST emit no completion attestation until the directory contains only the canonical `archive` directory. The changes root and archive path MUST be ordinary directories; files, directories, symbolic links, malformed entries, and multiple concurrent entries outside `archive` MUST all fail closed before Moon launches. Active changes MAY exist during local authoring, but they MUST be completed, synchronized, verified, and archived before the protected merge-ready gate can succeed.

#### Scenario: Accept an archive-only repository
- **WHEN** `openspec/changes` and `openspec/changes/archive` are ordinary directories and no other direct entry exists
- **THEN** repository policy continues to the protected Moon task

#### Scenario: Reject an unarchived change
- **WHEN** any file, directory, or symbolic link other than `archive` exists directly under `openspec/changes`
- **THEN** preflight reports every unarchived entry, launches no Moon child, and emits no policy attestation

#### Scenario: Reject an invalid archive boundary
- **WHEN** the changes root or canonical archive path is missing, is not a directory, or is a symbolic link
- **THEN** preflight fails before Moon launch and emits no policy attestation
