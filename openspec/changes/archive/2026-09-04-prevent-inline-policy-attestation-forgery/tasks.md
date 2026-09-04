# 1. Move policy attestation

- [x] 1.1 Restore the exact one-line workflow policy command without inline output emission.
- [x] 1.2 Emit the policy marker only from the final successful structural-validator command in the Moon task.

# 2. Structural enforcement

- [x] 2.1 Validate the exact Moon policy-task body and command order.
- [x] 2.2 Make validation-plus-attestation invoke its writer only after workflow and Moon configuration both pass.

# 3. Regression evidence and documentation

- [x] 3.1 Cover inline writes, masked or altered workflow commands, and missing, reordered, duplicated, or neutralized Moon attestation commands.
- [x] 3.2 Cover exact output on success and absent output after every structural failure.
- [x] 3.3 Document the full Moon task as the attested unit and the retained external trust boundary.

# 4. Verification and closure

- [x] 4.1 Run policy tests, the direct validator, and the pinned OpenSpec task.
- [x] 4.2 Run contract, TypeScript, integration, smoke, Python, Rust, and native Ubuntu parity checks.
- [x] 4.3 Confirm fixture and golden hashes remain unchanged and no public or persisted interface changes.
- [x] 4.4 Verify coherence, synchronize the repository-validation delta, and archive the change.
