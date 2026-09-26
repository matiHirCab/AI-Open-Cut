## 1. Duration policy and evidence

- [x] 1.1 Add a pure, testable duration and exception evaluator that fails closed on missing/invalid run timestamps, negative elapsed time, malformed or expired exceptions, and caps above 135 minutes; cover 120- and 135-minute boundaries.
- [x] 1.2 Update the protected workflow so the final foundation status waits for OpenSpec, contract, render, rules-screen, correctness, and packaged-smoke results, checks every result, and runs the duration assertion last even after an earlier failure. Use read-only Actions API access and log measured duration, effective budget, and complete exception justification.
- [x] 1.3 Set explicit job timeouts for protected jobs and document the measurement boundary, observed baseline, dated exception, failure behavior, and renewal/removal process in `docs/ci-parity-gates.md`.

## 2. Fail-closed workflow governance

- [x] 2.1 Extend `scripts/validate-ci-gates.ts` to pin the new prerequisite set, exact assertion order and failure propagation, read-only token scope, timeout bounds, and exception shape and expiry.
- [x] 2.2 Add adversarial `scripts/validate-ci-gates.test.ts` cases for omitted prerequisite, bypassed/ignored duration assertion, altered API source, missing or expired justification, oversized cap, removed timeout, and weakened existing parity assertions.

## 3. Verification and lifecycle

- [x] 3.1 Run focused duration and CI-policy tests, `bun --config=bunfig.toml --no-env-file run scripts/validate-ci-gates.ts`, and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; record exact commands and full logs.
- [x] 3.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke` from `apps/agent-bridge` as applicable, `bun run apps/agent-bridge/scripts/run-python-tests.ts`, and `moon run root:openspec-validate`; report any environment-limited check explicitly. No product contract or migration fixture changes are expected.

- [x] 3.3 Verify every changed scenario against the workflow, tests, and observed timing via `$openspec-verify-change`; resolve all conformance findings before synchronizing or archiving.

Pre-archive policy/Moon rejects only the active `bound-protected-ci-duration` change, as required by `docs/spec-driven-development.md`. The ordinary sandboxed Windows bridge unit run timed out on child-process tests; the same command passed with normal subprocess permissions (415 passed, 1 skipped). Final post-archive policy/Moon and remote protected CI remain pending.

After pre-archive verification, synchronize and archive the accepted delta, rerun strict OpenSpec and protected Moon, obtain designated CODEOWNER review, and require the final protected CI status to pass with elapsed time and exception justification printed. These post-archive gates remain mandatory under `AGENTS.md` and `docs/spec-driven-development.md` even though they cannot be pre-archive implementation checkboxes.
