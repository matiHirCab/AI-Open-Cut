## 1. Approved scope and inventory

- [x] 1.1 Record standing scoped approval for proposal, delta requirements, design and tasks before executable fixture edits; validate with `bunx @fission-ai/openspec@1.5.0 validate define-illustrated-scene-benchmark --strict --no-interactive` (I1–I4).
- [x] 1.2 Add versioned original hero recipe and three bounded target inventories, independently authored samples and hermetic inventory/drift tests (I1/I3).

## 2. Native evidence and observations

- [x] 2.1 Author through existing core APIs, prove editable roles/state preservation, native frame/range dimensions/count and independent pixels; include required evidence in existing native golden entry point (I2/I3).
- [x] 2.2 Capture three fresh and three repeated frame/range samples with exact source/runtime/host/tool/font identity; record unchanged guards and post-measurement reference targets in docs (I4).

## 3. Verification and delivery

- [x] 3.1 Run focused inventory/native tests; `cargo fmt --check --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; bridge `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`; root `bun run apps/agent-bridge/scripts/run-python-tests.ts`; strict all-spec validation. Preserve logs and failures (I1–I4).
- [x] 3.2 Inspect pre-archive protected `moon run root:openspec-validate` rejection naming only this active change; record complete conformance/scenario evidence and review the verified synchronization/archive plan (I1–I4).

After implementation checkboxes are complete, sync/archive the verified change, then rerun protected validation and strict all-spec validation. Commit/push, open a draft PR and require all exact-head platform/release/foundation CI including the protected duration audit. Those future delivery results are recorded in the PR/external checkpoint after execution, not pre-checked during archival. No merge/deploy.
