## Context

`FileSystemStorage::entry_kind` currently uses metadata that follows symlinks. Consequently `list_projects` can treat a link under `projects_root` as a real project directory and later open, recover, or garbage-collect its external target. `existing_project_dir` canonicalizes the selected directory but does not verify that the result remains under the canonical configured root.

The `syn` architecture analyzer handles ordinary out-of-line modules and several alias forms, but it only inspects immediate `cfg_attr` arguments, treats standalone `#[test]` functions as production, does not normalize every legal root-reaching relative path, and ignores `extern crate` aliases. Its direct-filesystem rule recognizes canonical `std::fs` paths but not filesystem methods invoked on `Path` values.

## Goals / Non-Goals

**Goals:**

- Prevent linked or canonicalized project directories from escaping the configured project root before any lock, read, recovery, or garbage collection.
- Make architecture enforcement cover the identified legal Rust forms and direct filesystem method calls deterministically.
- Preserve adapter injection, normal project behavior, public interfaces, persisted representations, stable diagnostics, and the allowed dependency matrix.

**Non-Goals:**

- Supporting custom `#[path]` modules, full compiler name resolution, macro expansion, or arbitrary Rust type inference.
- Changing ADR 0003, exposing dependency injection publicly, changing FFmpeg behavior, or altering valid project layouts.

## Decisions

### 1. Directory-entry classification does not follow links

`StorageEntryKind` will represent symbolic links separately, and the filesystem adapter will implement classification with `symlink_metadata`. High-level project-directory listing accepts only real directories and skips links. Draft, transaction, and asset enumeration retain their existing file/directory behavior for ordinary entries; linked entries are not treated as managed files or directories.

`existing_project_dir` will canonicalize the candidate through the selected storage adapter and require the result to start with the already-canonical `projects_root` before acquiring the lock or performing any project I/O. An escape returns `PATH_NOT_ALLOWED`; an ordinary missing project retains `PROJECT_NOT_FOUND`. A deterministic fake-storage test proves the check on every platform, and a Unix filesystem test covers an actual directory symlink.

Alternative considered: only restore the previous `DirEntry::file_type` behavior in listing. Rejected because direct lookup through a linked project ID would still escape confinement.

### 2. Attribute analysis is recursive and conservative

The analyzer will recursively inspect `Meta` values selected by `cfg_attr`. A false predicate suppresses its payload; true or unknown predicates inspect every payload recursively, and malformed metadata remains a deterministic owner/file error. Any production-reachable `path` rejects the module.

Direct `#[test]` items are classified as test-only alongside items disabled by the production cfg evaluator. The visitor still processes every sibling production item before and after the excluded subtree. Attributes that merely become `#[test]` under a test-only `cfg_attr` are not incorrectly excluded from production.

### 3. Canonical paths include relative roots and extern-crate aliases

Canonicalization first removes legal leading `self` qualifiers, then resolves `super` against logical module depth, and finally expands lexical/module aliases to a fixed point. Alias expansion uses a visited-state set and a bounded iteration count so cycles terminate deterministically.

The visitor records `extern crate self as root` as `root -> crate` and external declarations such as `extern crate std as platform` as `platform -> std`. Exact owner segments remain mandatory, so similarly prefixed identifiers do not become dependencies.

### 4. Adapter operations are syntactically distinct from native Path I/O

Private storage and artifact path-inspection methods will use adapter-specific names where they overlap native `Path` methods. The analyzer can then reject canonical `std::fs` access and native filesystem method names (`read_dir`, `read_link`, `metadata`, `symlink_metadata`, `canonicalize`, `exists`, `try_exists`, `is_file`, and `is_dir`) in owners that must delegate I/O, without type inference or false positives on approved adapters.

The rule applies to `assets`, `store`, `renderer`, and render planning/orchestration owners according to their existing responsibilities. `persistence` and `render_artifact` remain the authorized filesystem adapters. Store's positive delegation requirement for `assets::garbage_collect` remains unchanged.

## Risks / Trade-offs

- [Non-following classification changes linked managed entries] -> links are explicitly non-managed and cannot be traversed or deleted as project-owned content; ordinary files and directories retain current behavior.
- [Canonical prefix checks can be platform-sensitive] -> compare canonical `PathBuf` values supplied by the same adapter and cover the decision with a cross-platform fake.
- [Filesystem method names can collide with adapter calls] -> rename only private adapter methods and test both rejected native calls and permitted adapter calls.
- [Conservative unknown cfg evaluation may reject code that is inactive on one target] -> this is intentional because architecture CI must not leave potentially productive custom modules unexamined.

## Migration Plan

No data or runtime migration is required. After approval, add failing regressions, implement the private changes, run the full repository gates, verify and archive the OpenSpec change, repeat review against `main...HEAD`, and push one additional commit to PR #94. Rollback is that single commit.

## Open Questions

None.
