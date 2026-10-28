## 1. Approval

- [x] 1.1 Obtain exact-artifact approval and freeze hashes before executable edits. Preserve fc48d575/failed job122835122356; refresh PR229/main ownership/refs.

## 2. Test-only correction

- [x] 2.1 Implement configured hermetic backends and bounded owned PNG/WAV probe; preserve shared fake backend/runtime. Regress unknown/malformed/changed/out-of-root/missing-tool failures and absent ambient media tools.
- [x] 2.2 Add explicit genuine mode with explicit real paths; retain all genuine decoding/common assertions. No readiness skip, timeout/profile/workflow change.

## 3. Verify

- [x] 3.1 Run bridge `bun run typecheck`, `bun run lint`, `bun run test:unit --maxWorkers=1`, `bun run contracts:check --maxWorkers=1`, `bun run test:integration`, `bun run test:smoke` under corrected child launcher. Disclose worker limits. Test hermetic combined source/package with owned PATH supplying interpreters but no media tools.
- [x] 3.2 With OPENCUT_ARTWORK_DURABLE_GENUINE=1 and explicit OPENCUT_FFMPEG_PATH/OPENCUT_FFPROBE_PATH run `bun x --no-install vitest run --config vitest.integration.config.ts -t 'combines artwork and narration'` and `bun run test:smoke`; retain separate genuine evidence. Reuse prior Rust/worker/artwork evidence only if inputs/toolchain remain unchanged; rerun affected checks otherwise.
- [x] 3.3 Run root `bun test scripts`, `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`, `moon run root:openspec-validate`; only this active-change rejection is expected pre-archive. Verify conformance, sync/archive only after mandatory passes, then require protected/strict success.

## 4. Deliver

- [ ] 4.1 Commit/push verified descendant to PR229 without force or readiness change; observe unchanged exact-head CI to terminal and report failures/skips/pending limits. No unchanged rerun-until-green, merge/deploy/approval/closure, denied description update or comments.
