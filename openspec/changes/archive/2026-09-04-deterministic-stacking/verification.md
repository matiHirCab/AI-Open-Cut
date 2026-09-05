# Verification Report: deterministic-stacking

Verified with openspec-verify-change against the approved proposal, amended design, five delta specs, tasks, code and regression evidence. The rendering-boundary plan was explicitly approved in this task.

## Summary

| Dimension | Status |
| --- | --- |
| Completeness | 20/21 tasks complete; 8 requirements and 22 scenarios mapped in evidence.md |
| Correctness | All 8 requirements have implementation evidence; the confirmed rendering rejection gap is fixed and regression-tested |
| Coherence | Shared validation owner, documented inward edge and architecture enforcement agree with the approved amendment |

## Correctness and coherence evidence

- validation.rs:65 owns canonical ordinal validation; persistence validation delegates at line 80. evaluated_scene.rs:332-333 runs complexity preflight before the same validator, without mutation, before layer construction and resource preparation. Sorting remains unchanged.
- renderer.rs::malformed_stacking_is_rejected_without_side_effects_for_all_facades reproduces the former accepted-invalid-input defect and now checks gaps, duplicates, swapped ordinals, hidden tracks/items, audio, transitions, exact error/message, immutable schema-9 input, and rejection through frame/range/export without publication. Consecutive ordinals and empty tracks remain valid.
- ADR 0003, OWNER_MATRIX and evaluated_scene_excludes_persistence_and_renderer_details explicitly allow evaluation to call validation while retaining every other boundary.
- Valid hand-built evaluation, render-plan and native/golden fixtures now assign canonical ordinals at construction; malformed stacking fixtures are not normalized. Native stacking, legacy/golden, split, Transform2D and headless lifecycle tests pass without changing rendering baselines.
- Existing model/migration, mutation/alias/history, headless/MCP and canonical catalog evidence remains mapped by scenario in evidence.md. No public API, error catalog, schema or migration was changed by the correction.

## Verification results

Rust formatting, strict workspace Clippy, the full serial workspace suite (232 primary tests), bridge typecheck/lint/unit/contracts/integration/packaged-smoke, and all 15 strict OpenSpec entries passed. Native tests used explicit FFmpeg/FFprobe 8.1.2 and DejaVuSans. The original failing regression and first-run fixture corrections are recorded in evidence.md. A concurrent memory-sampler failure passed in isolation and the complete serial rerun; sampler code was untouched. Python provider tests were not rerun because their behavior and contracts are unaffected. Five ignored Rust tests are existing helper/maintenance entrypoints.

## CRITICAL: completion gates

1. Task 4.3: canonical and consumer contract parity passes, but designated CODEOWNER @matiHirCab review of the resulting diff has not been recorded. Obtain that review; proposal and corrective-plan approval do not substitute for it.
2. Task 6.4: the user explicitly requested archival after being informed that contract-owner review remains pending. Requirements are synchronized and archival is being completed under that instruction; the Moon result is recorded below. Task 4.3 remains open and this is not a contract approval.

## Warnings and suggestions

No unresolved implementation/spec divergence or uncovered correction scenario was found. No additional code changes are recommended by this verification.

## Assessment

The rendering fix is implemented and technically verified. Final contract-owner review remains required for merge readiness. The user explicitly authorized archival with that review still pending.

## Archive result

Archived to openspec/changes/archive/2026-09-04-deterministic-stacking under the explicit user instruction. Eight requirements were synchronized into five living specs. Pinned Moon 2.3.3 root:openspec-validate passed: 231 policy tests, all 14 living specs and CI parity policy. Task 6.4 is complete. The sole incomplete task is 4.3 (designated contract-owner review); PR #105 remains draft.
