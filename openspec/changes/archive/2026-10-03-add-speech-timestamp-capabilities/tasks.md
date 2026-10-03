## 1. Planning and approval

- [x] 1.1 Obtain independent spec review and resolve findings; record issue-scoped delegated approval and narrow shared sections.

## 2. Governed contract and consumers

- [x] 2.1 Extend canonical speech status fixture and legacy/negative cases, MCP status output catalog and deliberate digest, and narrow speech ownership entry.
- [x] 2.2 Add strict speech timestamp metadata schema, conservative legacy normalization, provider input typing, and typed malformed-status failure at bridge/service boundaries.
- [x] 2.3 Advertise truthful all-false metadata in Kokoro worker and unavailable fallback; document discovery/compatibility semantics.
- [x] 2.4 Cover every added scenario with provider/service/adapter tests and cross-language canonical parity evidence.

## 3. Review and verification

- [x] 3.1 Obtain independent implementation review and fix findings.
- [x] 3.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` from root.
- [x] 3.3 Run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke` from apps/agent-bridge; run `bun run apps/agent-bridge/scripts/run-python-tests.ts` from root.
- [x] 3.4 Run pinned strict all-spec validation and pre-archive `moon run root:openspec-validate`; verify conformance with openspec-verify-change and record complete evidence.

## 4. Finalization after implementation verification

Synchronize and archive only this verified change, then rerun strict all-spec validation and protected `moon run root:openspec-validate`. Create a verified commit and preserved bundle, push the issue branch, create a draft PR with CODEOWNER review request, and inspect exact-head CI through terminal. Record finalization and publication evidence separately; these steps follow implementation conformance verification and do not waive any required final check.

## 5. Windows CI fixture correction

- [x] 5.1 Replace generated workers under the teardown root with a test-only environment override in the existing fake worker; preserve every assertion and timeout.
- [x] 5.2 Independently review the regression correction and run affected typecheck/lint/unit/integration/smoke checks.
- [x] 5.3 Record the original Windows failure, verify unchanged requirements and fixture parity, and run the expected pre-archive gate. Follow the finalization steps in section 4 again before the correction commit.

## 6. Authorized main reconciliation

- [x] 6.1 Integrate main `5e6472a1` with a normal merge, preserve both accepted catalog/schema sections, and recompute only the pinned combined digest.
- [x] 6.2 Obtain independent reconciliation review and pass full required implementation/conformance checks.
Finalization after reconciliation: preserve synchronized requirements, rearchive, pass protected gates, recheck main and record shared sections/merge order before verified push and exact-head CI.
