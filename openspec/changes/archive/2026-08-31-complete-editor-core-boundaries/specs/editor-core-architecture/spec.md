## MODIFIED Requirements

### Requirement: Canonical editor-core ownership
The editor core MUST maintain one documented canonical owner for the editor model, validation, persistence, migrations, asset lifecycle including garbage collection, timeline operations, drafts, scene evaluation and render planning, renderer process execution, and render artifact I/O, and outer applications and facade modules SHALL delegate domain and infrastructure decisions to those owners.

#### Scenario: Resolve a domain responsibility
- **WHEN** a contributor needs to change a project, timeline, asset, garbage-collection, draft, migration, or rendering rule
- **THEN** the ownership map identifies exactly one editor-core module responsible for that rule and no bridge, desktop, or facade implementation is required

#### Scenario: Reuse canonical validation
- **WHEN** headless, bridge, or desktop code submits an invalid editor operation
- **THEN** the typed failure originates from canonical editor-core validation rather than a parallel domain validator

#### Scenario: Collect managed assets
- **WHEN** a committed project no longer retains a managed asset through current state, undo/redo history, or durable drafts
- **THEN** the assets owner decides collection and performs managed-file operations only through the persistence storage boundary while preserving existing warnings

### Requirement: Inward dependency direction
Editor-core modules MUST follow the complete documented allowed-dependency matrix from orchestration and infrastructure adapters toward domain models and canonical rules, and repository architecture checks MUST reject every undocumented internal edge as well as forbidden outward imports and duplicated owner implementations.

#### Scenario: Detect an inverted dependency
- **WHEN** any editor-core owner imports another internal owner outside its allowed matrix, or a domain/planning owner imports transport, presentation, provider, environment-configuration, FFmpeg-process, artifact-publication, or direct managed-file concerns contrary to the ownership map
- **THEN** an automated architecture check fails with the owner and violated boundary

#### Scenario: Detect responsibility duplication
- **WHEN** `renderer` reimplements scene input enumeration or command execution, or `store` reimplements asset garbage collection
- **THEN** an automated architecture check fails even if the duplicated code compiles

#### Scenario: Review a boundary exception
- **WHEN** a proposed change cannot follow an existing allowed dependency edge
- **THEN** repository review rules require an ADR update and boundary-test matrix update before the new edge is accepted

### Requirement: Replaceable persistence boundary
Storage locking, durable file replacement, transaction recovery, managed-file enumeration, and managed-file deletion MUST remain behind a narrow editor-core persistence interface whose fault outcomes can be supplied deterministically in tests without changing domain or garbage-collection rules.

#### Scenario: Inject a persistence fault
- **WHEN** a deterministic test supplies a storage failure before or after the durable commit point or during garbage collection
- **THEN** the canonical transaction behavior returns the same typed rejection, recovery warning, or `ASSET_GC_FAILED` warning required by project persistence semantics

#### Scenario: Use filesystem persistence
- **WHEN** the production editor core creates, opens, migrates, mutates, or garbage-collects a project
- **THEN** the filesystem adapter performs locking and durable managed-file I/O without exposing filesystem details to timeline, draft, validation, asset-policy, or store orchestration rules

### Requirement: Separated render planning and execution
Scene evaluation and render planning MUST produce deterministic declarative data containing ordered logical inputs, resource requests, dimensions, timing, composition/audio instructions, and output intent independently of FFmpeg process execution and artifact publication, and execution and artifact I/O MUST be accessed through narrow facade-injectable interfaces.

#### Scenario: Test a render plan deterministically
- **WHEN** a test evaluates a fixed project or future evaluated scene for a frame, range, or export request
- **THEN** it can assert ordered inputs, resource requests, composition/audio plan, dimensions, timing, and output intent without starting FFmpeg, touching the filesystem, or publishing an artifact

#### Scenario: Inject a renderer execution failure
- **WHEN** a renderer facade is constructed internally with a process executor that reports readiness, probe, spawn, non-zero-exit, cancellation, or diagnostic-output failure
- **THEN** the public renderer operation maps it to the existing structured error behavior without altering the deterministic render plan or starting a real process

#### Scenario: Publish a completed artifact
- **WHEN** process execution succeeds and the renderer facade uses an injected artifact adapter for publication
- **THEN** the adapter applies or deterministically fails existing temporary-file, collision, overwrite, cleanup, path, MIME, size, and warning behavior independently of plan construction

### Requirement: EvaluatedScene-compatible render seam
The render-planning boundary MUST provide a single inward seam through which issues #12 and #13 can later supply `EvaluatedScene` semantics to frame preview, range preview, draft preview, and export, and the renderer facade MUST NOT inspect project tracks/items or reconstruct logical input/resource ordering outside that seam.

#### Scenario: Introduce EvaluatedScene later
- **WHEN** the renderer-neutral evaluated representation is implemented
- **THEN** all render entry points can substitute it at the planning boundary without changing renderer orchestration, persistence ownership, process execution, or artifact storage
