## 1. Provider lifecycle
- [x] 1.1 Add real subprocess regression tests for cancellation/timeout with queued and immediate mixed-mode successors, and reproduce against the base.
- [x] 1.2 Isolate worker generations and retire/reap before successor inference; preserve typed failures and FIFO concurrency.
- [x] 1.3 Document/test graceful direct close and supported bridge shutdown during cancellable inference.

## 2. Required native evidence
- [x] 2.1 Add exact integrated narration invocation to configured workflow and policy command set.
- [x] 2.2 Add omission/masking negative policy tests; run `bun test scripts/validate-ci-gates.test.ts`.

## 3. Complete MCP resource verification
- [x] 3.1 Profile source/package schema compilation and validation independently of production processes and record evidence.
- [x] 3.2 Implement only the demonstrated scoped allocation correction; preserve full validation with negative regression tests.
- [x] 3.3 Run unchanged complete `bun run test:integration` and `bun run test:smoke`, recording observed resource evidence and all outcomes.

## 4. Verification and delivery
- [x] 4.1 Run `bun run typecheck`, `bun run lint`, `bun run test:unit`, relevant hermetic Python checks, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`.
- [x] 4.2 Run configured exact integrated native narration test and predecessor lifecycle/cache checks affected by main lineage, preserving assertions and thresholds.
- [x] 4.3 Run pinned strict OpenSpec validation and pre-archive `moon run root:openspec-validate`; verify complete requirement/design/test traceability with openspec-verify-change.
## Delivery follow-through

After all implementation tasks pass: synchronize/archive verified deltas, rerun the protected gate and strict all-spec validation, then commit/push the ordinary repair branch and open a draft PR targeting main. If executor API access is blocked, provide exact SHA/title/body for the parent's authorized connection. These procedural steps remain tracked in the verification/delivery report until actually completed; no merge or deploy is authorized.
