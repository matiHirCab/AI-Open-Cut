# Verification: fix-shape-golden-platform-portability

## Approval and failing reproduction

The user approved all concrete artifacts before comparison/test edits. `cargo test -p opencut-editor-core shape_reference_plan_accepts_measured_platform_contours --lib -- --nocapture` failed under the existing exact comparison after inserting the three observed Linux contour values into the stored Windows plan. It now passes with the approved bounded comparison.

## Requirement and scenario mapping

The Portable shape semantic references requirement is implemented only in `crates/editor-core/src/renderer/golden/shapes.rs`:

- `compare_shape_reference_plan` checks all non-coordinate lines exactly and applies both <=8 ordered-f64 ULPs and <=1e-12 absolute difference only to validated corresponding generated contour coordinates. Signed-zero differences are rejected.
- `shape_plan_coordinates` tracks explicit Debug scopes from EvaluatedScene through visual layer, Shape source, EvaluatedShape, contours, Contour, points and VectorPoint. It validates nesting, indentation, matching delimiters, x/y order and finite values. Authored VectorPoints and every other field remain exact. Non-finite inputs fail even when the two plans are identical.
- Existing repeated same-runtime raw plan equality, recipe SHA-256, RGB reference comparison, audio/timing checks and legacy/rule-card comparisons remain intact. No production code, dependency, contract, schema, migration or fixture was changed by this follow-up.

| Scenario | Automated evidence |
| --- | --- |
| Accept measured platform contour variation | `shape_reference_plan_accepts_measured_platform_contours` reproduces all three CI values against the checked-in reference plan. |
| Reject semantic drift and excessive numeric variation | `shape_reference_plan_enforces_both_numeric_bounds` checks positive/negative 8 versus 9 ULPs, a one-ULP difference rejected by the absolute cap, zero and actual contour movement. `shape_reference_plan_rejects_unrelated_semantic_drift` checks authored points, dimensions, position, paint, density, ordering, time, bounds and closure. |
| Fail closed on unsupported coordinates and structure | Numeric tests reject NaN/infinity and signed-zero changes; `shape_reference_plan_rejects_malformed_or_mismatched_structure` covers malformed scopes, missing/extra fields, wrong point types, truncation and unmatched structure. |
| Preserve independent golden evidence | Full release native conformance is required without recapture variables, plus hosted Linux render/foundation parity and remaining PR checks before completion. |

## Validation

Local logs: ignored `local-data/shape-portability-verification/`.

- PASS: all four focused `shape_reference_plan_` regressions.
- PASS: `cargo fmt --check --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- PASS: apps/agent-bridge `bun run typecheck`, `bun run lint`, `bun run test` (380 tests).
- PASS: apps/kokoro-tts `bun run ../agent-bridge/scripts/run-python-tests.ts` (10 unittest and 5 pytest).
- PASS: pinned OpenSpec strict validation (21 items).
- PASS: `cargo test --workspace`, including a final rerun after exact trailing-newline enforcement.
- PASS: `bun run contracts:check` (Rust fixtures and 316 TypeScript parity tests).
- PASS: final `cargo test --release -p opencut-editor-core native_golden_render_conformance --lib -- --nocapture` (96.07 seconds), with OPENCUT_GOLDEN_REQUIRED=1 and configured FFmpeg/FFprobe 8.1.2 plus repository DejaVuSans font.
- PASS: source MCP integration (10 tests) and packaged smoke (5 tests).
- PASS: OpenSpec verification and whitespace validation; unchanged fixture bytes checked against Git HEAD.
- PASS: synchronized living requirements, archived the verified change, and ran the post-archive Moon gate (20 specs and CI policy).
- PENDING: publication and hosted CI confirmation (task 4.2).

## Assessment

Completeness: 8/9 tasks complete; all implementation and local validation tasks passed. Only the authorized archive/publication/hosted-CI lifecycle task remains.
Correctness: all four scenarios mapped, with independent negative controls against broad numeric tolerance.
Coherence: approved test-only scope and unchanged references/production semantics.

No critical, warning or suggestion discrepancies remain in implementation, design or local scenario coverage. Ready for authorized synchronization and archival. Completion remains conditional on publication and passing hosted CI.

The six pre-existing ignored workspace entries are isolated subprocess helpers invoked by parent tests and explicit recapture/report-only utilities. No required check was skipped.
