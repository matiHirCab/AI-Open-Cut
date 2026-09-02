## 1. Evaluator preflight

- [x] 1.1 Add checked video/audio source-range validation after missing-reference validation and before complexity/output allocation, preserving the image exception and error precedence.
- [x] 1.2 Add the inclusive 10,000 positive pre-merge voiceover activity-range preflight limit without allocating the scene-level interval table.

## 2. Evaluated model and behavior

- [x] 2.1 Move merged voiceover interval ownership to `EvaluatedScene` and remove interval storage from `EvaluatedDucking` while preserving per-layer ducking settings.
- [x] 2.2 Preserve deterministic merge/order and current renderer compatibility traversal without consuming the evaluated result.

## 3. Verification coverage

- [x] 3.1 Add media source-timing boundary, overflow, image-exception, and missing-asset-precedence tests.
- [x] 3.2 Add 10,000/10,001 pre-merge range tests and shared scene-level ducking interval ownership/semantics tests.
- [x] 3.3 Extend architecture assertions to enforce scene-level interval ownership and exclude repeated interval vectors from `EvaluatedDucking`.

## 4. Architecture documentation

- [x] 4.1 Update ADR 0004 and the motion-graphics implementation plan with source preflight ordering, the activity-range limit, and linear scene-level interval storage.
- [x] 4.2 Verify the OpenSpec change against implementation, sync the approved delta to the living spec, and archive only this corrective change.

## 5. Validation

- [x] 5.1 Run `bunx @fission-ai/openspec@1.5.0 validate close-evaluated-scene-review-gaps --strict`.
- [x] 5.2 Run `cargo fmt --check --all`.
- [x] 5.3 Run workspace Clippy with warnings denied.
- [x] 5.4 Run `cargo test --workspace`.
- [x] 5.5 Run `git diff --check` and inspect final repository status.
