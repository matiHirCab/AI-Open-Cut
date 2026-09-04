## 1. Aggregate policy propagation

- [x] 1.1 Add OpenSpec validation as the third direct aggregate dependency and exact result binding.
- [x] 1.2 Require the aggregate to log and accept only three `success` results.

## 2. Policy-job enforcement

- [x] 2.1 Validate the exact OpenSpec job identity, runner, steps, commands, properties, and order.
- [x] 2.2 Reject ignored failures, inherited environment, run defaults, containers, custom shells, custom conditions, and added or altered policy steps.

## 3. Regression evidence and documentation

- [x] 3.1 Expand the pure result matrix to all 64 policy/contract/render terminal combinations.
- [x] 3.2 Add structural tests for missing or altered policy dependencies, bindings, commands, and policy-job bypasses.
- [x] 3.3 Prove a policy-detected `GITHUB_ENV` mutation makes the aggregate fail even when both parity leaves succeed.
- [x] 3.4 Document that the aggregate covers structural policy plus both functional parity boundaries.

## 4. Verification and closure

- [x] 4.1 Run policy tests, the direct validator, and the pinned OpenSpec task.
- [x] 4.2 Run contract, TypeScript, integration, smoke, Python, Rust, and configured native Linux parity checks.
- [x] 4.3 Confirm fixture and golden hashes remain unchanged and no public or persisted interface changes.
- [x] 4.4 Verify coherence, synchronize the repository-validation delta, and archive the change.
