## 1. Evaluator preflight and indexing

- [x] 1.1 Add missing-asset-first preflight and validate every named complexity limit before scene allocation or voiceover derivation.
- [x] 1.2 Replace per-layer transition scans with a stable endpoint index capped at 4,096 emitted facts, preserving declaration order and dual facts for equal endpoints.

## 2. Resource-binding boundary

- [x] 2.1 Return a private `EvaluatedSceneResult` with a path-free scene and separate deterministic media/font request bindings.
- [x] 2.2 Keep current render output unchanged while evaluating and discarding the corrected internal envelope until the routing milestone.

## 3. Verification coverage

- [x] 3.1 Add exact-limit and overflow tests for per-channel keyframes and transition facts, including voiceover preflight, missing-asset precedence, stable order, and equal endpoints.
- [x] 3.2 Add path/family/default font-binding tests and architecture assertions that paths and renderer-process/backend types remain outside `EvaluatedScene`.

## 4. Documentation and validation

- [x] 4.1 Update ADR 0004 and the motion-graphics implementation plan with preflight ordering, the transition-fact limit, and the resource-binding sidecar.
- [x] 4.2 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 4.3 Verify the completed change against its proposal, design, requirements, tasks, code, and tests; sync the delta into the living spec; then archive the change.
