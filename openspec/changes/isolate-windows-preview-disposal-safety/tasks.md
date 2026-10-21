## 1. Current evidence and approval

- [x] 1.1 Read repository instructions, OpenSpec workflow, relevant living requirements and prior disposal design; preserve unrelated work and create branch from freshly fetched main8cb21a6b.
- [x] 1.2 Check current PR/issue ownership through the authorized connector; retrieve original Windows failure and distinguish observations from hypotheses. No rerun requested.
- [x] 1.3 Obtain explicit user/reviewer approval of proposal, design, delta specification and this task list before executable edits. Delegated reviewer in parent thread01a1583b-f8d9-70ae-8c32-75210bced0e5 explicitly approved review diff SHA2562d46814469057528303e89cb32695e044f7e38f7f1d3614ae2e487db49a0747e on2026-10-21, then authorized publishing locally checked diagnostics as an investigation draft before Windows verification. No production change is approved.

## 2. Agent-bridge private acceptance controls

- [x] 2.1 Add isolated fixture-only and native-only controls plus phase-specific sentinel assertions and runtime/replacement observations; retain original race, twelve calls, foreign/reparse safety and close/debt checks. Diagnostic scenarios covered; conditional correction scenario remains pending2.3. Linux focused23/23 and actual native integration selection1/1 passed; Windows remains pending.
- [ ] 2.2 Execute controls on supported Windows CI with source and packaged native binaries; retain exact SHA/runtime and decisive attribution evidence. Do not rerun unchanged acceptance until green.
- [ ] 2.3 Only if the fixture-only control establishes target traversal, correct junction removal consistently with a nonrecursive primitive, cover preservation and teardown failures, and retain native assertions. Otherwise update concrete design/approval before changing production code.

## 3. Required verification and delivery

- [ ] 3.1 With pinned tools, run `cargo build -p opencut-headless`, `cargo test -p opencut-editor-core preview_disposal`, `cargo test -p opencut-editor-core --test preview_disposal_windows`, and `cargo test -p opencut-headless --test preview_disposal`. Windows-only tests executing zero cases on Linux are not Windows evidence.
- [ ] 3.2 From apps/agent-bridge run `bun install --frozen-lockfile`, focused `bun x --no-install vitest run --config vitest.unit.config.ts tests/preview-disposal.test.ts tests/preview-lifecycle.test.ts` plus added diagnostic cases; `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`.
- [ ] 3.3 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and hermetic Python `python3 -m unittest discover -s apps/kokoro-tts -p 'test*.py'` and `python3 -m pytest apps/faster-whisper` with their required isolated dependencies. Record any failure or unavailable check explicitly.
- [ ] 3.4 Run pinned `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; run `moon run root:openspec-validate` before archival and inspect full output. Rejection naming only this active change is expected, not success.
- [ ] 3.5 Invoke `$openspec-verify-change`; resolve conformance gaps; synchronize via `$openspec-sync-specs` and archive via `$openspec-archive-change` only after mandatory checks pass. Run protected Moon gate and strict all-spec validation again after archival.
- [ ] 3.6 Commit/push verified scope and open a draft PR with exact-head Windows source/package acceptance evidence or explicit pending limits. No merge/deploy; no #228/#229 changes.

Reviewer sequencing clarification: diagnostic publication may precede Windows verification after appropriate local checks. Label the draft as investigation with Windows validation pending; retain the active change and expected policy-gate rejection. This does not complete3.6 or establish a verified repair.
