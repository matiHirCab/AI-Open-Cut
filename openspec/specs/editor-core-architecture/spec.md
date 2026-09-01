# editor-core-architecture Specification

## Purpose
TBD - created by archiving change establish-editor-core-boundaries. Update Purpose after archive.
## Requirements
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
Editor-core modules MUST follow the complete documented allowed-dependency matrix from orchestration and infrastructure adapters toward domain models and canonical rules, and repository architecture checks MUST reject every undocumented internal edge expressed through direct, grouped, nested, aliased, relative, qualified, or multiline Rust imports and paths in every production item across inline and standard out-of-line modules, regardless of its position around test-only items, as well as forbidden outward imports and duplicated owner implementations. Root-reaching relative paths, module- or block-local `extern crate` and type aliases, nested conditional attributes, and direct filesystem method calls including symlink inspection MUST receive the same enforcement as their canonical forms. Unsupported custom module paths MUST fail architecture analysis explicitly rather than leave production code unexamined.

#### Scenario: Detect an inverted dependency
- **WHEN** any editor-core owner or its out-of-line submodule imports or references another internal owner outside its allowed matrix using a crate-root path, a root-reaching relative path, a top-level or nested use group, a module- or block-local alias chain, or another supported parsed Rust path form in any production item
- **THEN** an automated architecture check fails with the owner, source file, and violated boundary

#### Scenario: Resolve aliases through module ancestry
- **WHEN** a production item reaches a crate-root or external path through lexical aliases, aliases inherited through `super`, or multiple alias steps
- **THEN** the architecture check applies the same dependency and responsibility rules to the canonical path

#### Scenario: Resolve every legal root alias
- **WHEN** a production item reaches a crate owner or external filesystem API through leading `self` and `super` segments, a module- or block-local `extern crate self` alias, an external-crate alias, a path-based type alias, or a chained lexical/module alias
- **THEN** the architecture check resolves the exact canonical path and applies the existing dependency and responsibility rules

#### Scenario: Resolve block-local aliases
- **WHEN** a production block declares `use`, `extern crate`, or a path-based type alias that reaches an internal owner or external filesystem API
- **THEN** the architecture check resolves the declaration in that lexical scope and applies the same matrix and responsibility rules as for a module-level alias

#### Scenario: Reject an unexamined custom module path
- **WHEN** an editor-core owner declares an out-of-line production module using `#[path]`
- **THEN** architecture analysis fails with the owner and source file instead of silently skipping the module

#### Scenario: Reject a nested custom module path
- **WHEN** a production-reachable nested `cfg_attr` can select a custom `#[path]` module
- **THEN** architecture analysis fails with the owner and source file instead of analyzing a different standard module file

#### Scenario: Exclude test-only dependencies without truncating production
- **WHEN** an owner contains a test-only item and additional production items before or after it
- **THEN** the architecture check excludes only the test-only subtree and still enforces every production dependency in every analyzed source file

#### Scenario: Exclude only test code
- **WHEN** an owner contains a standalone `#[test]` item and production items before or after it
- **THEN** the architecture check excludes the test item and enforces every sibling production dependency

#### Scenario: Exclude test-only block aliases
- **WHEN** a production block contains an alias item that is disabled outside tests and additional production code
- **THEN** the architecture check excludes that alias item without hiding or changing the resolution of production siblings

#### Scenario: Detect direct filesystem methods
- **WHEN** an asset, store, renderer, or render-planning owner performs filesystem I/O or symlink inspection through a native `Path` method, UFCS call, path/type alias, or aliased `std::fs` path instead of its authorized adapter
- **THEN** the architecture check fails even when the source does not spell `std::fs` or `Path` directly

#### Scenario: Permit adapter path operations
- **WHEN** an owner delegates path inspection or canonicalization through its private persistence or artifact adapter
- **THEN** the architecture check accepts the operation without treating the adapter call as native filesystem I/O

#### Scenario: Detect responsibility duplication
- **WHEN** `renderer` reimplements scene input enumeration through field access or structured patterns, `assets` performs direct filesystem I/O, or `store` reimplements asset garbage collection under any function name
- **THEN** an automated architecture check fails even if the duplicated code compiles or uses aliases

#### Scenario: Review a boundary exception
- **WHEN** a proposed change cannot follow an existing allowed dependency edge
- **THEN** repository review rules require an ADR update and boundary-test matrix update before the new edge is accepted

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
Storage locking, project and draft directory operations, persisted reads and durable replacement, transaction recovery, managed-file enumeration, and managed-file deletion MUST remain behind one narrow editor-core persistence interface selected by the editor facade, whose real I/O outcomes can be supplied deterministically in tests without changing domain or garbage-collection rules. Project discovery and lookup MUST NOT follow a linked directory or accept a canonical project path outside the configured project root, and lookup MUST establish canonical containment before probing project markers or performing project operations.

#### Scenario: Inject a persistence fault
- **WHEN** a deterministic facade test supplies a storage failure during locking, directory creation or listing, persisted reading, durable replacement, recovery, draft cleanup, or garbage collection before or after the durable commit point
- **THEN** the canonical operation returns the same typed rejection, recovery warning, draft-cleanup warning, or `ASSET_GC_FAILED` warning required by existing project persistence semantics

#### Scenario: Use filesystem persistence
- **WHEN** the production editor core creates, lists, opens, migrates, mutates, drafts, recovers, or garbage-collects a project
- **THEN** the filesystem adapter performs locking and durable managed-file I/O without exposing filesystem details or concrete storage selection to timeline, draft, validation, asset-policy, or store orchestration rules

#### Scenario: Ignore a linked project entry
- **WHEN** project enumeration encounters a symbolic link or equivalent linked entry under the configured project root
- **THEN** it does not expose, open, recover, or garbage-collect the linked target as a project

#### Scenario: Reject a canonical project escape
- **WHEN** a syntactically valid project ID canonicalizes outside the configured project root, including when the external target contains no project or transaction marker
- **THEN** the operation fails with `PATH_NOT_ALLOWED` before any marker probe, lock, persisted read, recovery, or garbage collection

#### Scenario: Preserve an ordinary missing project
- **WHEN** a syntactically valid project ID identifies no directory inside the configured project root
- **THEN** lookup fails with the existing `PROJECT_NOT_FOUND` code without changing public error semantics

#### Scenario: Preserve ordinary project behavior
- **WHEN** a real project directory remains inside the configured root
- **THEN** create, list, open, recovery, drafts, history, managed assets, and garbage collection retain their existing results, errors, and warnings through the selected storage adapter

### Requirement: Separated render planning and execution
Scene evaluation and render planning MUST produce deterministic declarative data containing ordered logical inputs, resource requests, dimensions, timing, composition/audio instructions, and output intent independently of FFmpeg process execution and artifact publication, and execution and complete artifact I/O MUST be accessed through narrow facade-injectable interfaces. Complete artifact I/O includes workspace lifetime, temporary paths, resource and filter reads and writes, path inspection, publication, cleanup, and metadata.

#### Scenario: Test a render plan deterministically
- **WHEN** a test evaluates a fixed project or future evaluated scene for a frame, range, or export request
- **THEN** it can assert ordered inputs, resource requests, composition/audio plan, dimensions, timing, and output intent without starting FFmpeg, touching the filesystem, or publishing an artifact

#### Scenario: Inject a renderer execution failure
- **WHEN** a renderer facade is constructed internally with a process executor that reports readiness, probe, spawn, non-zero-exit, cancellation, or diagnostic-output failure
- **THEN** the public renderer operation maps it to the existing structured error behavior without altering the deterministic render plan or starting a real process

#### Scenario: Prepare and publish through an artifact adapter
- **WHEN** a renderer facade is constructed internally with an artifact adapter for a frame, range, or export operation
- **THEN** the adapter performs or deterministically fails workspace creation, temporary-path allocation, resource and filter I/O, path inspection, publication, metadata, and cleanup while preserving existing stages, collision and overwrite rules, MIME, size, warnings, and best-effort cleanup behavior

### Requirement: EvaluatedScene-compatible render seam
The render-planning boundary MUST provide a single inward seam through which issues #12 and #13 can later supply `EvaluatedScene` semantics to frame preview, range preview, draft preview, and export, and the renderer facade MUST NOT inspect project tracks/items or reconstruct logical input/resource ordering outside that seam.

#### Scenario: Introduce EvaluatedScene later
- **WHEN** the renderer-neutral evaluated representation is implemented
- **THEN** all render entry points can substitute it at the planning boundary without changing renderer orchestration, persistence ownership, process execution, or artifact storage
