## 1. Reviewed specification and canonical cases

- [x] 1.1 Obtain explicit independent approval of proposal/design/all delta requirements/tasks before executable edits; record artifact hashes and any approved precision updates.
- [x] 1.2 Freeze canonical narration-blocks-v1 planning/Unicode/abbreviation/manual-boundary/provider-expansion/manifest/wire negative cases and an explicit predecessor inverse plan before implementing consumers. Register canonical owner, all consumers and CODEOWNER surfaces; do not generate expected cases from live outputs.

## 2. Core model, compatibility and transactions

- [x] 2.1 Implement model DTOs and pure bounded planning/manifest validation with per-block pinned profile snapshot, globally stable UUIDs, exact scalar source spans, optional real caption/cue ownership and typed failure coverage.
- [x] 2.2 Adopt schema47 consistently across current state, history, drafts, raw introduced-field checks and journal recovery. Cover schema46/current/history/draft migration, older forged metadata, missing/null current fields, future versions and fault-atomic unchanged generations.
- [x] 2.3 Implement one staged native commit transaction over all generated blocks with tracked resource rollback and existing font/edit/preflight/persistence ordering. Cover late failures, locks, safe placement, real alignment opt-ins, stale revisions, one undo/redo/reopen and complete recovery.
- [x] 2.4 Apply canonical manifest validation to existing mutation paths and explicit detach operation. Cover invalidating legacy delete/regenerate/batch rejection and independent single-block detachment with exact unchanged unrelated blocks/provenance.

## 3. Thin transport and provider-neutral workflow

- [x] 3.1 Add uniquely named headless planning/commit/detach dispatch and TS structural schemas/types, canonical catalogs/parity consumers and explicit schema transition inverses. Keep existing MCP/provider operation meanings unchanged.
- [x] 3.2 Implement separate block planning/preflight/generation workflow using existing speech resolution/estimation/previews. Reserve token groups exclusively in existing retained owner; preserve original expiry, immutable metadata, actual alignment and complete-token reuse on failures. Cover concurrent lease/discard, expiry, cancellation, uncertainty and cleanup warnings without automatic retry.
- [x] 3.3 Add bounded persistent JSON-lines source CLI with existing provider/config/headless lifecycle and document plan/preview/commit/discard, exact limits, readiness, scalar units, manual abbreviations and source-versus-installed limitations. No new UI or provider.

## 4. Scenario verification

- [x] 4.1 Run focused Rust planning/manifest/migration/history/fault tests and architecture tests, plus canonical TS structural/lease/workflow tests. Record independent fixture correspondence for every scenario.
- [x] 4.2 Run real headless + fake provider complete multisentence workflow proving independent blocks, immutable snapshots, actual native asset integrity/audio placement, partial/cancel nonpublication, stale retry without inference, history/reopen and explicit detach. Exercise real bounded local alignment only if already available; record actual readiness without downloading models or equating fake timing with model evidence.
- [x] 4.3 Run all required local suites on stable inputs: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; from apps/agent-bridge `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`; existing hermetic Python `(cd apps/kokoro-tts && python3 -m unittest test_worker.py)` and `(cd apps/faster-whisper && python3 -m pytest test_worker.py)`; repository policy `bun test scripts`; pinned `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`. Use existing required native/platform/GUI checks and unchanged CI, with explicit evidence limitations. Full logs stay outside Git.

## 5. Ordered closure and publication obligations

These closure steps follow implementation verification; their separate states are recorded in verification.md and remote PR evidence. They are not implementation checkboxes, and future actions must never be counted as completed.

5.1 Run prearchive `moon run root:openspec-validate`; inspect complete result. Only rejection naming this own active change is expected, not a pass. Resolve all other failures; use `$openspec-verify-change`, obtain independent deep conformance review and scenario evidence before archival.
5.2 Synchronize/archive only this verified change with local skills, then pass unchanged protected Moon gate and pinned strict all-spec validation. Refresh main/PR/dependency/CI evidence and preserve all originals/backups. Push only verified source and open a draft PR; record exact head/tree and all required checks to terminal. No merge/deployment, no conflict resolution for #228/#229, no claim of their interoperability from isolated CI.

Final publication and remote CI are post-verification actions. Report actual observed results, CODEOWNER review separately, and any blocker; do not mark future checks as passed or add a source commit solely to loop CI evidence.
