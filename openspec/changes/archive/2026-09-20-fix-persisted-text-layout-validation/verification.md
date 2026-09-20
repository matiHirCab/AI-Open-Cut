# Persisted layout correction evidence

Status: complete on 2026-09-20. Implementation verified, living specifications synchronized, correction archived and final protected gates passed. Artifact approval received on 2026-09-20 ("yes"). Both previous archives are preserved.

## Scenario traceability

| Scenario | Implementation | Automated evidence |
| --- | --- | --- |
| P1, P2 | Shared scope validation invokes canonical style validation only with layout present | persisted_root_layouts_validate_current_hidden_undo_and_redo: current/hidden/undo/redo, numeric and structural failures, exact code/retryability, restoration rejection and complete file inventory/byte equality |
| P3 | Existing load/recovery/revision paths retained; layout-absent root guard | persisted_layout_classification_preserves_legacy_and_unrelated_errors; valid_stale_layout_draft_survives_migration_without_replay; existing valid advanced lifecycle and schema migration tests |
| E1, E2 | Dedicated layout deserializer and exact anchored CoreError classification | expanded retained_invalid_layout_rejects_reopen_and_migration_without_publication: schema 20/21 x add/update/component create/update x twelve invalid styles; root/history decoding matrix |
| E3 | Headless consumes core classification, MCP translates existing envelope | layout_request_errors_expose_core_classification_without_private_markers; persisted_layout_errors_keep_exact_headless_codes_and_files; verifyRichTextWorkflow root/draft failures and unrelated INTERNAL_ERROR control in MCP integration and packaged smoke |
| R1 | Shared scope validation before resource preparation and renderer preflight | invalid_root_layouts_fail_before_any_render_side_effect: frame/range/export, hidden/visible, four invalid styles, unavailable resources/process and existing destination, empty I/O events and preserved output bytes |

The private classification marker is tested by layout_decode_classification_requires_the_exact_anchored_marker, including non-anchored and marker-like text controls. It is stripped from public core/headless messages. Unrelated request error messages retain the existing headless prefix.

The original fourteen issue-35 scenarios remain covered by the original and corrective shaping, layout, migration, lifecycle, contract and native conformance suites. This correction changes no layout arithmetic, raster geometry, shared budget, public field or encoding policy. Required final reruns establish regression evidence; previous reports alone are not proof.

## Check history

Full logs are outside the repository under C:/Users/matia/AppData/Local/Temp/.

- Planning artifacts: strict change validation and all-spec validation passed, 28 items (opencut-persisted-layout-artifacts.log).
- Red regressions: root reopening unexpectedly succeeded and retained null layout returned INTERNAL_ERROR, both exit 101 (opencut-persisted-layout-red.log).
- After the core fix, the full rich-text persistence suite passed, 16 tests, including both expanded matrices (opencut-persisted-layout-core.log).
- Focused renderer preflight and headless request classification passed (opencut-persisted-layout-focused.log).
- Initial bridge formatting found await-in-loop lint in the intentionally sequential persisted-file fixture. Replaced the loop with explicit sequential helper calls; typecheck and lint passed. No concurrency or assertion weakening was used.
- Final Rust formatting and strict Clippy passed; workspace, native, bridge, Python and lifecycle gates are tracked below as they complete.

## Required final results

All implementation checks ran sequentially with Rust 1.97.0. Native checks explicitly used temporary FFmpeg/FFprobe 7.1.1 and the checked-in DejaVuSans fixture. No references were recaptured.

| Check | Result | Full log under the temporary directory above |
| --- | --- | --- |
| cargo fmt --check --all; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace -- --test-threads=2 | All exit 0, including persistence matrices and 28 headless protocol tests | opencut-persisted-layout-rust-final.log |
| Native rich-text conformance; complete font_resolution suite | Both exit 0; native conformance 164.38 seconds, all 16 font tests passed | opencut-persisted-layout-native-final.log |
| Bridge typecheck, lint, test:unit, contracts:check, test:integration, test:smoke | All exit 0; 396 unit, 330 TS contract plus native contract tests, 11 MCP integration, 6 packaged smoke tests | opencut-persisted-layout-bridge-final.log |
| Hermetic Python runner | Exit 0; 10 unittest and 5 pytest tests | opencut-persisted-layout-python-final.log |
| git diff --check; strict all-spec validation | Both exit 0; 28 specification/change items passed | opencut-persisted-layout-prearchive.log |
| Pre-archive protected Moon gate | Exit 1 solely for active fix-persisted-text-layout-validation; policy tests and strict validation passed. Expected rejection, not a passed gate. | opencut-persisted-layout-prearchive.log |

## OpenSpec conformance verification

Applied openspec-verify-change using CLI status and apply context. All four planning artifacts are complete. Proposal, design, both deltas, tasks, implementation, tests and supporting documentation agree.

| Dimension | Assessment |
| --- | --- |
| Completeness | All 19 tasks complete, including conformance, synchronization, archival and final protected gates. |
| Correctness | All three added requirements and seven scenarios have automated coverage in the traceability table. Numeric and structural failures were reproduced before correction. |
| Coherence | Core owns validation and decoding classification; headless only translates core classification. The existing model-to-error dependency is retained. Layout-absent validation and unrelated persisted/request error behavior remain covered by controls. |

No critical issues, warnings or uncovered corrective scenarios were found. Shared validation covers current and both history stacks before candidate publication and is reached by direct renderer preflight. The dedicated deserializer rejects the same strict type and only exact anchored marker matching changes JSON error classification; nested root/history/operation matrices establish propagation. Public messages strip the internal marker. Full inventories establish rejection atomicity without stale-draft replay. Existing transaction recovery ordering is unchanged.

Compatibility was checked through the existing layout, shaping, evaluated geometry, work-budget, alias/lifecycle, migration and native audiovisual suites. Exact legacy expectations and native frame/range/draft/export comparisons passed without geometry, fitting, encoding or fixture changes in this correction. The original fourteen scenarios and prior corrective tests remain in these suites.

Limitations: seven existing core subprocess-helper/report/reference-capture ignores remain intentional; native-specific tests were separately configured and executed rather than relying on unconfigured workspace early returns. The existing proc-macro-error2 future-compatibility notice remains; strict Clippy passed. This verification covers the configured Windows toolchain and FFmpeg 7.1.1, not compatibility with other FFmpeg versions. No required implementation check is unavailable or unresolved. Synchronization and archival are complete; post-archive protected Moon and strict all-spec gates passed.

## Completion

Added the two advanced-text-layout requirements and one rendering-export requirement to the living specifications, preserving existing content. Archived separately at openspec/changes/archive/2026-09-20-fix-persisted-text-layout-validation/. Both 2026-09-19 archives remain intact.

Post-archive protected Moon, strict all-spec validation (27 items) and git diff --check all exited 0: opencut-persisted-layout-postarchive.log. Final gates are rerun after this evidence/task update in opencut-persisted-layout-final-gates.log. No implementation inputs changed after the passing implementation suites. No unresolved required check failures remain.
