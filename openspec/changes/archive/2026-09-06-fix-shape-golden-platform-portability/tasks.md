## 1. Approval and regressions

- [x] 1.1 Obtain explicit approval of proposal, design, delta scenarios and tasks before editing the comparison or tests.
- [x] 1.2 Add failing core golden regressions for the three measured platform pairs, tracing to Accept measured platform contour variation. Preserve the failed CI evidence and checksum correction.

## 2. Scoped golden comparison

- [x] 2.1 Implement test-only shape stored-reference comparison scoped to generated contour coordinates, enforcing both 8 ULPs and 1e-12 absolute difference. Preserve exact same-runtime plans and all other semantic fields.
- [x] 2.2 Cover Reject semantic drift and excessive numeric variation and Fail closed on unsupported coordinates and structure: 8/9 ULP boundaries, absolute cap, negative and signed-zero values, non-finite values, malformed/missing/extra fields, authored points and unrelated semantic mutations. Confirm RGB/reference plan bytes and production code remain unchanged.

## 3. Validation

- [x] 3.1 Run focused new regressions, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Record exact focused commands and results in verification.md.
- [x] 3.2 Run `cargo test --release -p opencut-editor-core native_golden_render_conformance --lib -- --nocapture` with OPENCUT_GOLDEN_REQUIRED=1 and configured FFmpeg, FFprobe and deterministic font; trace to Preserve independent golden evidence.
- [x] 3.3 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`; from apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts`. Preserve contracts and fixture parity.

## 4. Completion and PR

- [x] 4.1 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `git diff --check`. Use openspec-verify-change; resolve every discrepancy among requirements, design, tasks, tests and code.
- [x] 4.2 Synchronize and archive with openspec-sync-specs and openspec-archive-change, then run `bunx @moonrepo/cli@2.3.3 run root:openspec-validate`. Push the verified correction to PR #114 and confirm hosted render/foundation parity plus remaining required CI checks pass. Report any failed or skipped required check as blocking completion.
