## 1. Reviewed contracts and specifications

- [x] 1.1 Obtain independent Sol medium concrete specification review and record delegated approval before implementation.
- [x] 1.2 Update canonical headless fixtures, MCP catalog, ownership and deliberate expanded digest for additive review support; do not include unrelated PR changes.

## 2. Canonical editor-core behavior

- [x] 2.1 Add typed preset/request options resolver with canonical bounds validation; preserve exact custom/project dimensions and existing rendering route.
- [x] 2.2 Add automated aspect/tie/limits/defaults/explicit options tests and native audio default/false, export parity, immutable/reopen and early failure evidence.

## 3. Headless and bridge consumers

- [x] 3.1 Update additive typed headless operation/defaults and capability lists plus transport fixtures/tests.
- [x] 3.2 Add MCP review schema/tool and forwarding; preserve legacy tool schema/defaults; update types/status schemas/instructions and documentation.
- [x] 3.3 Add MCP schema/contract/workflow/integration/packaged coverage, including ready/unavailable capability, missing media/paths/revisions, job reuse and audio choices.

## 4. Verification and publication

- [x] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; log full results outside repository.
- [x] 4.2 From apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, `bun run test:smoke`; run hermetic relevant worker tests according to Moon definitions.
- [x] 4.3 Run pinned `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`, protected `moon run root:openspec-validate` prearchive, independent Sol medium implementation/conformance review; fix and retest all findings.
- [x] 4.4 Test disposable semantic union with exact unmerged PR137/138 heads and recompute merged digest; document conflicts without copying unrelated changes.

## Post-verification delivery

After implementation conformance acceptance, invoke openspec-verify-change, synchronize and archive through repository skills, then require postarchive protected and strict gates to pass. Record these results in verification.md before the feature commit.

Publication follows the verified feature commit: push the scoped branch, open a draft PR, observe every exact-head CI check terminal, retain a recoverable bundle and report genuine CODEOWNER review limitations. These future delivery results belong in an external delivery ledger and PR body so recording publication does not change the tested commit. No merge or deployment is authorized. These delivery steps are administrative lifecycle gates, not completed implementation tasks.
