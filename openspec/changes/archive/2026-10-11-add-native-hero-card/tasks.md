## 1. Approved specification

- [x] 1.1 Record delegated standing approval for proposal/design/H1–H8/tasks before executable edits; validate `bunx @fission-ai/openspec@1.5.0 validate add-native-hero-card --strict --no-interactive`.

## 2. Read-only recipe and existing MCP workflow

- [x] 2.1 Add bounded versioned native recipe and existing-prompt discovery/retrieval documentation/tests (H1/H2); preserve all structural catalogs/pins and existing prompt behavior.
- [x] 2.2 Implement shared actual MCP two-instance creation/customization, persisted motion, stale/late-invalid/missing-asset rollback and history/resource/reopen evidence; invoke source and fresh packaged smoke (H2–H6).
- [x] 2.3 Add independent native PNG/six-frame range and negative-control checks through existing default source/packaged platform entry points, preserving reports/workflows/limits (H7/H8).

## 3. Checks and conformance

- [x] 3.1 Run focused prompt/recipe tests; bridge `bun run typecheck`, `bun run lint`, `bun run test --no-file-parallelism`, canonical `bun run contracts:check --no-file-parallelism`, `bun run test:integration --no-file-parallelism`, `bun run test:smoke`, and configured existing `bun run scripts/run-platform-renderer.ts`; retain full /tmp/issue182-*.log evidence. Run required workspace `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, strict all-spec validation and hermetic Python runner where inputs changed; unchanged passing evidence may be reused only with explicit source/toolchain/environment proof per AGENTS.md. All published-head platform/native/Rust gates remain mandatory.
- [x] 3.2 Run protected `moon run openspec-validate` pre-archive; require its only rejection to name this active change. Verify all H1–H8 scenario/design/task conformance and resolve findings before synchronization/archival.

## 4. Living specification and delivery

- [x] 4.1 Synchronize verified native-hero-card requirements and archive only this completed change; pass `moon run openspec-validate` and pinned strict all-spec validation again.

After the implementation/archival tasks are complete, preserve unrelated branches, commit/push this issue branch, open a draft PR, verify all required exact-head CI including duration/foundation gates and update delivery evidence. No merge/auto-merge/deployment; coordinate the next dependency-ready issue with the parent before selecting it.
