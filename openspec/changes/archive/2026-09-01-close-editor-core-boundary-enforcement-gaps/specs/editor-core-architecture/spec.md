## MODIFIED Requirements

### Requirement: Inward dependency direction
Editor-core modules MUST follow the complete documented allowed-dependency matrix from orchestration and infrastructure adapters toward domain models and canonical rules, and repository architecture checks MUST reject every undocumented internal edge expressed through direct, grouped, nested, aliased, relative, qualified, or multiline Rust imports and paths in every production item across inline and standard out-of-line modules, regardless of its position around test-only items, as well as forbidden outward imports and duplicated owner implementations. Root-reaching relative paths, `extern crate` aliases, nested conditional attributes, and direct filesystem method calls MUST receive the same enforcement as their canonical forms. Unsupported custom module paths MUST fail architecture analysis explicitly rather than leave production code unexamined.

#### Scenario: Detect an inverted dependency
- **WHEN** any editor-core owner or its out-of-line submodule imports or references another internal owner outside its allowed matrix using a crate-root path, a root-reaching relative path, a top-level or nested use group, an alias chain, or another supported parsed Rust path form in any production item
- **THEN** an automated architecture check fails with the owner, source file, and violated boundary

#### Scenario: Resolve aliases through module ancestry
- **WHEN** a production item reaches a crate-root or external path through lexical aliases, aliases inherited through `super`, or multiple alias steps
- **THEN** the architecture check applies the same dependency and responsibility rules to the canonical path

#### Scenario: Resolve every legal root alias
- **WHEN** a production item reaches a crate owner or external filesystem API through leading `self` and `super` segments, an `extern crate self` alias, an external-crate alias, or a chained lexical/module alias
- **THEN** the architecture check resolves the exact canonical path and applies the existing dependency and responsibility rules

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

#### Scenario: Detect direct filesystem methods
- **WHEN** an asset, store, renderer, or render-planning owner performs filesystem I/O through a native `Path` method or an aliased `std::fs` path instead of its authorized adapter
- **THEN** the architecture check fails even when the source does not spell `std::fs` directly

#### Scenario: Permit adapter path operations
- **WHEN** an owner delegates path inspection or canonicalization through its private persistence or artifact adapter
- **THEN** the architecture check accepts the operation without treating the adapter call as native filesystem I/O

#### Scenario: Detect responsibility duplication
- **WHEN** `renderer` reimplements scene input enumeration through field access or structured patterns, `assets` performs direct filesystem I/O, or `store` reimplements asset garbage collection under any function name
- **THEN** an automated architecture check fails even if the duplicated code compiles or uses aliases

#### Scenario: Review a boundary exception
- **WHEN** a proposed change cannot follow an existing allowed dependency edge
- **THEN** repository review rules require an ADR update and boundary-test matrix update before the new edge is accepted

### Requirement: Replaceable persistence boundary
Storage locking, project and draft directory operations, persisted reads and durable replacement, transaction recovery, managed-file enumeration, and managed-file deletion MUST remain behind one narrow editor-core persistence interface selected by the editor facade, whose real I/O outcomes can be supplied deterministically in tests without changing domain or garbage-collection rules. Project discovery and lookup MUST NOT follow a linked directory or accept a canonical project path outside the configured project root.

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
- **WHEN** a syntactically valid project ID resolves outside the canonical configured project root
- **THEN** the operation fails with `PATH_NOT_ALLOWED` before lock, persisted read, recovery, or garbage collection

#### Scenario: Preserve ordinary project behavior
- **WHEN** a real project directory remains inside the configured root
- **THEN** create, list, open, recovery, drafts, history, managed assets, and garbage collection retain their existing results, errors, and warnings through the selected storage adapter
