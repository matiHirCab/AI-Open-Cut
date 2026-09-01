## 1. Architecture Enforcement

- [x] 1.1 Add failing-first architecture coverage for the complete owner-to-allowed-import matrix, every private owner, stable root re-exports, and reviewed responsibility exclusions.
- [x] 1.2 Remove the `assets -> drafts` edge by changing draft asset-reference inputs to draft IDs plus operation slices/iterators, then update the ADR graph and matrix for every intentional edge.

## 2. Render Planning Seam

- [x] 2.1 Add failing-first deterministic frame, range, and export plan tests covering ordered media inputs, text/resource requests, dimensions, duration, composition/audio data, and output intent without process or filesystem access.
- [x] 2.2 Introduce the private logical scene-evaluation result and prepared/final plan stages, preserving the single `SceneInput` substitution point for future `EvaluatedScene`.
- [x] 2.3 Move project track/item traversal and logical input/resource ordering from `renderer` into `render_plan`, leaving the facade as evaluate → prepare → finalize → execute orchestration.
- [x] 2.4 Move FFmpeg command and output-argument construction into `render_process` while preserving exact preview, range, export, escaping, timing, audio, and progress behavior.

## 3. Injectable Renderer Adapters

- [x] 3.1 Add private `Arc`-backed process and artifact adapter fields to `Renderer`, keep `Renderer::new`, `Clone`, `Debug`, public methods, and root re-exports unchanged, and add only a crate-private/test constructor for replacement adapters.
- [x] 3.2 Delegate readiness, probe, execution, publication, metadata, and failure cleanup through the injected adapters; remove production hard-coding of `SystemProcessExecutor` and `FileSystemArtifactIo` from facade paths.
- [x] 3.3 Add facade-level fake-adapter tests for readiness/probe failures, spawn/non-zero/diagnostic failures, successful execution followed by publication/metadata failure, cleanup, and unchanged structured errors.

## 4. Asset Garbage Collection Boundary

- [x] 4.1 Add the minimal storage entry-kind/directory operation required for recursive managed-file enumeration and focused adapter tests.
- [x] 4.2 Move retained-path garbage-collection decisions, recursive enumeration, and deletion from `store` to `assets`, using `Storage` for all managed-file I/O.
- [x] 4.3 Preserve the named post-commit garbage-collection fault and exact `ASSET_GC_FAILED` warning, with tests for current state, undo/redo history, durable drafts, collection, and injected failure.
- [x] 4.4 Reduce `store` to invoking the asset owner and appending its warnings; verify architecture tests reject reintroduced GC or direct managed-asset deletion there.

## 5. Compatibility, Verification, and Publication

- [x] 5.1 Run focused `cargo test -p opencut-editor-core render_plan`, `renderer`, `render_process`, `render_artifact`, `asset`, `persistence`, `draft`, `store`, `history`, architecture, and compatibility suites, including FFmpeg-available native consistency tests.
- [x] 5.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo test -p opencut-headless`.
- [x] 5.3 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke`; document any unavailable packaged prerequisite.
- [x] 5.4 Run strict OpenSpec validation and `$openspec-verify-change`, resolve every mismatch, then archive the change so the living `editor-core-architecture` spec is updated.
- [x] 5.5 Run `$code-review` over `main...HEAD`, resolve every actionable finding, confirm the unrelated motion-graphics document remains untracked, create one additional commit without rewriting history, and push `feat/issue-83-editor-core-boundaries` without creating a pull request.
