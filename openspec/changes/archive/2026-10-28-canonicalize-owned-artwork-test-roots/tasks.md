## 1. Review

- [x] 1.1 Obtain exact-artifact approval; freeze originals/hashes before status edits; refresh PR229/main. Preserve97db1db3 macOS and restricted Windows failures. No executable edits before approval.

## 2. Private repair

- [x] 2.1 Add cross-platform ancestor-alias PNG/WAV positives and allowlisted alias/foreign/linked-child negatives. Demonstrate positive failure before repair. Retain existing negatives; Windows directory junctions; no new skips.
- [x] 2.2 Canonicalize created roots in both approved test producers before paths/config/hash keys. Probe/shared backend/runtime stay verbatim; no refactor or relaxed validation.

## 3. Verify/archive

- [x] 3.1 Under existing child reaper run bridge `bun run typecheck`, `bun run lint`, `bun x --no-install vitest run --config vitest.unit.config.ts tests/artwork-durable-backends.test.ts`, `bun run test:unit --maxWorkers=1`, `bun run contracts:check --maxWorkers=1`, `bun run test:integration`, `bun run test:smoke`. Record limits/failures. Prove combined source/assembled package with aliased TMPDIR and interpreter-only PATH lacking media tools.
- [x] 3.2 With OPENCUT_ARTWORK_DURABLE_GENUINE=1 and explicit absolute genuine tools run `bun x --no-install vitest run --config vitest.integration.config.ts -t 'combines artwork and narration'` and full `bun run test:smoke`; retain64x64 decode/common assertions. Reuse Rust/worker/artwork evidence only after unchanged-input/toolchain audit.
- [x] 3.3 Root `bun test scripts`; `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; `moon run root:openspec-validate`. Only this active-change pre-archive rejection is expected. Verify conformance; sync/archive only this change after mandatory passes; require final protected/strict success.

## 4. Publish

- [ ] 4.1 Push verified descendant to existing PR229; observe new-head CI to terminal and report failures/skips. No force/readiness change, reruns-until-green, PR233/native disposal edits, denied description/comments, approval, merge or deployment.
