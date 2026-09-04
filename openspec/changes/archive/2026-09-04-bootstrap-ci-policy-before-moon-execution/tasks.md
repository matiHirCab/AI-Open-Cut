## 1. Bootstrap the protected policy job

- [x] 1.1 Install explicit Bun and Moon bootstrap versions before checkout and replace the direct Moon workflow command with the exact policy bootstrap.
- [x] 1.2 Implement pre-execution validation, shell-free Moon spawning, child output-channel isolation, and post-success attestation.
- [x] 1.3 Make the structural validator pure and require exact Moon, workspace, toolchain, proto, workflow, and bootstrap configuration.

## 2. Regression coverage and review ownership

- [x] 2.1 Adapt all existing policy tests and cover valid bootstrap execution, failure paths, output custody, exact workflow structure, and proto mutations.
- [x] 2.2 Add an isolated real-Moon regression for the `BASH_ENV` forgery and prove that the bootstrap blocks it before child execution.
- [x] 2.3 Protect every CI-policy source with CODEOWNERS and document the pre-execution boundary, local reproduction, and personal-repository trust limitation.

## 3. Verification and closure

- [x] 3.1 Run policy tests, bootstrap validation, the direct validator, and `moon run root:openspec-validate`.
- [x] 3.2 Run contracts, TypeScript typecheck/lint/unit, integration/smoke, Python, Rust fmt/Clippy/workspace tests, and native Ubuntu 24.04 parity.
- [x] 3.3 Confirm fixture and golden hashes are unchanged and `git diff --check` is clean.
- [x] 3.4 Verify OpenSpec coherence, synchronize `repository-validation`, and archive the change.
