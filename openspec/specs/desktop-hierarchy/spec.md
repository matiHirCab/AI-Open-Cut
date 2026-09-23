# Desktop Hierarchy Specification

## Purpose
Define core-backed desktop project loading, scoped hierarchy inspection, shared selection and revisioned parent/z-index/history controls.
## Requirements
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

### Requirement: Structured vector and text inspection
Desktop MUST show selected shape geometry/paint/stroke, procedural-grid descriptor and text document/style/layout with the existing scoped identity. Root controls MUST expose rectangle/rounded-rectangle/ellipse dimensions, rounded radii, solid fill color, stroke color/width; grid spacing/angle/line color/width; and text content, font size/base color, selected run bold/italic/color, tracking, line height, bounds/fit and existing stroke/shadow layer fields. Font identity and unsupported geometry/paint fields MUST remain visible read-only. Component-local occurrences MUST remain entirely read-only. Controls MUST label units according to existing local-pixel, degree and text-layout contracts; they MUST NOT expose raw SVG, executable expressions, paths or network resources.

#### Scenario: I1 Inspect supported and read-only content
- **WHEN** selection moves among root shapes, grids, text and repeated component occurrences
- **THEN** values reflect the selected authoritative item and scope, only applicable root controls are editable, and unsupported values are preserved

### Requirement: Transactional vector and text inspector edits
Desktop MUST submit existing typed core operations with the displayed expected revision. Apply MUST preserve fields not edited, including geometry variant, gradients, font bindings, style layers, layout and document runs; explicit plain-content replacement MUST follow existing simple-text replacement semantics. Parse failures MUST not call core mutation; core MUST own all semantic validation and complexity limits. Success MUST reload the authoritative snapshot and reset the draft. Selection, refresh and history changes MUST discard stale drafts. Failures MUST preserve project/history and show parsing feedback or unchanged core code/message/retryability without automatic retry or speculative publication. Reset MUST restore displayed authoritative values without mutation.

#### Scenario: I2 Apply and preserve unrelated fields
- **WHEN** users edit a supported vector/grid property or text style/run property and apply
- **THEN** one core transaction changes only the intended fields, preserves other authored values and reloads the committed revision

#### Scenario: I3 Reject malformed invalid or stale edits
- **WHEN** parsing fails, core rejects non-finite/out-of-range values, the item is missing or locked, or the displayed revision is stale
- **THEN** no partial edit commits, the applicable failure remains visible and conflicts offer explicit refresh

#### Scenario: I4 Restore history and discard stale drafts
- **WHEN** an inspector edit is undone, redone or reopened, or the user changes selection, refreshes or resets a draft
- **THEN** displayed values match the applicable authoritative state and stale local input is never submitted against a different item or revision
