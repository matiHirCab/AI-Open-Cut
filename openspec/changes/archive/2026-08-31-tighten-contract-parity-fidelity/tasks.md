## 1. Rust Wire-Tag Fidelity

- [x] 1.1 Add a test helper that verifies derived operation names are accepted by the actual Serde request deserializer, and verify a test-only renamed enum demonstrates the mismatch failure mode with `cargo test -p opencut-headless`.
- [x] 1.2 Apply the helper to every production `Request::VARIANTS` entry while retaining the canonical catalog comparison, and verify `cargo test -p opencut-headless capability_sets_match_the_canonical_headless_contract` passes.

## 2. MCP Compatibility Projection

- [x] 2.1 Add schema-specific recursive normalization that omits only `description`, sorts every remaining object key, and preserves array order; verify focused TypeScript tests cover nested descriptions and structural differences.
- [x] 2.2 Keep annotation normalization lossless and confirm the checked-in MCP catalog remains unchanged by running `git diff --exit-code -- contracts/mcp-surface-v1.json` after the focused contract test.

## 3. Documentation and Complete Verification

- [x] 3.1 Update ADR 0002 and contract-governance guidance to describe Serde tag recognition and description-only schema exclusion, and verify OpenSpec strict validation passes with `bunx @moonrepo/cli@2.3.3 run openspec-validate`.
- [x] 3.2 Run `bun run contracts:check`, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 3.3 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke`.
- [x] 3.4 Run `bun run apps/agent-bridge/scripts/run-python-tests.ts` and verify all hermetic Python worker tests pass.
- [x] 3.5 Run the OpenSpec verification workflow, synchronize `contract-governance`, archive this change, and validate the synchronized living spec.
- [x] 3.6 Run `git diff --check origin/main...HEAD`, commit only the follow-up and previously preserved PR files, push `feat/issue-84-contract-governance`, and verify PR #93 CI without publishing review comments.
