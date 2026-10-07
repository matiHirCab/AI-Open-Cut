## 1. Approved contract baseline

- [x] 1.1 Record delegated issue-scoped specification approval, verified predecessor CI, source hashes and exact canonical predecessor captures before executable edits.
- [x] 1.2 Add the manually authored speech-alignment-markers-v1 fixture and update ownership/CODEOWNERS and affected catalogs additively; preserve frozen prior catalogs and all historical proof chains.

## 2. Core ownership

- [x] 2.1 Add the closed policy model and canonical marker selection/name/offset/bounds implementation; cover all policy, malformed/missing source, naming collision, count, ordinal and numeric bounds scenarios against the canonical fixture.
- [x] 2.2 Add standalone/batch edit and component-scope alias handling; cover single-result alias success, zero/multiple/forward alias rollback, stale revision, undo/redo/reopen and shared evaluated timing.
- [x] 2.3 Extend generated-asset insertion with optional default-none policy inside tracked atomic publication; cover aligned insertion, conflict, semantic/persistence failure and exact managed-resource/history rollback without changing replacement/transcription semantics.

## 3. Public consumers and retained workflow

- [x] 3.1 Forward optional policy through typed headless request/edit/capability and TypeScript strict schemas/types; canonical Rust protocol and TS positive/negative parity cover the new operation.
- [x] 3.2 Register the thin MCP speech_markers_generate tool and batch union; extend generated speech and preview insert placement policy, preserving owned snapshot/retry/cleanup and existing outputs.
- [x] 3.3 Extend canonical contract tests and exact #61 predecessor projections without changing historical hashes or unrelated drift negatives; keep every existing consumer in the complete contracts command and protected policy.
- [x] 3.4 Add a shared real-headless MCP lifecycle workflow to full integration and packaged smoke, retaining every prior test; add focused retained speech conflict/marker policy tests and public documentation.

## 4. Verify, review and archive

- [x] 4.1 Run cargo fmt --check --all; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace. Keep full logs and fix failures without weakening coverage.
- [x] 4.2 From apps/agent-bridge run bun run typecheck; bun run lint; bun run test; bun run contracts:check; bun run test:integration; bun run test:smoke. Run relevant existing hermetic Python worker unittest/pytest unchanged. Preserve standard/runtime diagnostic evidence distinctions.
- [x] 4.3 Run bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive and moon run root:openspec-validate pre-archive; inspect expected rejection naming only this active change. Perform substantive delegated CODEOWNER/conformance review with openspec-verify-change and fix every mismatch.

## Post-verification delivery obligations (pending)

Synchronize accepted deltas and archive only this verified change using openspec-sync-specs/openspec-archive-change; preserve all prior living requirement blocks/archive bytes and rerun the unchanged protected gate plus strict all-spec validation. Push verified work and create/update one draft cumulative PR targeting main, explicitly ordered after PR156 while unmerged. Require all 11 exact-head CI successes before reporting completion or starting the next dependency-ready issue, and reconcile any user merges without discarding work. These future publication/self-head checks remain pending external exact-SHA receipt obligations, not predeclared source-snapshot passes. Issue completion remains blocked until every delivery gate passes. No merge, deployment or closure is performed.

Local synchronization, archival, strict48-spec validation and protected461-control gate now pass, recorded in local-archival-gate-receipt.json. Final publication/external exact-head CI acceptance remains pending and must be recorded outside this self-referential source snapshot before completion or issue65.
