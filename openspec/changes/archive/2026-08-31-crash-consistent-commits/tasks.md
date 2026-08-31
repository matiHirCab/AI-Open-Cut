## 1. Recovery Contracts

- [x] 1.1 Reconcile the existing partial issue #81 edits and add `PROJECT_RECOVERY_FAILED`, `PERSISTENCE_RECOVERY_PENDING`, and `DRAFT_CLEANUP_FAILED` to the canonical error catalog and Rust error mapping without changing unrelated diagnostics.
- [x] 1.2 Extend contract assertions so `cargo test -p opencut-editor-core error::tests` verifies the stable recovery error and warning metadata consumed by transports.

## 2. Transaction Persistence

- [x] 2.1 Implement versioned journal serialization, validation, path handling, durable atomic replacement/deletion, and cleanup of unique temporary JSON files in `crates/editor-core`.
- [x] 2.2 Implement locked, idempotent transaction recovery that replays project state, history, and optional committed-draft cleanup before normal reads and fails closed with `PROJECT_RECOVERY_FAILED` for irrecoverable journal state.
- [x] 2.3 Refactor project/history loading and schema migration to operate on a consistent pair and publish any migration through the common transaction path.
- [x] 2.4 Route project creation and every project/history mutation, including undo/redo, through the transaction protocol while preserving optimistic revisions and existing successful results.
- [x] 2.5 Make draft commit record draft consumption in the journal, return committed results with stable post-commit warnings, and ensure retries cannot apply draft operations twice.

## 3. Deterministic Fault Coverage

- [x] 3.1 Add thread-local test fault injection before journal publication and after journal, project, history, draft-cleanup, and journal-cleanup phases.
- [x] 3.2 Add targeted tests proving pre-commit rejection preserves the prior generation and every post-commit interruption recovers one matching project/history generation after reopening.
- [x] 3.3 Add targeted draft tests proving cleanup warnings describe a committed revision, retries never duplicate operations, and reopen completes draft consumption.
- [x] 3.4 Add corrupt/unsupported/inconsistent journal tests proving `PROJECT_RECOVERY_FAILED` preserves diagnostic evidence and does not rewrite live documents.
- [x] 3.5 Add filesystem assertions proving successful persistence and recovery leave no managed journal or unique temporary transaction files.
- [x] 3.6 Reap UUID-suffixed orphan temporary files for project, history, and transaction-journal replacements under the project lock, preserve unrelated files, and add reopen regression coverage.

## 4. Operational Documentation

- [x] 4.1 Add `docs/project-persistence.md` documenting the lock scope, journal commit point, recovery phase order, warnings, stable failure, durability limitations, backup handling, and downgrade procedure.
- [x] 4.2 Link the persistence guarantees from `docs/agent-bridge.md` and keep the OpenSpec delta requirements aligned with the implemented behavior.

## 5. Verification

- [x] 5.1 Run targeted crash and draft recovery coverage with `cargo test -p opencut-editor-core persistence_transaction` and `cargo test -p opencut-editor-core draft_commit_recovery` (using the final test-name filters introduced by tasks 3.2 and 3.3).
- [x] 5.2 Run strict specification validation with `moon run openspec-validate`.
- [x] 5.3 Run Rust formatting with `cargo fmt --check --all`.
- [x] 5.4 Run strict Rust linting with `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.5 Run the full Rust workspace suite with `cargo test --workspace`.
- [x] 5.6 Run `$openspec-verify-change`, resolve all implementation/spec/design/task mismatches, and archive only after explicit approval and successful verification.
