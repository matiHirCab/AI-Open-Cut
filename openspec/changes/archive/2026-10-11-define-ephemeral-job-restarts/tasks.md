## 1. Contract and decision
- [x] 1.1 Inspect issue/comments, closed #15/#73, clean current main, instructions, living specs and measured existing job/startup/failure/retry behavior.
- [x] 1.2 Add canonical ephemeral capability/behavior fixture and ownership; write ADR and safe client/operator retry/reconciliation guidance.
## 2. Bridge implementation
- [x] 2.1 Add exclusive private crash admission guard; wire job producer admission, truthful provider cleanup-debt failure/retry and successful shutdown/doctor release without changing canonical job envelopes/errors or adopting files.
- [x] 2.2 Advertise the additive bridge capability in editor status and CLI health, with canonical parity and source/packaged smoke evidence.
## 3. Conformance
- [x] 3.1 Add actual crash/restart tests for admitted queued and provider-waiting work, running/completed/failed/cancelled states, repeated-crash bound, fresh IDs and private resources; guard isolation/unsafe/replaced marker/cleanup-failure/graceful lifecycle tests.
- [x] 3.2 Pass `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; bridge `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`; `bun run apps/agent-bridge/scripts/run-python-tests.ts`; focused restart/cancellation/retention/isolation tests.
- [x] 3.3 Run strict all-spec validation and pre-archive `moon run root:openspec-validate`; verify conformance with explicit evidence before sync/archive.
## Delivery follow-through after verified archival

Run post-archive `moon run root:openspec-validate` and strict all-spec validation, commit/push the verified issue-scoped tree, open a draft PR requesting CODEOWNER review, verify exact-head CI and address in-scope failures. Report next dependency-ready issue candidates for parent coordination before selecting one. The verification report records prepublication conformance; the issue-linked draft PR and external checkpoint record publication and exact-head CI evidence when available.
