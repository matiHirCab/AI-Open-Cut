## 1. Approval and canonical contract preparation

- [x] 1.1 Obtain explicit owner approval of proposal/design/all five delta specifications/tasks; record exact approved content and scope. Recheck main, PR #133 and dependencies; preserve the published midpoint fix and do not publish without separate authorization.
- [x] 1.2 Cross-layer contract governance: add manually reviewed `contracts/animation-presets-v1.json` with all six property bounds, four curve families, effective defaults, fixed keys/provenance, collision/retirement/lifecycle/error fixtures and named limits. Add ownership/CODEOWNER coverage for new consumers; plan synchronized protocol-1 headless/MCP fixture changes without touching workflow gates.

## 2. Core persisted model and migration

- [x] 2.1 Core model/validation: add typed parameters, provenance records and schema-29 optional sidecar with strict null/unknown-field handling, finite/version/identity checks and bounded maps; keep existing channel payloads unchanged. Add independent model/fixture tests for descriptive retired versions and malformed/orphan records.
- [x] 2.2 Core migrations: implement empty-provenance migration through supported older schemas for root/component items in current state and every retained undo/redo snapshot; reject premature fields and unknown future schemas; preserve primitive/media/revision/timestamp state.
- [x] 2.3 Core persistence tests: prove complete-generation atomicity on invalid current/undo/redo state and named pre/post-commit I/O fault phases, recovery without recompilation/duplicate edits, mixed older snapshots and byte-preserving idempotent reopen. Do not add a new transaction owner or draft format.

## 3. Core compiler, edits and source lifecycle

- [x] 3.1 Core timeline: add the pure nested compiler for `scalar_tween@1`, version dispatch, default normalization, checked safe-integer timing, explicit scalar endpoints and terminal hold/no-loop output. Cover every property/curve with independently fixed keys and sample oracles, not compiler-generated expectations.
- [x] 3.2 Core timeline/validation: integrate `apply_animation_preset` through existing edit/batch paths, canonical target/legacy checks, reject/default and whole-channel explicit-replace policies, stable channel order and provenance updates; preserve unrelated channels/static properties. Cover disjoint/identical collisions, legacy coupled-axis collisions, timing/endpoint/curve boundaries and incompatible/deferred inputs.
- [x] 3.3 Core timeline/store tests: prove stale-revision/missing-item/missing-asset/locked-track/alias misuse errors, earlier `@alias` resolution, non-creator `resultAlias` rejection, one-revision success and whole-batch rollback after earlier successes or final scene-safety failure. Retain existing resource/history/certification budgets.
- [x] 3.4 Core lifecycle: clear metadata on raw collection replacement and changed/removed/retimed primitives; preserve known records for exact core copies/unchanged local channels; sanitize raw component replacements; cover static/move/parent/visibility edits, deletion, split/duration outcomes, and exact undo/redo/reopen of primitives plus source records.
- [x] 3.5 Core draft orchestration: reject preset application intents on create/update/read-materialization/preview/rebase/commit before writes, using the current core error path; preserve accepted draft inputs/version 2. Test existing draft previews of committed presets and isolated raw-channel clearing/commit/discard.

## 4. Typed headless and bridge consumers

- [x] 4.1 Headless: integrate the typed nested edit with current standalone/batch envelopes; advertise `animation_presets_v1` and schema 29 under protocol 1; synchronize request/state/capability fixtures and protocol tests. Keep transport free of domain compilation/validation.
- [x] 4.2 Bridge: add typed preset input/output sidecar schemas, exported nested edit types, `timeline_apply_animation_preset` registration and batch mapping; use injected core transport and existing WriteResult/error adapters. Add compile-time negative fixtures and runtime direct/MCP failure tests, including unknown versions reaching core.
- [ ] 4.3 Cross-layer contract governance: update the manually reviewed headless/MCP catalogs and reviewed expanded-schema digest, all governed consumers and fixture evidence; exercise the new catalog through existing canonical parity entry points as well as focused tests so `bun run contracts:check` actually covers it. Obtain `@matiHirCab` CODEOWNER review; preserve all existing identifiers, annotations, errors and retryability.
- [x] 4.4 Bridge integration/packaged smoke: add one real headless/MCP workflow covering direct apply, creation alias, explicit replace, failed later batch, source clearing, undo/redo and reopen. Exercise a semantically invalid preset after valid operations with structurally valid Zod input so rollback reaches core. Distinguish mocked smoke from native render evidence.

## 5. Conformance and documentation

- [x] 5.1 Core renderer conformance: compare preset-created channels to independently authored primitives through evaluated scenes/frame/range/ordinary draft/export at boundaries/interior/fractional inherited clocks; include visual and gain/audio examples, saved retired provenance and untagged controls. Preserve all old tolerances, goldens and exact-midpoint regression.
- [x] 5.2 Documentation/fixtures: document the single seed, exact parameter bounds and units, explicit versions, half-open timing, collisions, source lifecycle, protocol/schema transition, backup rollback and draft/root-only authoring limitations; distinguish #45's seed from #46's creative pack. Keep every delta scenario linked to an automated test using the coverage map below; document any technically impossible automation explicitly.

## 6. Required verification, synchronization and archival

- [x] 6.1 Core checks from repository root: `cargo fmt --check --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; focused `cargo test -p opencut-editor-core --test animation_presets`; `cargo test -p opencut-editor-core --test animation_channels --test animation_lifecycle_regressions --test inherited_animation_timing --test extended_visual_animation --test motion_blur_sampling --test architecture`; `cargo test -p opencut-headless`. Include new migration/persistence/draft unit tests in the workspace run; record named fault coverage and all skips/failures.
- [x] 6.2 Bridge checks from `apps/agent-bridge`: `bun run typecheck`; `bun run lint`; `bun run test`; `bun run contracts:check`; `bun run test:integration`; `bun run test:smoke`; focused `bun run test:unit tests/animation-presets.test.ts tests/contracts.test.ts`. Hermetic Python regression from repository root: `bun run apps/agent-bridge/scripts/run-python-tests.ts` (existing unittest/pytest runner; no provider behavior change).
- [x] 6.3 Native parity from repository root after configuring actual FFmpeg/FFprobe and the approved font: `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact`; `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact`; the focused seed tests in 6.1 with `OPENCUT_GOLDEN_REQUIRED=1`; and every unchanged mandatory PR native/rules-screen/cache command documented in `docs/ci-parity-gates.md`. Keep existing required markers, thresholds and default-headless rebuild; record actual native evidence separately from mocks and inherited PR #133 evidence. The separate full weekly scope is optional for this change, as clarified by the latest user instruction recorded in `approval.md`; preserve its partial/cancellation evidence without claiming a full pass.
- [x] 6.4 Planning/implementation specification checks: `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; `moon run root:openspec-validate`; isolated `bun --config=/dev/null --no-env-file run scripts/run-ci-policy.ts` on Linux (`NUL` on Windows). Before archival, inspect the full protected result and require any rejection to name only this active change; that rejection is expected and is not a pass. Any other required failure blocks verification/archival.
- [ ] 6.5 After all required implementation checks pass, use `$openspec-verify-change` and resolve every requirement/design/task/test/code mismatch. Obtain the repository-required review/approval for synchronization and use `$openspec-sync-specs`/`$openspec-archive-change` only for this verified change. Do not archive this proposal to bypass authoring rejection.
- [ ] 6.6 Post-archive, repeat strict all-spec validation and the unchanged protected Moon/bootstrap gate; all must pass before completion. Record exact-head evidence and remaining blockers. Do not push/create a PR, merge or deploy without the user's separately relevant authorization.

## 7. Scenario coverage map

This map records owning implementation coverage. Exact named tests and check results are recorded in verification.md as verification completes. Every scenario in the five deltas must be named by an automated case and its recorded check before verification. New test files were created only after the recorded explicit design approval.

| Delta requirement | Owning tests / tasks |
| --- | --- |
| Versioned primitive compilation | Core `animation_presets.rs` identity/unknown-version/default fixtures; 1.2, 3.1, 4.3 |
| Scalar tween seed semantics | Core `animation_presets.rs` six properties x four curves, independent samples, half-open/safe-integer/incompatibility boundaries; 3.1, 3.2 |
| Explicit channel collision policy | Core `animation_presets.rs` default/disjoint/identical/replacement/legacy collision cases; 3.2, 3.4 |
| Descriptive persisted provenance | Core model/store/schema fixture tests, retired-source reopening and independent snapshot oracles; 2.1, 2.3, 3.4, 5.1 |
| Provenance follows primitive lifecycle | Core `animation_presets.rs` plus `animation_lifecycle_regressions.rs`, component and draft cases; 3.4, 3.5 |
| Transactional preset edits | Core/store failure and alias tests, headless protocol and actual MCP workflow; 3.3, 4.1, 4.4 |
| Bounded safe preset expansion | Core direct input/merged/retained-budget tests, bridge structural and semantic failure cases; 2.1, 3.1-3.3, 4.2 |
| Primitive-only shared evaluated behavior | Independent evaluated/native render/audio parity, fractional clocks, legacy and midpoint controls; 5.1, 6.1, 6.3 |
| Atomic schema 29 preset provenance migration | Core migrations/store fixtures for every supported source schema, current/component/undo/redo, premature fields and idempotence; 2.2, 2.3 |
| Fail-closed complete provenance generations | Core retained-corruption/future-schema/I/O phase recovery tests; 2.3 |
| Typed discoverable preset edit parity | Headless protocol, bridge type/Zod/MCP negative/positive cases; 4.1, 4.2, 4.4 |
| Governed versioned animation preset contracts | Rust/TypeScript consumers of checked-in new/catalog fixtures and existing parity runner; 1.2, 4.3, 6.2 |
| Bounded preset authoring boundary for drafts | Core draft/store rejection, injected intent, isolation, commit/discard cases; 3.5 |
