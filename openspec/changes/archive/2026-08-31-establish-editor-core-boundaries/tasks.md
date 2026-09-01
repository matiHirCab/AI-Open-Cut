## 1. Architecture Decision and Enforcement

- [x] 1.1 Add an ADR with the canonical owner matrix, allowed dependency graph, persistence and renderer ports, `EvaluatedScene` handoff seam, compatibility constraints, and review procedure for changing an edge.
- [x] 1.2 Link the ADR from `AGENTS.md` and add concise review rules prohibiting duplicated validation in headless, bridge, or desktop code.
- [x] 1.3 Add failing-first editor-core architecture tests for every forbidden dependency edge and for preservation of the stable facade/re-export surface.

## 2. Store Domain Extraction

- [x] 2.1 Add focused failing-first validation tests covering settings, timing, transforms, text/style/color, audio/ducking, keyframes, track compatibility, operations, and draft labels before moving canonical validators into the private `validation` owner.
- [x] 2.2 Add focused failing-first timeline tests covering edit application, alias resolution, atomic batches, revisions, history, split/duplicate timing, locks, and rollback before moving pure mutation logic into the private `timeline` owner.
- [x] 2.3 Extract asset-reference inventory, integrity checks, content-addressed storage policy, retained-path calculation, and garbage-collection decisions into the private `assets` owner with focused current-state/history/draft and fault coverage.
- [x] 2.4 Extract deterministic project/history asset migrations into the private `migrations` owner with current, retained undo/redo, legacy-compatible, and unknown-future-schema tests.
- [x] 2.5 Extract draft serialization and lifecycle orchestration into the private `drafts` owner with reopen, preview, update, rebase, commit-once, discard, revision-conflict, cleanup-warning, and recovery tests.

## 3. Persistence Boundary

- [x] 3.1 Define the crate-private persistence/storage port and production filesystem adapter without changing `EditorCore` construction or public DTOs.
- [x] 3.2 Move locking, bounded JSON reads, project/history/draft document paths, journal validation/replay, synchronized atomic replacement, durable deletion, and managed-file I/O behind the persistence boundary.
- [x] 3.3 Replace global/thread-local persistence fault coupling with deterministic injected adapter outcomes and preserve before-commit rejection, after-commit warning, repeatable recovery, and artifact-cleanup coverage.
- [x] 3.4 Reduce `store` to the stable `EditorCore` facade and cross-owner orchestration, then run `cargo test -p opencut-editor-core store`, `cargo test -p opencut-editor-core persistence`, `cargo test -p opencut-editor-core asset`, `cargo test -p opencut-editor-core draft`, and `cargo test -p opencut-editor-core history`.

## 4. Renderer Separation

- [x] 4.1 Add failing-first deterministic plan tests for frame preview, range preview, draft-equivalent candidate state, export, ordering, transforms/keyframes, captions/text, audio mixing/ducking, and input/path escaping without starting FFmpeg.
- [x] 4.2 Extract declarative project-to-render planning into the private `render_plan` owner with one crate-private scene-input seam reserved for issues #12 and #13 and no process, environment, or publication dependencies.
- [x] 4.3 Define a narrow renderer process executor port; extract FFmpeg/FFprobe readiness, probe, spawn, progress, bounded/redacted diagnostics, cancellation, and exit mapping into `render_process` with injected success and failure tests.
- [x] 4.4 Extract workspace/resource preparation, temporary outputs, overwrite/collision policy, atomic publication, cleanup, MIME/size metadata, and warnings into `render_artifact` with injected I/O fault tests.
- [x] 4.5 Keep `Renderer`, `ExportOptions`, `PreviewRangeOptions`, `RenderArtifact`, and `RenderProgress` signatures/re-exports compatible while routing every existing frame, range, and export entry point through plan, execution, and publication stages.
- [x] 4.6 Run `cargo test -p opencut-editor-core renderer` and the FFmpeg-available native preview/range/export consistency smoke tests, documenting any environment-based skips.

## 5. Compatibility and Integration Evidence

- [x] 5.1 Add fixture/reopen assertions proving project, history, and draft JSON shapes, schema versions, revisions, stable errors/warnings, and existing `EditorCore` public results are unchanged.
- [x] 5.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 5.3 Run `cargo test -p opencut-headless` and verify the existing headless protocol render and reopen suites remain unchanged.
- [x] 5.4 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke`; document any packaged-smoke prerequisite that is unavailable.
- [x] 5.5 Run `moon run openspec-validate`, use `$openspec-verify-change`, resolve every mismatch among requirements/design/tasks/tests/code, and archive with `$openspec-archive-change` so `editor-core-architecture` becomes a living specification.
