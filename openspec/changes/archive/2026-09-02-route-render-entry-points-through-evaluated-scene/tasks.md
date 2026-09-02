## 1. Canonical routing tests

- [x] 1.1 Add editor-core scene-to-plan tests for current media, repeated logical assets, text/font selection, solid-color, rectangle, transforms, keyframes, transitions, fades, automation, roles, ducking, stable ordering, and frame/range/export clipping using only `EvaluatedSceneResult` inputs.
- [x] 1.2 Add renderer-facade tests proving frame preview, audiovisual range preview, draft-materialized frame preview, and export evaluate once through the same path and that invalid input, missing references, non-finite values, complexity failures, inconsistent bindings, and canonical/symlink path escapes invoke no workspace, artifact, process, or publication side effects.
- [x] 1.3 Add headless regression tests for revision conflict before rendering plus render-after-edit, undo, redo, deterministic reopen across separate headless processes, and materialized draft-preview isolation without project, revision, history, or draft mutation; record schema migration and batch-alias behavior as unchanged because this change adds no persisted or edit operation.
- [x] 1.4 Add fixed synthetic FFmpeg parity coverage that compares non-empty semantic plans exactly and verifies sampled frame preview, range preview, and export output at SSIM `>= 0.99`, aligned float-PCM RMS error `<= 0.0001`, and timing within one output video frame; run the focused test in Linux CI with explicit FFmpeg, FFprobe, and deterministic font paths and fail when configured dependencies cannot run.

## 2. Editor-core EvaluatedScene consumption

- [x] 2.1 Change resource preparation to consume evaluated logical media/text instructions and `SceneResourceBindings`, preserving project-root and font-root containment, deterministic binding closure, warnings, and prepared-path separation from `EvaluatedScene`.
- [x] 2.2 Change render planning to consume the closed evaluated visual/audio instructions plus prepared resources, derive deterministic backend input instances, and preserve current FFmpeg filter, timing, transition, animation, text, composition, and audio behavior for every render intent.
- [x] 2.3 Route `Renderer::render_preview`, `render_preview_range`, and `export_video` through one evaluate-then-prepare path so headless draft preview inherits the same semantics.
- [x] 2.4 Remove `SceneInput`, borrowed `SceneEvaluation`, and all duplicate persisted project/track/item/asset traversal from production render planning; update editor-core architecture tests and ADR 0003 ownership enforcement to reject reintroduction.

## 3. Public capability and contract parity

- [x] 3.1 Add `evaluated_scene_rendering` to the canonical headless capability catalog and MCP status catalog while retaining protocol version 1 and all existing render operation/request/response shapes.
- [x] 3.2 Update Rust headless status production and protocol tests, TypeScript headless status typing and Zod schemas, MCP status registration/exposure, and standalone cross-language parity tests for the additive capability.
- [x] 3.3 Run `bun run contracts:check` from `apps/agent-bridge` and resolve every canonical fixture, Rust Serde/status, TypeScript/Zod, and MCP schema/annotation mismatch.

## 4. Documentation and compatibility evidence

- [x] 4.1 Update ADR 0004 and `docs/motion-graphics-implementation-plan.md` to describe the completed production substitution, intent-independent clipping, separate path bindings, capability identifier, and exact SSIM/RMS/timing tolerance.
- [x] 4.2 Document in rendering/headless guidance that top-left pixels, integer-millisecond half-open spans, stable track/item ordering, deterministic local fallback, canonical limits, unsafe-resource rejection, and stable failures are shared by preview and export.
- [x] 4.3 Confirm with serialization/history fixtures that project schema version 6, current state, retained undo/redo generations, drafts, optimistic revisions, atomic batch rollback/aliases, stable error catalog, and deterministic reopen behavior are unchanged.

## 5. Verification and closure

- [x] 5.1 Run `moon run openspec-validate`, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 5.2 Run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`, plus `bun run apps/agent-bridge/scripts/run-python-tests.ts` from the repository root.
- [x] 5.3 Use `$openspec-verify-change`, resolve every requirement/design/task/test mismatch, obtain the required contract CODEOWNER review, and archive with `$openspec-archive-change` so accepted deltas update the living specifications.
