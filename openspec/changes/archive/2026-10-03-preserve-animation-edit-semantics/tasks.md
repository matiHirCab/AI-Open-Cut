## 1. Approval and canonical contract

- [x] 1.1 Obtain independent specification review and record delegated approval without claiming human CODEOWNER review.
- [x] 1.2 Add bounded clock model/fixtures, schema31 and fail-closed old/reserved/future migration; cover root/component/current/all-history atomicity.

## 2. Core editing and evaluation

- [x] 2.1 Preserve source clocks/keys through split/trim/duplicate, replacement reset and marker lifecycle; cover all supported item/value/curve/loop types and errors.
- [x] 2.2 Apply retained clocks exactly once to scene scalar/compound/legacy/visibility evaluation, inherited fractional clocks, render artifacts/planner and audio gain; retain complexity validation.
- [x] 2.3 Add independent before/after interior/seam/turn/exhaustion, alias rollback, stale/locked/missing/bounds, undo/redo/reopen and native renderer parity tests.

## 3. Consumers and documentation

- [x] 3.1 Synchronize bridge channel/project schemas, headless/MCP/capability catalogs and all governed consumers; document reserved30 integration adapter and source clocks.
- [x] 3.2 Run focused Rust animation-edit tests and canonical cross-language parity.

## 4. Verification and completion

- [x] 4.1 Run cargo fmt --all -- --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace; bridge bun run typecheck/lint/test/contracts:check; MCP integration and packaged smoke; relevant hermetic Python worker tests. Record exact commands and logs from the repository task definitions and focused tests.
- [x] 4.2 Run pinned bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive and moon run root:openspec-validate before archive, recording only-this-change expected rejection separately.
- [x] 4.3 Use openspec-verify-change with inspected code/test conformance and independent implementation review; resolve findings.
- [x] 4.4 Coordinate with root to synchronize/archive the verified change using repository skills; final protected/all-spec gates remain the post-archive delivery gate below.
