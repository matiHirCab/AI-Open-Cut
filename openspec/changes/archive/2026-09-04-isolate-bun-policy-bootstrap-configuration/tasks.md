## 1. Isolate Bun policy execution

- [x] 1.1 Start the workflow bootstrap with an empty trusted Bun config and dotenv loading disabled, preserving the policy output and job identities.
- [x] 1.2 Validate the canonical `bunfig.toml`, add it to the protected source inventory and CODEOWNERS, and reject configuration mutations before Moon launch.
- [x] 1.3 Run every protected Moon-task Bun command with the validated config and dotenv loading disabled, including the real-Moon integration regression.

## 2. Regression coverage and documentation

- [x] 2.1 Add structural tests for exact workflow/task commands and canonical Bun configuration failure cases.
- [x] 2.2 Add real-Bun regressions proving preload and dotenv attacks cannot execute before preflight or inside the protected task.
- [x] 2.3 Update CI policy documentation and `repository-validation` to describe the isolated Bun boundary and mandatory Linux regression.

## 3. Verification and closure

- [x] 3.1 Run policy tests, bootstrap validation, direct validation, OpenSpec strict validation, and the real regression on Ubuntu 24.04.
- [x] 3.2 Run contracts, TypeScript typecheck/lint/unit, integration/smoke, Python, Rust fmt/Clippy/workspace, and native Ubuntu parity.
- [x] 3.3 Confirm fixture and golden hashes are unchanged and `git diff --check` is clean.
- [x] 3.4 Verify OpenSpec coherence, synchronize `repository-validation`, and archive the change.
