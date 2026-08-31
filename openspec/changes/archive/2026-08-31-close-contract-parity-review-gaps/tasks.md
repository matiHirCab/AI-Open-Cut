## 1. Rust Request Parity

- [x] 1.1 Add `strum` 0.27.2 with derive support to the workspace and headless crate dependencies, and verify `cargo check -p opencut-headless` succeeds.
- [x] 1.2 Derive snake-case variant names from the actual headless `Request` enum, remove the duplicate operation constant, and verify the Rust contract test fails when a request variant is not represented in `contracts/headless-protocol-v1.json` and passes after parity is restored.

## 2. MCP Surface Parity

- [x] 2.1 Extend the contract harness to capture every registered tool definition and normalize Zod input/output JSON Schemas plus annotations with recursively sorted object keys and unchanged array order; verify `bun run test:unit -- tests/contracts.test.ts` exercises complete definitions.
- [x] 2.2 Add canonical `toolDefinitions` to `contracts/mcp-surface-v1.json`, keep the readable tool-name list, and verify mutations to a tool input schema, output schema, or annotations each produce an MCP catalog mismatch.
- [x] 2.3 Preserve focused behavioral validator coverage for runtime refinements that JSON Schema cannot represent, and verify the existing agent-bridge unit tests pass.

## 3. Standalone Gate and Documentation

- [x] 3.1 Prepend `bun run typecheck` to `contracts:check`, add an assertion that the standalone command retains that compiler gate, and verify TypeScript-only request-union drift causes `bun run contracts:check` to fail.
- [x] 3.2 Update the contract-governance documentation for enum-derived Rust parity and full MCP tool-definition parity, and verify the documented commands and catalog ownership remain consistent with ADR 0002.

## 4. Complete Verification

- [x] 4.1 Run `bun run contracts:check` and verify Rust, TypeScript compiler, and MCP parity gates all pass.
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` and verify the Rust workspace is clean.
- [x] 4.3 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke` and verify all TypeScript and packaged-smoke checks pass.
- [x] 4.4 Run `bun run apps/agent-bridge/scripts/run-python-tests.ts` and verify the hermetic Python worker tests pass.
- [x] 4.5 Run `bunx @moonrepo/cli@2.3.3 run openspec-validate` and the OpenSpec verification workflow, synchronize the modified `contract-governance` specification, archive this change, and verify the archived change validates successfully.
- [x] 4.6 Run `git diff --check origin/main...HEAD`, inspect the final diff for unrelated files or generated artifacts, then commit and push the fixes to `feat/issue-84-contract-governance` so PR #93 updates without publishing the earlier review comments.
