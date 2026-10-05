## 1. Approved conformance correction

- [x] 1.1 Obtain explicit parent approval after promoted strict-validated specification review; record approval before test edits.
- [x] 1.2 Correct the single readiness-aware protocol test to validate boolean readiness, exact canonical editor/rendering/top-level lists, error semantics and unique capability iff ready; require ready under OPENCUT_GOLDEN_REQUIRED=1 and retain the explicit missing health test.

## 2. Required verification

- [x] 2.1 Execute controlled nonexistent FFmpeg/FFprobe protocol test with required native mode unset; run actual configured FFmpeg/FFprobe/reviewed font with OPENCUT_GOLDEN_REQUIRED=1 and the explicit missing health regression, through reap.py and external logs.
- [x] 2.2 Run cargo fmt --check --all, cargo clippy --workspace --all-targets -- -D warnings and complete unfiltered cargo test --workspace via reap.py with activation/native linking; run full bun run contracts:check from apps/agent-bridge. Record exact exit/count evidence and distinguish reused unaffected native/TS/Python checks under unchanged-input policy.
- [x] 2.3 Run pinned bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive; document any protected prearchive rejection as active-change lifecycle evidence, never a passed gate.

## 3. Independent review and archival

- [x] 3.1 Apply openspec-verify-change and independent code/conformance review, map all scenarios and resolve findings before archival.
- [x] 3.2 Synchronize/archive only fix-linear-capability-contract-readiness; run final unchanged moon run root:openspec-validate and pinned all-spec strict validation. Parent owns commit/push/restack; preserve stashed #50 work.
