## 1. Isolated inherited environment policy

- [x] 1.1 Reject any `workflow.env` and any job-level `env` on contract, render, and foundation parity, including empty maps.
- [x] 1.2 Preserve exact critical step environments, golden-mode diagnostics, and non-parity job environments.

## 2. Regression evidence and documentation

- [x] 2.1 Add negative tests for literal and expression-valued `BASH_ENV`, `PATH`, `LD_PRELOAD`, `LD_AUDIT`, `NODE_OPTIONS`, benign-looking keys, and empty inherited maps at every protected scope.
- [x] 2.2 Add positive coverage for the checked-in workflow, exact parity step environments, and an environment map on a non-parity job.
- [x] 2.3 Document the empty inherited-environment boundary and the required step-scoped approval path.
- [x] 2.4 Confirm workflow commands, IDs, names, fixtures, goldens, contracts, schemas, renderer, and persisted data remain unchanged.

## 3. Verification and closure

- [x] 3.1 Run focused policy tests, the direct validator, and the pinned OpenSpec validation task.
- [x] 3.2 Run contract, TypeScript typecheck/lint/unit, integration/smoke, and Python checks.
- [x] 3.3 Run Rust fmt, strict Clippy, and workspace tests.
- [x] 3.4 Run configured native Linux golden conformance, lifecycle, and report validation; compare fixture hashes before and after.
- [x] 3.5 Verify requirement/design/task/test/code coherence, synchronize all three deltas, and archive the change.
