# 1. Isolate Moon policy execution

- [x] 1.1 Qualify the workflow target and isolate the root project from inherited tasks.
- [x] 1.2 Include every relevant Moon configuration source in the protected task inputs.

# 2. Enforce the effective configuration boundary

- [x] 2.1 Validate exact root project, workspace, and toolchain configuration.
- [x] 2.2 Discover global task configuration and reject environment injection, external extension, or unsupported sources before attestation.

# 3. Regression evidence and documentation

- [x] 3.1 Cover root environment and execution overrides, root redirection, toolchain mutation, inherited task injection, and output suppression.
- [x] 3.2 Preserve positive coverage for non-root global tasks and the exact qualified workflow command.
- [x] 3.3 Document the effective Moon configuration as part of the attested unit.

# 4. Verification and closure

- [x] 4.1 Run policy tests, the direct validator, and pinned OpenSpec validation.
- [x] 4.2 Run contract, TypeScript, integration, smoke, Python, Rust, and native Ubuntu parity checks.
- [x] 4.3 Confirm fixture and golden hashes remain unchanged and no public or persisted interface changes.
- [x] 4.4 Verify coherence, synchronize the repository-validation delta, and archive the change.
