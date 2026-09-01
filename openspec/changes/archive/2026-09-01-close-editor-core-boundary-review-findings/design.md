## Context

The current extraction gives `renderer` injectable process and publication adapters, and gives asset GC a `Storage`-based implementation, but `EditorCore`, transaction recovery, drafts, workspace preparation, fonts, and filter scripts still select the real filesystem internally. The architecture integration test parses owner files with `syn`, yet stops at out-of-line modules, discards parent alias scopes, records only expression fields, and enforces filesystem/GC responsibilities through a few names. These gaps conflict with the already accepted architecture requirements even though the current source and CI are green.

The change is private to editor-core. Public Rust types, JSON contracts, persisted documents, FFmpeg arguments, warning strings, error codes, and ADR 0003's dependency matrix must remain byte-for-byte or behaviorally compatible. The unrelated untracked motion-graphics plan remains outside the change.

## Goals / Non-Goals

**Goals:**

- Make every persistence operation used by `EditorCore` and managed assets flow through one injectable storage object, including the project lock and transaction recovery.
- Make every render workspace/resource/publication operation flow through the renderer's injected artifact adapter.
- Make architecture enforcement fail for out-of-line production modules, parent/root/external alias chains, structured track patterns, direct asset filesystem access, and renamed store-owned GC.
- Preserve production behavior and exact fault mappings while adding deterministic adapter-level failure coverage.

**Non-Goals:**

- Changing public constructors, methods, serialized formats, stable diagnostics, transaction semantics, rendering output, or the allowed dependency graph.
- Exposing storage or renderer adapters publicly, expanding Rust macros, implementing full compiler name resolution, or adding a production dependency.
- Refactoring unrelated domain logic or the outer applications.

## Decisions

### 1. One storage object is owned by the editor facade

`Storage` will require `Debug + Send + Sync` and gain an exclusive-lock operation returning an opaque guard whose drop releases the lock. `EditorCore` will own `Arc<dyn Storage>` and retain `PersistenceFaults` only for deterministic crash-point simulation; its public constructor installs `FileSystemStorage`, while a private test constructor installs a fake. All existing persistence helpers will accept the selected storage rather than constructing `FileSystemStorage` or calling `std::fs` themselves.

Store orchestration will use high-level persistence helpers for project-directory listing/creation and draft removal, while assets and drafts receive only the operations they need. This lets the architecture test forbid low-level filesystem/list/type/removal primitives in `store` without preventing legitimate orchestration.

Alternative considered: keep the phase fault plan as the only injection seam. Rejected because it cannot model lock, read, list, metadata, or cleanup failures produced by the real storage interface.

Alternative considered: add a public generic `EditorCore<S>`. Rejected because it changes the facade and downstream type signatures.

### 2. Artifact I/O owns the complete render workspace lifecycle

`ArtifactIo` will expose private primitives for deterministic request IDs/temporary paths, directory creation/removal, file read/write/removal, entry listing/type inspection, canonicalization, existence, rename, and size. `FileSystemArtifactIo` preserves the current environment/UUID and filesystem behavior. `RenderWorkspace` retains an `Arc<dyn ArtifactIo>` so its `Drop` uses the same adapter that created it.

Resource resolution, font discovery and reads, text/filter writes, temporary-output creation, publication, metadata, and best-effort cleanup will all receive the adapter. Existing error-stage mapping remains at the current owner boundary. Renderer tests use a recording/failing adapter and do not add a public injection API.

Alternative considered: keep workspace/resource I/O as free filesystem functions because they already live in `render_artifact`. Rejected because ownership without facade injection does not satisfy deterministic failure testing.

### 3. Architecture analysis follows the source module graph and canonical paths

The real-source analyzer will start from each owner file and recursively resolve `mod child;` through Rust's `child.rs` and `child/mod.rs` conventions while tracking logical module depth and module directories. A `#[path]` module is rejected with an owner/file diagnostic instead of being silently skipped. In-memory syntax tests remain supported through a source-loader abstraction, and filesystem fixtures exercise recursion.

Alias frames will distinguish lexical blocks from module ancestry. Canonical alias maps will resolve fixed-point chains for crate-root and external paths, so `super::root::persistence` and chained aliases of `std::fs` retain their original meaning. The visitor will record named members from both expression fields and struct patterns.

Alternative considered: reject every out-of-line module. Rejected because ordinary Rust module layout is supported and can be analyzed deterministically. Full macro expansion remains excluded because it requires compiler integration; direct parsed imports and paths are still mandatory.

### 4. Responsibility enforcement is structural

`assets` will be rejected for any canonical path into `std::fs` or filesystem file primitives, regardless of the called method or aliases. Once store filesystem operations are routed through persistence helpers, `store` will be rejected for direct filesystem paths and low-level storage enumeration/type/deletion operations; the integration test will also require the canonical assets GC delegation. Renderer scene enumeration will detect `tracks` in expression and pattern fields. Function-name-only GC assertions will be removed.

Alternative considered: add more forbidden function names. Rejected because renaming preserves the duplicated responsibility and repeatedly reopens the same bypass.

## Risks / Trade-offs

- [Passing storage through all persistence paths can alter error mapping] → retain existing mapping at each helper and add facade-level equality assertions for codes and warnings.
- [A boxed lock guard could weaken lifetime guarantees] → the guard is owned for the full operation scope and the filesystem implementation keeps the locked file alive until drop.
- [Workspace cleanup errors are intentionally ignored today] → preserve best-effort cleanup while recording adapter calls in tests; do not introduce a new public failure.
- [Module resolution differs around custom paths] → reject `#[path]` explicitly and test both standard out-of-line layouts.
- [Alias resolution can create false positives] → use lexical frames, exact canonical segments, and similarly-prefixed identifier regressions.
- [The private refactor is broad] → land as one follow-up commit after full compatibility, native-renderer, recovery, bridge, smoke, and strict OpenSpec gates.

## Migration Plan

No runtime or data migration is required. After approval, implement persistence and artifact adapters, harden architecture analysis, run all repository gates, verify and archive this change, repeat review against `main...HEAD`, and push one additional commit to PR #94. Rollback is that single commit; persisted data remains compatible.

## Open Questions

None.
