## 1. Regression and Extractor

- [x] 1.1 Add failing regression cases for direct, grouped, nested, multiline, and aliased crate-local owner imports, plus similarly-prefixed non-owner identifiers.
- [x] 1.2 Implement a test-local import dependency extractor and route the owner-matrix assertion through its extracted dependency set.
- [x] 1.3 Verify diagnostics still identify the importing owner, forbidden dependency, and allowed set.

## 2. Verification

- [x] 2.1 Run `cargo fmt --check --all`, `cargo test -p opencut-editor-core --test architecture --no-fail-fast`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 2.2 Run `cargo test --workspace`, the bridge typecheck/lint/unit/integration/smoke commands, and strict OpenSpec validation.
- [x] 2.3 Run `$openspec-verify-change` and `$code-review` against PR #94, resolve every actionable mismatch, archive the change, then create and push one follow-up commit without including `docs/motion-graphics-implementation-plan.md`.
