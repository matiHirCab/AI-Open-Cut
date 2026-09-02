## 1. Commit-State Classification

- [x] 1.1 Replace pointer replacement's undifferentiated error with explicit uncommitted and committed-with-durability-pending outcomes.
- [x] 1.2 Confirm ambiguous replacement errors by strictly rereading `CURRENT`, and remove a newly installed generation only when non-commit is confirmed.
- [x] 1.3 Skip inactive-generation cleanup after uncertain durability while reporting a non-fatal durability warning.

## 2. Invocation Reconciliation

- [x] 2.1 Add startup reconciliation that validates the active pointer/generation before bounded cleanup on every golden invocation.
- [x] 2.2 Support first publication without `CURRENT` by cleaning only recognized stages and pointer temporaries until an active generation exists.
- [x] 2.3 Preserve selected generations and unknown paths, and report startup or post-commit cleanup failure as non-fatal pending work.

## 3. Failure and Reopen Coverage

- [x] 3.1 Replace the pre-commit `PointerReplace` simulation with injectable failures immediately before and after the actual rename boundary.
- [x] 3.2 Test pre-rename failure, post-rename sync failure, and reopening with either the prior or new pointer selected.
- [x] 3.3 Test ordinary-invocation cleanup of recognized stages, pointer temporaries, and inactive generations plus preservation of active and unknown paths.
- [x] 3.4 Test non-fatal startup and post-commit cleanup failures and assert that the checked-in Linux generation digest is unchanged.

## 4. Documentation and Verification

- [x] 4.1 Update golden documentation to describe commit-state classification, uncertain-durability retention, and ordinary-invocation reconciliation.
- [x] 4.2 Run focused golden tests, `cargo fmt --check --all`, strict workspace Clippy, workspace tests, and the exact headless lifecycle test.
- [x] 4.3 Run agent-bridge contracts, typecheck, lint, unit, integration, smoke, and hermetic Python worker tests.
- [x] 4.4 Run required Linux golden conformance with schema-2 report validation without update or recapture, OpenSpec normalization and strict validation, and `git diff --check`.
- [x] 4.5 Apply `$openspec-verify-change`, sync the accepted delta into the living spec, and archive `fix-golden-pointer-commit-recovery` after all checks pass.
