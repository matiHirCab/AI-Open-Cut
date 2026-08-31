## 1. Core Reference Inventory and Validation

- [x] 1.1 Add an `editor-core` asset-reference inventory that classifies media-item, caption-source, draft-operation, snapshot, and managed-path ownership without changing persisted JSON shapes.
- [x] 1.2 Make project loading validate every current and undo/redo media or caption reference against its containing snapshot asset catalog, returning deterministic `ASSET_INTEGRITY_FAILED` diagnostics for legacy dangling references.
- [x] 1.3 Enumerate and validate retained draft asset references under the project lock for get, rebase, preview, commit, deletion, and persistence cleanup paths while preserving explicit discard as a recovery operation.

## 2. Deletion and Managed-File Ownership

- [x] 2.1 Replace the media-only asset deletion check with the centralized inventory so current media, current caption provenance, and durable drafts block deletion through actionable `ASSET_IN_USE` errors while revision checks remain first.
- [x] 2.2 Refactor garbage collection to derive managed relative-path roots from current state, undo/redo history, and durable drafts through the same ownership graph, preserving content-addressed deduplication and `ASSET_GC_FAILED` post-commit warnings.
- [x] 2.3 Audit generated asset replacement, transaction recovery, draft discard/commit, and history eviction against the centralized graph so no path can collect content still reachable by a retained owner.

## 3. Core Regression Coverage

- [x] 3.1 Add deletion tests proving caption provenance and retained drafts block without mutation and return stable reference-class diagnostics.
- [x] 3.2 Add undo/redo, history eviction, and close/reopen tests proving caption-source assets remain resolvable until their final retained root expires.
- [x] 3.3 Add garbage-collection tests for shared managed content across current state, captions, drafts, generated assets, and retained history, including cleanup warning behavior.
- [x] 3.4 Add deterministic persisted fixtures/tests for dangling caption, media, history, and draft references and valid legacy state with unchanged schema shape.

## 4. Contract and Documentation Synchronization

- [x] 4.1 Update canonical error fixtures or headless/agent-bridge assertions only where they expose the new actionable `ASSET_IN_USE` or `ASSET_INTEGRITY_FAILED` diagnostics; keep transport handlers free of duplicated ownership rules.
- [x] 4.2 Update user-facing asset deletion guidance to state that current media, caption provenance, and retained drafts block deletion while undo/redo history retains managed bytes.
- [x] 4.3 Mark each task complete as implemented and run `moon run openspec-validate` to validate the proposal, design, delta specs, and checklist.

## 5. Verification

- [x] 5.1 Run targeted core tests with `cargo test -p opencut-editor-core asset`, `cargo test -p opencut-editor-core caption`, `cargo test -p opencut-editor-core draft`, and `cargo test -p opencut-editor-core history`.
- [x] 5.2 Run Rust formatting and strict linting with `cargo fmt --check --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.3 Run the complete Rust suite with `cargo test --workspace`.
- [x] 5.4 Run affected bridge validation with `moon run agent-bridge:typecheck`, `moon run agent-bridge:lint`, `moon run agent-bridge:test`, `moon run agent-bridge:test-integration`, and `moon run agent-bridge:test-smoke`.
- [x] 5.5 Run `$openspec-verify-change`, resolve every requirements/design/tasks/test mismatch, then run `$openspec-archive-change` so the accepted media-assets and transcription-captions deltas become living specifications.
