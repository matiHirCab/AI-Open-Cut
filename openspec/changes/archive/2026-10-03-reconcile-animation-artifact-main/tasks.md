## 1. Specification and authorization

- [x] 1.1 Obtain actual independent delegated specification review and record its outcome without claiming human CODEOWNER approval.

## 2. Reconciliation and verification

- [x] 2.1 Preserve reviewed issue47 bundles/head and normally merge exact main4c2897e0; resolve actual overlaps only and keep imported issue59/74 archive/living specifications intact.
- [x] 2.2 Independently review both-parent semantic union, unchanged issue47 test partitions/shallow-copy setup, artifact delivery/security coverage, ownership consumers and canonical catalog; deliberately recompute the MCP digest using the existing parity procedure.
- [x] 2.3 From apps/agent-bridge run `bun run contracts:check` including artifact-responses.test.ts, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration` and `bun run test:smoke`; record exact results/logs and investigate every failure without weakening checks.
- [x] 2.4 Record exact Rust/headless/Python/native source/fixture/toolchain equality before reusing prior evidence. If invalidated, run affected mandatory checks from repository root: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `bun run apps/agent-bridge/scripts/run-python-tests.ts` and the previously recorded native6/7 and golden/cache commands corresponding to changed inputs; do not substitute an unaffected passing suite for affected evidence.
- [x] 2.5 From repository root run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and pre-archive `moon run root:openspec-validate`; use openspec-verify-change and independent implementation review, then openspec-sync-specs/openspec-archive-change. Active-change-only pre-archive rejection is expected, not a passed gate; post-archive gates remain pending in the delivery checklist.
