## 1. Approval and original failure evidence

- [x] 1.1 Obtain explicit approval of this proposal, delta specs, design and tasks under AGENTS.md before implementation.
- [x] 1.2 Preserve the full Library audit, both exact-main reproduction sources/results and initial failure logs; add baseline regression assertions that fail against unchanged current main.

## 2. Core native artifact ownership

- [x] 2.1 Add narrowly scoped editor-core native handle-anchored preview disposal with strict owned UUID/file-role validation, safe errors, idempotent missing files and Linux/macOS/Windows ancestor replacement protection; no public/persisted contract change.
- [x] 2.2 Add genuine native core regression tests for original ancestor replacement, project/previews/root swaps, final-file links/reparse points, Unicode/trusted-parent aliases, missing files and unrelated-file preservation; test handle cleanup and platform sharing refusal.

## 3. Private headless adapter and bridge budgeting

- [x] 3.1 Add a bounded private stdin-only cleanup mode inside the existing headless binary and bridge adapter, preserving public Request/MCP discovery and the four-role default package; test malformed/oversized input, termination/error redaction and actual native invocation.
- [x] 3.2 Reserve unsettled preview producer slots before dispatch, settle required eviction before replacement, block production while charged disposal debt remains, and retain existing inclusive successful count/byte budgets and independent nonpreview work.
- [x] 3.3 Make cleanup failures observable and close retryable without reopening admission or dropping ownership; keep cancellation/TTL/noncancellable phases and stale revision semantics unchanged.
- [x] 3.4 Add bridge regressions for both original reproductions, sequential retries and persistent disposal errors, overlapping producers/reservations, count/byte boundaries, cancelled/oversized debt, failed/retried close and safe cleanup errors.

## 4. Cross-platform packaged/native acceptance

- [x] 4.1 Extend existing source/default-package native verification to execute actual confined cleanup and report required genuine supported-platform results without adding a runtime role or weakening existing commands/assertions/deadlines.
- [x] 4.2 Update narrowly scoped lifecycle/operational docs and scenario-to-test evidence; preserve protocol1/schema44, catalogs, provider contracts, frozen references and review media semantics.
- [x] 4.3 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` and `cargo test --workspace`; run focused native cleanup/private adapter tests and actual source/default-package native workflows. Record all failures/blocked checks with complete logs.
- [x] 4.4 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`; run `python3 scripts/test_setup_platform_renderer.py`, `python3 scripts/test_setup_motion_release_backends.py` and relevant hermetic provider tests. Preserve exact regression/media/lifetime bounds.
- [x] 4.5 Run the pre-archive protected gate, accept only this active-change rejection, verify conformance with openspec-verify-change and prepare synchronization/archival of approved deltas. Post-archive strict/protected gates remain mandatory delivery acceptance below.

## 5. Post-archive delivery acceptance (tracked externally against the exact published head)

1. Synchronize and archive the verified approved deltas; require `moon run root:openspec-validate` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` to pass after archival.
2. Inspect the final tracked diff and scenario evidence, commit/push the scoped branch and open a draft PR against current main with failures, evidence reuse and limitations disclosed; do not merge/deploy.
3. Require terminal exact-final-head standard, default-package native, complete-release and applicable focused CI; repair scoped recoverable failures, rerun affected checks and update the draft PR/evidence until terminal acceptance or an explicit external blocker.

Approval: explicit delegated reviewer approval on 2026-10-10 covering proposal, design, seven acceptance criteria and constraints. User authorization permits implementation, verification and draft PR; no merge/deploy.

Delivery and terminal exact-head CI are post-archive acceptance obligations. They remain pending and will be recorded in the draft PR/external evidence without changing the verified commit merely to record its CI result. Platform execution is pending until genuine CI succeeds; local Windows cross-compilation is supplemental evidence only.
