## 1. Scope and regression

- [x] 1.1 Reproduce standalone and aliased-batch legacy migration defect and trace approved transactional requirements.
- [x] 1.2 Obtain independent reviewer acceptance of proposal/spec/design scope before executable edits; record acceptance and any CODEOWNER gate.
- [x] 1.3 Add core and actual headless regression coverage for rejected legacy requests, retained history/components/drafts/assets/fonts and canonical typed errors.

## 2. Core correction

- [x] 2.1 Separate prepare-only migration/resource planning from eager load in existing store/assets owners without new dependency edges.
- [x] 2.2 Apply preset-containing transactions exactly once, certify complete candidate, and commit migration/edit/history/drafts through one journal.
- [x] 2.3 Preserve pre-journal rollback and post-journal recovery semantics; add targeted fault/resource/lifecycle tests.
- [x] 2.4 Obtain independent Sol medium review of code and regression coverage and resolve findings.

## 3. Verification and delivery

- [ ] 3.1 Run cargo fmt --all --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace; run affected native/render shards with required environment.
- [ ] 3.2 Run bridge bun run typecheck, bun run lint, bun test, bun run contracts:check, MCP integration, packaged smoke, and hermetic Python worker checks using repository commands.
- [ ] 3.3 Run moon run openspec-validate, openspec-verify-change, synchronize/archive accepted delta, then protected repository gate per docs/spec-driven-development.md.
- [ ] 3.4 Verify commitability and dry-run push, report exact-head checks and recoverable artifacts before publication; issue 46 remains separate and blocked pending concrete approval.

Independent reviewer `/root/correction_review` (gpt-6-sol, medium) explicitly accepted these artifacts before code on 2026-10-02: existing transactional restoration; no fresh public/persisted contract approval. Conditions: track rollback for new assets/fonts, preserve preexisting files, verify copied bytes, and report any cleanup limitation.

Independent code review completed with no remaining correctness/ADR blocker. Targeted tests: standalone/alias schema28, schema18 resources/components/v1 drafts/undo+redo, copy corruption and font write rollback, nine persistence phases, journal sync-after-rename, complete-candidate scene budget schema28+29, and actual headless error/undo/redo passed. Full gates remain pending below.
