## 1. Approval and canonical contracts

- [x] 1.1 Obtain explicit approval of proposal, design and delta specs before editing implementation; record approver/evidence in proposal.md.
- [x] 1.4 Obtain approval of approval-amendment.md and Bounded extended candidate certification before implementing its conservative rejection behavior; add the 65536-node bound to canonical fixtures after approval.
- [x] 1.2 Update canonical animation/motion-graphics, visual-property, headless/MCP and capability catalogs plus contract ownership entries with scoped targets, bounds, defaults and schema 27; obtain designated CODEOWNER review. Trace: Governed extended visual animation activation.
- [x] 1.3 Add canonical valid/invalid and sampled fixtures for every extended property, target/identity mismatch, bound and compound topology/color case before consumer implementation. Trace: Bounded extended visual channel targets; Deterministic compound visual sampling.

## 2. Core persisted models and migration

- [x] 2.1 Add closed crop/effect/animation-target DTOs and schema-27 identity defaults in editor-core model; keep hierarchy references unchanged. Trace: Bounded authored effect stacks; Bounded extended visual channel targets.
- [x] 2.2 Add migration and fault-injection tests for current state, nonempty undo/redo and retained drafts, premature fields, future schemas and deterministic reopen; implement locked atomic schema-27 migration. Trace: Atomic schema-27 extended visual animation migration.
- [x] 2.3 Add core validation tests and canonical validators for eligible kinds/scopes, effect order/IDs, crop coupled bounds, path topology, gradient topology and channel identities. Preserve errors and revision precedence. Trace: Bounded authored effect stacks; Bounded extended visual channel targets.
- [x] 2.4 Extend existing core standalone/batch/draft property/channel edits with atomic preflight; test aliases, missing/locked references, stale revisions, failed-batch byte preservation, undo/redo, clear and reopen. Trace: Transactional extended visual edits.

## 3. Core sampling and evaluated scene

- [x] 3.1 Add sample-first tests and implement absolute rotation with transform2d exception, crop/static fallback, and compound path/gradient/tint interpolation through canonical curves/loops. Trace: Closed typed animation channels; Deterministic compound visual sampling.
- [x] 3.2 Extend renderer-neutral evaluated facts with sampled crop/geometry/paint/effects, preserving parent/instance/repeater clocks and stacking; test root/nested/hidden content and intermediate invalid coupled geometry. Trace: Shared extended visual channel rendering; Deterministic compound visual sampling.
- [x] 3.3 Implement canonical budget preflight for expanded geometry, crop, ordered effects and raster supports before edit/render publication; test exact limit and overflow cases. Trace: Canonical effect composition; Fail before output side effects.
- [x] 3.4 Implement approved bounded candidate certification; test unsafe spring crop, correlated-safe envelopes, deterministic analysis order, exact exhaustion and unchanged batch/draft/history bytes. Trace: Bounded extended candidate certification.

## 4. Core renderer

- [x] 4.1 Implement common source-crop and animated affine routing with independent asymmetric-source/anchor fixtures. Trace: Shared extended visual channel rendering.
- [x] 4.2 Extend bounded shape raster preparation for sampled path points, per-subpath trim and gradient stops; include every sample dependency in cache keys. Trace: Shared extended visual channel rendering; Deterministic compound visual sampling.
- [x] 4.3 Implement bounded local effect kernels/composition and support expansion without anchor drift; test identity and declared order independently. Trace: Canonical effect composition.
- [x] 4.4 Add deterministic golden/sample fixtures for every extended channel across frame, audiovisual range, draft and export, nested clocks, loop seams/reflection, effects, gradients and untouched audio; assert SSIM/PCM/timing tolerance and unchanged legacy fixtures. Trace: all Shared extended visual channel rendering scenarios.

## 5. Governed consumers and documentation

- [x] 5.1 Synchronize TypeScript/Zod, headless and MCP input/output types and canonical parity tests, preserving thin transports and existing operation names; test standalone and aliased batches through MCP. Trace: Governed extended visual animation activation; Transactional extended visual edits.
- [x] 5.2 Add accurate capability/schema reporting after renderer implementation; document supported subset, coordinates, timing, topology/color/effect semantics, bounds, fallbacks and typed failures in animation/render docs. Trace: Governed extended visual animation activation.
- [x] 5.3 Maintain a scenario-to-test/evidence table in verification.md covering every normative requirement and implementation edit; document technically impossible automation with a concrete justification rather than treating missing coverage as passed.

## 6. Required verification and archival

- [x] 6.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; save full output in uncommitted local logs and report exit status/failures.
- [x] 6.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; run required native render regression coverage with configured FFmpeg/font dependencies. Keep full evidence and resolve failures.
- [x] 6.3 Confirm provider source/protocol is unaffected and from apps/agent-bridge run `bun run scripts/run-python-tests.ts` (the hermetic kokoro-tts test task) as required by the affected packaged-smoke boundary; record the result in verification.md.
- [x] 6.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and protected `moon run root:openspec-validate` before archival; only rejection naming this active change is expected. Resolve every other failure.
- [x] 6.5 Use `$openspec-verify-change` to resolve all code/design/spec/task/test mismatches after required implementation checks pass; retain scenario evidence and CODEOWNER review.
- [x] 6.6 Use `$openspec-sync-specs` and `$openspec-archive-change` to synchronize and archive the verified change; do not archive unrelated work.
- [x] 6.7 Rerun `moon run root:openspec-validate` and strict all-spec validation after archival; completion requires both to pass with full logs.
