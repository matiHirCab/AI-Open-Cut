## 1. Specification and regression
- [x] 1.1 Validate/approve this scoped proposal, design, delta and tasks with bunx @fission-ai/openspec@1.5.0 validate preserve-legacy-slot-edit-atomicity --strict --no-interactive.
- [x] 1.2 Independently reproduce all three schema44 rejection paths with exact project/history byte assertions before implementation; cover current and retained revisions without triggering a preliminary migrating read.

## 2. Core correction
- [x] 2.1 Route slot definition and instance update operations through existing staged migration; keep aliases, revision conflicts, draft/resource ownership and typed failures intact.
- [x] 2.2 Add valid migration/edit, stale revision, undo/redo/reopen and commit-point fault/recovery coverage; map both scenarios to actual tests.

## 3. Verification and conformance
- [x] 3.1 Pass cargo +1.97.0 fmt --all -- --check, cargo +1.97.0 clippy --workspace --all-targets -- -D warnings and cargo +1.97.0 test --workspace; retain full logs.
- [x] 3.2 In apps/agent-bridge pass bun run typecheck, bun run lint, bun run test:unit, bun run contracts:check, bun run test:integration and the complete original packaged smoke suite (39/39 via the documented supported pinned cross-build bootstrap after an independent native Bun readiness failure); retain failed native bootstrap evidence and require its ordinary exact-head CI gate. Run applicable root policy/agent-context, hermetic workers/native/platform requirements and record limitations without weakening checks.
- [x] 3.3 Pass pinned strict all-OpenSpec validation and inspect prearchive moon run root:openspec-validate (only rejection naming this active change is expected).

## Subsequent delivery gates
After local implementation checks, apply openspec-verify-change, resolve all completeness/correctness/coherence mismatches, then openspec-sync-specs and openspec-archive-change for this change only. Pass final protected Moon and strict all-spec validation. After archival, commit/push only the scoped correction, open and attach a separate draft PR, and require every protected CI check at its exact head before reporting completion. Preserve the #207 branch/worktree and rebase its later cumulative work onto this correction. No merge/deployment.
