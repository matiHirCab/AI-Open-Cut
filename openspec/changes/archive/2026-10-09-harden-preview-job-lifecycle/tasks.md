## 1. Implementation
- [x] 1.1 Add settled producer accounting and cancellation/close regression evidence (L1, L2).
- [x] 1.2 Add bounded serialized preview retention and confined disposable output I/O with boundary/failure tests (L4-L6).
- [x] 1.3 Confirm one-shot termination before cleanup and prove overflow isolation/late-write negative path (L3).
- [x] 1.4 Execute actual native source/package review and stale-revision/edit/undo/redo/reopen evidence; document unchanged contracts and operational limits (L2-L6).
## 2. Verification and delivery
- [x] 2.1 Pass required formatting, strict Clippy, workspace tests, bridge type/lint/unit/contract/integration/smoke, applicable Python and strict specs; inspect pre-archive protected result.
- [x] 2.2 Verify conformance/scenario traceability and synchronize accepted specs for archival.

## Delivery follow-through

Archive the verified change, require the final protected Moon gate and strict all-spec validation to pass, then commit/push the verified branch and publish a draft PR targeting main; record exact-head checks and remaining platform limits. This delivery step follows implementation completion and is not an archival prerequisite.

Required commands: `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; bridge `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`; `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`. Hermetic Python worker tests cover unchanged provider integration. Configured native worker/cache test requires actual FFmpeg/FFprobe and the declared font with instrumented headless; no skip counts as native evidence.

