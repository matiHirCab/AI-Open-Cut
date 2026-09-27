## 1. Approval and baseline

- [x] 1.1 Record explicit user or reviewer approval of this proposal, delta spec, design, and task list after `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` passes; do not edit implementation before approval.
- [x] 1.2 Reconfirm the pre-change catalog's 19,998,586-byte, 360,953-line baseline, 72 tools, version 1, and `JSON.stringify` SHA-256 `2b5c1a5d6c0f74ca80f2c2e813612c7a3d8edcd7bd31ed186c968d3131b40752` before the rewrite.

## 2. Canonical cross-language contract artifact

- [x] 2.1 Rewrite `contracts/mcp-surface-v1.json` with two-space formatted, meaningfully named local `$defs` and exact-subtree `$ref` substitutions. Preserve all expanded schemas, annotations, identifiers, resource templates, and version; record final bytes and lines.
- [x] 2.2 Update contract-governance guidance to explain the compact storage format and deliberate digest updates for later approved public contract changes without changing ADR 0002 ownership or CODEOWNER rules.

## 3. TypeScript parity consumer and scenario tests

- [x] 3.1 Add a strict TypeScript catalog expander used by `apps/agent-bridge/tests/contracts.test.ts`; reject missing, cyclic, malformed, non-local, sibling-bearing, and unused references before comparison.
- [x] 3.2 Test deterministic expansion and the exact baseline digest; test referenced structural drift, direct per-tool input/output/annotation drift, and tool/prompt/resource/capability identifier drift. Keep the complete live-registration comparison and description-only exclusion.

## 4. Verification and closure

- [x] 4.1 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`; run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the repository's hermetic `bun run apps/agent-bridge/scripts/run-python-tests.ts` from the root. Capture full logs outside the repository and report any failure or skipped required check. The integration suite may use a test-only timeout override when the host exceeds its default 60 seconds.
- [x] 4.2 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and pre-archive `moon run root:openspec-validate`; treat only an active-change rejection as expected, and resolve every other failure. Use `$openspec-verify-change` to reconcile requirements, design, tasks, tests, and implementation.
- [x] 4.3 Obtain `@matiHirCab` CODEOWNER review of the verified catalog and consumer change, then use `$openspec-sync-specs` and `$openspec-archive-change`. Rerun strict all-spec validation and `moon run root:openspec-validate` after archival; completion requires both to pass.
