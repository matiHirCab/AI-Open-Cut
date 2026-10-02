## 1. Scope and regression

- [x] 1.1 Reproduce standalone and aliased-batch legacy migration defect and trace approved transactional requirements.
- [x] 1.2 Obtain independent reviewer acceptance of proposal/spec/design scope before executable edits; record acceptance and any CODEOWNER gate.
- [x] 1.3 Add core and actual headless regression coverage for rejected legacy requests, retained history/components/drafts/assets/fonts and canonical typed errors.

## 2. Core correction

- [x] 2.1 Separate document preparation and tracked speculative asset staging from eager load in existing store/assets owners without new dependency edges.
- [x] 2.2 Apply preset-containing transactions exactly once, certify complete candidate, and commit migration/edit/history/drafts through one journal.
- [x] 2.3 Preserve pre-journal rollback and post-journal recovery semantics; add targeted fault/resource/lifecycle tests.
- [x] 2.4 Obtain independent Sol medium review of code and regression coverage and resolve findings.

## 3. Verification and delivery

- [ ] 3.1 Run cargo fmt --all --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace; run affected native/render shards with required environment.
- [x] 3.2 Run bridge bun run typecheck, bun run lint, bun test, bun run contracts:check, MCP integration, packaged smoke, and hermetic Python worker checks using repository commands.
- [ ] 3.3 Run moon run openspec-validate, openspec-verify-change, synchronize/archive accepted delta, then protected repository gate per docs/spec-driven-development.md.
- [ ] 3.4 Verify commitability and dry-run push, report exact-head checks and recoverable artifacts before publication; issue 46 remains separate and blocked pending concrete approval.

Independent reviewer `/root/correction_review` (gpt-6-sol, medium) explicitly accepted these artifacts before code on 2026-10-02: existing transactional restoration; no fresh public/persisted contract approval. Conditions: track rollback for new assets/fonts, preserve preexisting files, verify copied bytes, and report any cleanup limitation.

Independent code review completed with no remaining correctness/ADR blocker. Targeted tests: standalone/alias schema28, schema18 resources/components/v1 drafts/undo+redo, copy corruption and font write rollback, nine persistence phases, journal sync-after-rename, complete-candidate scene budget schema28+29, and actual headless error/undo/redo passed. Full gates remain pending below.

Follow-up independent probe found deferred future asset font selection could accept fallback when eager loading rejects an extensionless filename. Reviewer accepted amended speculative asset staging with request-wide rollback before code; no fresh governed contract gate. Full final executable checks must be rerun on the amended stable tree.

Exact-head static review of c454dee2 found no correctness/design blocker. Reviewer also approved the editorial destination scenario qualifier: integrity errors are asserted when staging/publication reaches that destination, preserving earlier invalid-preset error priority. This wording correction changes no executable or public contract semantics.

Final executable source c454dee2: fmt PASS, strict workspace Clippy PASS, workspace856 passed/9 default-ignored benchmarks, contracts Rust222+TS361 PASS, MCP15 PASS, packaged smoke9 PASS, fresh preset native2 PASS and headless lifecycle1 PASS. Native baseline and required PR rules-screen matrix remain running; task3.1 is intentionally incomplete. Prior TS typecheck/lint/unit431+1 optional native skip and hermetic Python10+5 passes are reused only because their source/fixture/toolchain inputs remain unchanged; required native bridge tests passed separately. Initial concurrently loaded MCP/smoke timeout failures and the corrected05dc Clippy failure remain disclosed in local evidence logs.
