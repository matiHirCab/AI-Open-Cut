## ADDED Requirements

### Requirement: Canonical editor-core ownership
The editor core MUST maintain one documented canonical owner for the editor model, validation, persistence, migrations, asset lifecycle, timeline operations, drafts, scene evaluation and render planning, renderer process execution, and render artifact I/O, and outer applications SHALL delegate domain decisions to those owners.

#### Scenario: Resolve a domain responsibility
- **WHEN** a contributor needs to change a project, timeline, asset, draft, migration, or rendering rule
- **THEN** the ownership map identifies exactly one editor-core module responsible for that rule and no bridge or desktop implementation is required

#### Scenario: Reuse canonical validation
- **WHEN** headless, bridge, or desktop code submits an invalid editor operation
- **THEN** the typed failure originates from canonical editor-core validation rather than a parallel domain validator

### Requirement: Inward dependency direction
Editor-core modules MUST follow the documented inward dependency direction from orchestration and infrastructure adapters toward domain models and canonical rules, and repository architecture checks MUST reject forbidden outward or dependency-inverting imports.

#### Scenario: Detect an inverted dependency
- **WHEN** an editor-core domain or planning module imports transport, presentation, provider, environment-configuration, FFmpeg-process, or artifact-publication concerns contrary to the ownership map
- **THEN** an automated architecture check fails with the violated boundary

#### Scenario: Review a boundary exception
- **WHEN** a proposed change cannot follow an existing allowed dependency edge
- **THEN** repository review rules require an ADR update and boundary-test update before the new edge is accepted

### Requirement: Compatibility-preserving facade
The editor core SHALL preserve the existing public facade, serialized project, history, and draft representations, stable errors and warnings, revision semantics, reopen behavior, and preview/export behavior while responsibilities are extracted.

#### Scenario: Reopen existing state after extraction
- **WHEN** an existing compatible project with retained history and drafts is opened after the module extraction
- **THEN** it materializes the same state and revision without a schema migration or serialized-shape change

#### Scenario: Invoke an existing caller
- **WHEN** an existing headless or bridge caller uses an editor-core store or renderer operation
- **THEN** it receives the same public result shape, stable failure code, warnings, and committed behavior as before the extraction

#### Scenario: Render through an existing entry point
- **WHEN** frame preview, range preview, draft preview, or export is invoked with an existing fixture
- **THEN** the resulting artifact remains compatible with the established visual, audio, path-safety, and overwrite behavior

### Requirement: Replaceable persistence boundary
Storage locking, durable file replacement, transaction recovery, and managed-file operations MUST remain behind a narrow editor-core persistence interface whose fault outcomes can be supplied deterministically in tests without changing domain rules.

#### Scenario: Inject a persistence fault
- **WHEN** a deterministic test supplies a storage failure before or after the durable commit point
- **THEN** the canonical transaction behavior returns the same typed rejection or recovery warning required by project persistence semantics

#### Scenario: Use filesystem persistence
- **WHEN** the production editor core creates, opens, migrates, or mutates a project
- **THEN** the filesystem adapter performs locking and durable I/O without exposing filesystem details to timeline, draft, or validation rules

### Requirement: Separated render planning and execution
Scene evaluation and render planning MUST produce deterministic renderer-neutral or declarative planning data independently of FFmpeg process execution and artifact publication, and execution and artifact I/O MUST be accessed through narrow replaceable interfaces.

#### Scenario: Test a render plan deterministically
- **WHEN** a test evaluates a fixed project or future evaluated scene for a frame, range, or export request
- **THEN** it can assert the ordered inputs, composition/audio plan, timing, and output intent without starting FFmpeg or publishing an artifact

#### Scenario: Inject a renderer execution failure
- **WHEN** a process executor reports spawn, non-zero-exit, cancellation, or diagnostic-output failure
- **THEN** the renderer facade maps it to the existing structured error behavior without altering the deterministic render plan

#### Scenario: Publish a completed artifact
- **WHEN** process execution succeeds and artifact publication is requested
- **THEN** the artifact adapter applies existing temporary-file, collision, overwrite, cleanup, path, MIME, size, and warning behavior independently of plan construction

### Requirement: EvaluatedScene-compatible render seam
The render-planning boundary MUST provide a single inward seam through which issues #12 and #13 can later supply `EvaluatedScene` semantics to frame preview, range preview, draft preview, and export without coupling scene evaluation to FFmpeg execution or artifact storage.

#### Scenario: Introduce EvaluatedScene later
- **WHEN** the renderer-neutral evaluated representation is implemented
- **THEN** all render entry points can consume it at the planning boundary without changing persistence ownership or importing FFmpeg and artifact I/O into scene evaluation
