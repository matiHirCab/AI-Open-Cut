## 1. Specification and contract foundation
- [x] 1.1 Independently review and explicitly approve these issue220 artifacts before implementation; record approval and environment evidence.
- [x] 1.2 Add canonical artwork v1 fixture, governed consumers/CODEOWNERS and precise additive catalog inverse proofs before consumers; preserve every historical raw capture and digest.
- [x] 1.3 Introduce schema47 optional derivation model and deterministic current/history migration tests, including future and premature metadata rejection.

## 2. Core assets and transaction
- [x] 2.1 Implement pinned PNG normalization and bounded pure cleanup/dilation/compositing with independent exact RGBA/mask oracles and malformed/animated/limit rejection.
- [x] 2.2 Extend canonical source reference/integrity/deletion/draft/history/GC ownership with cycle/hash/type/geometry tests; preserve existing roots.
- [x] 2.3 Implement core prepare/commit manifest facade using existing locks and atomic persistence; test conflicts, repeat/cancel/failure, source hash preservation, reopen/undo/redo and derivative byte identity.

## 3. Transport and operator surface
- [x] 3.1 Add thin typed headless prepare/commit operations, canonical request/output parity and optional asset metadata consumers.
- [x] 3.2 Add CLI preview/apply/discard with explicit approval, path confinement, bounded manifests/files, staging ownership and independent unsafe/foreign/missing/altered-file tests.
- [x] 3.3 Document supported PNG parameters, limits, padding, review/open commands, rollback/uncertainty and absence of GUI readiness evidence; demonstrate native synthetic preview/commit/reopen through actual CLI/headless.

## 4. Required verification and independent review
- [x] 4.1 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, focused artwork/migration/ownership and native synthetic acceptance; preserve full logs in /tmp.
- [x] 4.2 In apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, build/assembled `bun run test:smoke`; run hermetic Python worker suites and required native/platform checks. Record unavailable platform/GUI evidence honestly; no optional source result substitutes for a required check.
- [x] 4.3 Run pinned strict all-spec validation and `moon run root:openspec-validate` before archival; only this active-change rejection is expected. Any unrelated failure blocks archival.
- [x] 4.4 Use openspec-verify-change and independent implementation review to resolve requirement/design/test/code mismatches; retain the separate CODEOWNER requirement for the publication stage.

## Post-verification lifecycle actions

These remain mandatory completion obligations, tracked in the execution plan and verification report after implementation verification. They are not marked completed before execution.

- Only after all required implementation checks pass, use openspec-sync-specs and openspec-archive-change; then rerun protected Moon gate and pinned strict all-spec validation. Never archive incomplete work to bypass policy.
- Publish own branch and draft PR only; request designated CODEOWNER review through the actual PR without claiming acceptance. CODEOWNER acceptance remains separate and pending until genuine review occurs.
- Observe exact-head checks until terminal and record all failures/blockers. Completion requires the protected aggregate at the published head; any failed/skipped required check remains a blocker.
- No merge, deployment or second issue.
