## ADDED Requirements

### Requirement: Explicit core-backed desktop project session
Desktop MUST accept an explicit local project-store path and project ID at startup, load through EditorCore and display the authoritative revision. With no project selected it MUST show an empty state without creating or modifying a project. Load and refresh failures MUST expose the core error code and message without displaying an invalid candidate as loaded. Desktop MUST delegate path safety, migration, validation and persistence to core and MUST NOT introduce a new project format or transport operation.

#### Scenario: Load and refresh an existing project
- **WHEN** desktop starts with a valid store/project selection or the user refreshes after an external edit
- **THEN** hierarchy, timeline summary and inspector derive from the same authoritative core snapshot and revision

#### Scenario: Empty or invalid selection
- **WHEN** startup has no project selection or core rejects a missing, unsafe, malformed or future-schema project
- **THEN** desktop shows an empty state or the returned failure respectively and never substitutes a fabricated project

### Requirement: Scoped hierarchy and selection
Desktop MUST display expandable root group relationships and component-instance contents, including hidden items, with stable selection identities containing composition scope and complete instance path. Each item MUST expose its kind, ID, owning track, parent, local start/duration and z-index where applicable. Component-local rows MUST be explicitly read-only. Root timeline summaries MUST share selection with the hierarchy and inspector. Tree indentation MUST describe parentage without claiming to be paint order: the UI MUST explain track-first stacking, within-track z-index and stable item order, and contiguous component-instance stacking. Expansion MUST be lazy and bounded to at most 4096 displayed rows, with an explicit limit message rather than unbounded repeated-DAG expansion; this presentation limit MUST NOT reject an otherwise valid project.

#### Scenario: Inspect repeated scoped content
- **WHEN** three instances share one definition with equal local child IDs and different overrides
- **THEN** expanding and selecting each occurrence identifies the correct instance path and its stored overrides without changing the definition or conflating selections

#### Scenario: Inspect cross-track parenting and bounded expansion
- **WHEN** a group owns children across tracks or nested shared instances exceed the displayed-row limit
- **THEN** the hierarchy retains parent and track identities, shows the documented ordering distinction and stops expansion with a visible limit message

### Requirement: Revisioned parent and z-index controls
Desktop MUST offer root visual-item parent selection, including detach, and signed integer z-index editing through existing ItemSetParent and ItemSetZIndex operations with the displayed expected revision. Parent choices MUST identify root groups; core MUST remain responsible for reference, cycle, lock, scope and domain validation. Input parsing MUST reject text that cannot represent an i32 without calling a mutation. Reparenting MUST preserve local properties, not world coordinates. Component-local rows and unsupported item kinds MUST NOT expose these mutation controls. Successful changes MUST reload the authoritative snapshot and preserve selection if its identity survives.

#### Scenario: Edit and detach a root instance
- **WHEN** the user changes a root instance's parent or z-index and then detaches it
- **THEN** existing core operations commit the requested values with their standard revisions and local-preserving semantics

#### Scenario: Reject invalid edits and conflicts
- **WHEN** input is not an i32, or core reports a missing reference, cycle, locked track or stale revision
- **THEN** the UI reports parsing failure or the unchanged core code/message/retryability, publishes no speculative state and does not silently retry a conflicting edit

### Requirement: Desktop history and refresh consistency
Desktop MUST provide undo, redo and refresh through existing core APIs. Undo and redo MUST use the displayed revision, reload on success and clear selection only when the selected occurrence no longer exists. External edits MUST become visible on explicit refresh; a conflict MUST leave the attempted edit unapplied and offer refresh. Reopening MUST reconstruct hierarchy and persisted values without requiring saved UI state.

#### Scenario: Undo redo and reopen an inspector edit
- **WHEN** a parent or z-index edit is undone, redone and the project is reopened
- **THEN** hierarchy and inspector match each authoritative snapshot and exact restored values

#### Scenario: Removed selection or external edit
- **WHEN** refresh or history changes remove a selected occurrence or advance the project revision externally
- **THEN** stale selection is cleared as needed and the next user action uses the refreshed revision without automatic overwrite
