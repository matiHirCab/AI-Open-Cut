## Context

Issue #83 introduced private owners for editor domain, persistence, and rendering concerns, but review found four gaps between the accepted ADR/specification and the implementation. `renderer` still enumerates project media and constructs FFmpeg commands, its process and artifact ports are not reachable through the facade, `store` still owns asset garbage collection and direct managed-file I/O, and the architecture test checks only a subset of forbidden edges.

The correction must preserve all public constructors, DTOs, Serde shapes, stable errors/warnings, revisions, recovery behavior, and native render output. The unrelated untracked motion-graphics plan remains outside the change.

## Goals / Non-Goals

**Goals:**

- Make one private scene-evaluation seam own logical input ordering, resource requests, timing, dimensions, and output intent.
- Make real and fake process/artifact adapters selectable behind the unchanged `Renderer` facade.
- Put asset-GC policy in `assets` and all managed-file operations behind `Storage`.
- Enforce every documented internal dependency edge and the reviewed ownership exclusions automatically.
- Add facade-level and owner-level tests that fail before each correction and preserve compatibility evidence.

**Non-Goals:**

- Defining the roadmap `EvaluatedScene` public or persisted model.
- Changing public Rust APIs, headless/bridge contracts, project schemas, errors, warnings, or output encoding.
- Moving the complete draft lifecycle or making every `EditorCore` persistence operation injectable in this follow-up.
- Creating a pull request or rewriting the already-published issue #83 commit.

## Decisions

### 1. Use a two-stage logical and prepared render plan

`render_plan` will expose crate-private logical evaluation data containing ordered media inputs, text/resource requests, dimensions, duration, and `RenderIntent`. Artifact preparation resolves project-confined media paths and materializes requested text resources. A final declarative render plan combines those prepared resources with the logical scene and contains everything process execution needs.

`renderer` will call evaluate → prepare → finalize → execute and will not iterate tracks/items, resolve assets, write filter scripts, or construct `Command`. `render_process` will translate the final plan and output request into a `Command`. This leaves one inward scene input that issue #12 can replace without changing process or publication code.

Alternative considered: retain input enumeration in `renderer` and only add fields to the current plan. Rejected because it leaves the `EvaluatedScene` substitution coupled to `Project` traversal in the facade.

Alternative considered: define `EvaluatedScene` now. Rejected because issues #12/#13 own its semantics.

### 2. Store production adapters as private shared trait objects

`Renderer` will retain its public constructor and `Clone`/`Debug` behavior while privately storing `Arc<dyn ProcessExecutor + Send + Sync>` and `Arc<dyn ArtifactIo + Send + Sync>`. Both traits will require `Debug`; production implementations remain the defaults. A crate-private test constructor will replace them with deterministic fakes.

Readiness, probing, execution, publication, artifact metadata, and failure cleanup will delegate through these fields. The process port will receive declarative plan/output data rather than a preconstructed `Command`, so command construction belongs exclusively to `render_process`. The artifact port will be passed to publication/metadata helpers, allowing failures to be exercised through public render entry points.

Alternative considered: generic parameters on `Renderer`. Rejected because they would change the public type and leak infrastructure choices to callers.

Alternative considered: test the traits directly without injecting them into `Renderer`. Rejected because that cannot prove facade error mapping and cleanup.

### 3. Move garbage collection to assets over the storage port

`assets` will own retained-path evaluation, recursive managed-file discovery, and deletion decisions. `Storage` gains only the missing entry-kind/directory query needed for recursion; the filesystem adapter implements it. A `garbage_collect_with` path accepts a storage implementation for deterministic tests, while production uses `FileSystemStorage`.

The existing named garbage-collection fault remains at the same post-commit point and continues producing exactly `ASSET_GC_FAILED`. `store` only invokes the owner and appends its warnings.

To restore inward direction, `assets` will stop importing `EditDraft`. Asset-reference APIs will accept draft ID plus operations (or an iterator of those views), and `store` will adapt loaded drafts at the orchestration boundary.

Alternative considered: leave traversal/deletion in `store` because it coordinates persistence. Rejected because this duplicates the declared asset owner and bypasses the storage seam.

### 4. Enforce an explicit owner matrix

The architecture test will enumerate every private owner and, for each one, the complete set of allowed internal module imports. It will scan production source separately from `#[cfg(test)]` content and fail on any owner import not present in the matrix. Focused assertions will additionally forbid planning/project traversal in `renderer`, direct GC implementations in `store`, process/environment/publication dependencies in `render_plan`, and managed-file filesystem calls outside persistence/artifact adapters.

The ADR graph will explicitly show the already-intended outward-adapter dependency `render_artifact -> render_plan` for resource request/result contracts. Any future edge still requires an ADR and matrix update in the same change.

Alternative considered: keep a short list of forbidden tokens. Rejected because omissions allow dependency inversions to pass while the test remains green.

## Risks / Trade-offs

- [Logical and prepared plans could accidentally change FFmpeg argument ordering] → Preserve current argument generation verbatim in `render_process` and compare command arguments plus native preview/range/export output tests.
- [Trait-object fields could alter public derives or thread properties] → Require `Debug + Send + Sync`, use `Arc`, and retain the existing public `Renderer` signatures and root re-exports.
- [GC relocation could change warning timing or deletion reachability] → Reuse the existing retained-path calculation and named fault phase; assert current/history/draft roots and post-commit warning behavior.
- [Text-based architecture checks can produce false positives] → Scan only production sections and exact internal module import tokens, with the complete owner list centralized in the test.
- [The follow-up can grow into a broader persistence rewrite] → Limit storage changes to recursive managed-file enumeration/deletion needed by GC and leave other persistence construction unchanged.

## Migration Plan

No data or public API migration is required. Implement in four independently testable stages: architecture failing tests, render planning/adapters, asset GC/storage, then compatibility and integration gates. Rollback is the additional follow-up commit; the existing issue #83 commit remains intact.

## Open Questions

None. The selected scope is the four review findings, published as one additional commit on `feat/issue-83-editor-core-boundaries`.
