## MODIFIED Requirements

### Requirement: Inward dependency direction
Editor-core modules MUST follow the complete documented allowed-dependency matrix from orchestration and infrastructure adapters toward domain models and canonical rules, and repository architecture checks MUST reject every undocumented internal edge expressed through direct, grouped, nested, aliased, relative, qualified, or multiline Rust imports and paths in every production item across inline and standard out-of-line modules, regardless of its position around test-only items, as well as forbidden outward imports and duplicated owner implementations. Unsupported custom module paths MUST fail architecture analysis explicitly rather than leave production code unexamined.

#### Scenario: Detect an inverted dependency
- **WHEN** any editor-core owner or its out-of-line submodule imports or references another internal owner outside its allowed matrix using a crate-root path, a root-reaching relative path, a top-level or nested use group, an alias chain, or another supported parsed Rust path form in any production item
- **THEN** an automated architecture check fails with the owner, source file, and violated boundary

#### Scenario: Resolve aliases through module ancestry
- **WHEN** a production item reaches a crate-root or external path through lexical aliases, aliases inherited through `super`, or multiple alias steps
- **THEN** the architecture check applies the same dependency and responsibility rules to the canonical path

#### Scenario: Reject an unexamined custom module path
- **WHEN** an editor-core owner declares an out-of-line production module using `#[path]`
- **THEN** architecture analysis fails with the owner and source file instead of silently skipping the module

#### Scenario: Exclude test-only dependencies without truncating production
- **WHEN** an owner contains a test-only item and additional production items before or after it
- **THEN** the architecture check excludes only the test-only subtree and still enforces every production dependency in every analyzed source file

#### Scenario: Detect responsibility duplication
- **WHEN** `renderer` reimplements scene input enumeration through field access or structured patterns, `assets` performs direct filesystem I/O, or `store` reimplements asset garbage collection under any function name
- **THEN** an automated architecture check fails even if the duplicated code compiles or uses aliases

#### Scenario: Review a boundary exception
- **WHEN** a proposed change cannot follow an existing allowed dependency edge
- **THEN** repository review rules require an ADR update and boundary-test matrix update before the new edge is accepted

### Requirement: Replaceable persistence boundary
Storage locking, project and draft directory operations, persisted reads and durable replacement, transaction recovery, managed-file enumeration, and managed-file deletion MUST remain behind one narrow editor-core persistence interface selected by the editor facade, whose real I/O outcomes can be supplied deterministically in tests without changing domain or garbage-collection rules.

#### Scenario: Inject a persistence fault
- **WHEN** a deterministic facade test supplies a storage failure during locking, directory creation or listing, persisted reading, durable replacement, recovery, draft cleanup, or garbage collection before or after the durable commit point
- **THEN** the canonical operation returns the same typed rejection, recovery warning, draft-cleanup warning, or `ASSET_GC_FAILED` warning required by existing project persistence semantics

#### Scenario: Use filesystem persistence
- **WHEN** the production editor core creates, lists, opens, migrates, mutates, drafts, recovers, or garbage-collects a project
- **THEN** the filesystem adapter performs locking and durable managed-file I/O without exposing filesystem details or concrete storage selection to timeline, draft, validation, asset-policy, or store orchestration rules

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
