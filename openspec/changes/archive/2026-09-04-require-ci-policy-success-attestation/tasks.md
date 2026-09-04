# 1. Policy completion proof

- [x] 1.1 Add the exact policy step identifier, post-success output emission, and sole job output.
- [x] 1.2 Require the aggregate to log and accept only the exact policy attestation with three successful dependency results.

# 2. Structural enforcement

- [x] 2.1 Validate the exact policy step identifier, two-line command order, and job output mapping.
- [x] 2.2 Validate the exact aggregate attestation binding and fail-closed assertion body.

# 3. Regression evidence and documentation

- [x] 3.1 Extend the pure result matrix with true, empty, false, and arbitrary attestation values.
- [x] 3.2 Add structural regressions for ignored failures, skipped or neutralized execution, and missing, altered, extra, forged, or premature outputs.
- [x] 3.3 Document why the policy result and completion attestation are both required.

# 4. Verification and closure

- [x] 4.1 Run policy tests, the direct validator, and the pinned OpenSpec task.
- [x] 4.2 Run contract, TypeScript, integration, smoke, Python, Rust, and configured native Linux parity checks.
- [x] 4.3 Confirm fixture and golden hashes remain unchanged and no public or persisted interface changes.
- [x] 4.4 Verify coherence, synchronize the repository-validation delta, and archive the change.
