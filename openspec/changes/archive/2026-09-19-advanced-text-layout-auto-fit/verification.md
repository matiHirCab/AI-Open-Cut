# Issue #35 verification evidence

Status: complete. Implementation verified, living specifications synchronized, change archived and both final gates passed.

The user approved the proposal and public contract design on 2026-09-19. JSON compatibility is additive; schema 21 is a forward migration. Existing font profile and simple text behavior remain unchanged when layout is absent. No new private-owner dependency edge was introduced.

## Requirement and scenario traceability

| Requirement / scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| Closed opt-in layout: legacy/default inheritance; numeric/structural boundaries and locations | model/text_layout.rs, model.rs, validation/text_layout.rs | canonical_layout_boundaries; advanced-text-layout.test.ts; documents_survive_copy_split_move_trim_components_and_drafts (invalid hidden component layout) |
| Cluster-safe boxes: multilingual tracking/wrapping/mandatory breaks | fonts/shaping.rs | advanced_tracking_preserves_clusters_and_line_boxes; advanced_wrap_modes_honor_hard_breaks_and_complete_clusters; explicit_line_height_keeps_baseline_step_across_face_metrics; existing mandatory separator and bidi regression tests |
| Box positioning/painting and unusable padding | evaluated_scene/text_layout.rs, render_artifact/text.rs | fractional_box_alignment_uses_padding_and_line_boxes (all nine alignment combinations); rounded_background_is_confined_to_padding_box_without_filling_effect_margins; canonical_layout_boundaries |
| Fitting: all modes, exact ties, unavoidable overflow and work limits | evaluated_scene/text_layout.rs | fit_modes_choose_largest_exact_integer_and_preserve_authored_values; unavoidable_overflow_and_global_work_fail_before_rasterization; canonical_layout_boundaries; existing glyph/line shaping limits |
| Reversible edits: aliases, lifecycle, failed/stale/missing targets and drafts | existing timeline/store/draft paths carrying TextStyle | advanced_layout_aliases_history_drafts_replacements_and_rollback; documents_survive_copy_split_move_trim_components_and_drafts; verifyRichTextWorkflow in integration and packaged smoke |
| Atomic schema migration and legacy compatibility | migrations.rs, persisted model preprocessing, existing generation transaction | schema_20_layout_migration_preserves_complete_current_and_history; supported_migrations_recover_every_publication_phase and supported_migration_before_journal_failure_preserves_generation extended to schema 20; existing schema-19 styled migration/history/draft test |
| Public contract parity and capability | advanced-text-layout-v1.json, ownership catalog, headless/MCP catalogs and typed consumers | canonical native status tests; contracts.test.ts; advanced-text-layout.test.ts; MCP integration and packaged smoke |
| Shared render intents/diagnostics and resource failures | evaluated_scene/text_layout.rs, render_artifact/text.rs, renderer.rs | native_rich_text_render_conformance extended with all fit modes, rounded boxes, component slot and repeater occurrences; existing native pinned font reopen/source removal and integrity tests |

All paths above are relative to crates/editor-core/src or their existing test directories unless otherwise specified. No requirement uses a manual-only test exemption. Native tests compare decoded pixels/SSIM and audio RMS; pure layout tests use known DejaVu metrics independently of the implementation.

## Check history

Full command outputs are in `C:/Users/matia/AppData/Local/Temp/` and are not committed.

- `bun run typecheck`: exit 0 (`opencut-35-final-bridge.log`).
- `bun run lint`: exit 0, 70 files (`opencut-35-lint-final.log`). Prior line-ending failures were corrected with the project formatter.
- `bun run test`: exit 0, 396 tests (`opencut-35-final-bridge.log`). Prior schema fixture and canonical required-field order failures were corrected.
- `bun run contracts:check`: exit 0, native consumers plus 330 TypeScript contract tests (`opencut-35-contracts.log`). The final rerun after the status schema correction also passed (`opencut-35-contracts-final.log`).
- `bun run test:integration`: exit 0, 11 tests (`opencut-35-integration-final.log`). Initial status failures exposed and fixed the stale schema-20 status literal.
- `bun run test:smoke`: exit 0, 6 packaged tests (`opencut-35-packaged-smoke.log`).
- `bun run ../agent-bridge/scripts/run-python-tests.ts` from apps/kokoro-tts: exit 0, 10 unittest and 5 pytest tests (`opencut-35-python.log`).
- `cargo fmt --check --all` and `cargo clippy --workspace --all-targets -- -D warnings` with Rust 1.97.0: exit 0 (`opencut-35-final-clippy.log`). Clippy reports a dependency future-compatibility notice for proc-macro-error2; workspace strict lint succeeds.
- Strict all-spec validation: 27 passed, 0 failed before archival.
- `moon run root:openspec-validate`, invoked via `bunx @moonrepo/cli@2.3.3`: exit 1 naming only this unarchived change (`opencut-35-prearchive-gate.log`). This is expected pre-archive rejection, not a passed final gate. Proto bootstrap could not download its registry plugin; the exact pinned Moon npm package runs the unchanged gate successfully up to that expected rejection.
- Initial native workspace runs failed because sandbox access denied the system FFmpeg launcher and, with access restored, installed FFmpeg 8.1/9.0 rejected the existing filter_complex_script option. Logs: `opencut-35-rust-tests.log`, `opencut-35-rust-pinned.log`, `opencut-35-native-text-8.log`. FFmpeg 7.1.1 was downloaded from the GyanD/codexffmpeg release into a temporary test directory. System tools and renderer command syntax were not changed.

## Resolved verification findings

Native fit_box at the earlier CRF 23 produced SSIM 0.9860449207070738 (`opencut-35-native-text-diagnostic.log`), below the unchanged 0.99 threshold. Advanced-layout range/export now request medium-preset CRF 18, with a command regression retaining legacy encoding. The complete seven-mode native matrix then passed (`opencut-35-native-final.log`), as did all four native font tests (`opencut-35-native-font-final.log`). A broad native workspace attempt on the earlier encoding was interrupted after that already-understood conformance failure; it is not claimed as passing evidence (`opencut-35-workspace-7.log`). The normal workspace suite and explicit affected native suites are required independently.

The final review reproduced baseline drift with two pinned faces having different ascenders: explicit 60.5 px line height incorrectly yielded a 67.5078125 px baseline step. A synthetic font metric regression failed before the fix (`opencut-35-baseline-regression.log`) and passed after anchoring explicit steps to the first ascent (`opencut-35-baseline-fixed.log`). All nine horizontal/vertical alignment combinations now have independent geometry assertions.

An overlapping Windows test executable caused LNK1104 in `opencut-35-workspace-final.log`; after the executable exited, the full workspace passed (`opencut-35-workspace-complete.log`). Required Rust and affected native/bridge suites were rerun after the baseline correction; all passed as recorded below. TypeScript-only and Python inputs remain unchanged, so their passing evidence above is retained.

The initial OpenSpec CLI temporary cache disappeared (MODULE_NOT_FOUND). Reinstalling the same pinned package into an isolated temporary cache restored status, instructions and validation without changing repository toolchain configuration.

Final Rust formatting, strict workspace Clippy and all workspace tests passed with Rust 1.97.0 (`opencut-35-rust-verified.log`, exit 0). The final native seven-mode render matrix and four native font tests passed (`opencut-35-native-verified.log`, `opencut-35-font-verified.log`, both exit 0). Final contract parity also passed (`opencut-35-bridge-verified.log`). The simultaneous integration attempt in that log had one DEPENDENCY_UNAVAILABLE failure while the shared headless executable was being rebuilt; the other ten cases passed. The serial rerun passed after Rust completion: all 11 integration and 6 packaged smoke tests (`opencut-35-mcp-serial-final.log`, both exit 0).

## Conformance review

The review follows openspec-verify-change using the approved `advanced-text-layout-auto-fit` status/instructions context and all four planning artifacts. It covers all five requirements and fourteen scenarios from both deltas.

- Completeness: model, validation, fitting, raster backgrounds, diagnostics, migration, typed consumers, canonical fixtures and scenario tests are implemented. Implementation tasks 1.1 through 5.3 have passing evidence; the final packaged rerun also passed; only lifecycle closure remains pending.
- Correctness: numerical fitting expectations use known font units, including exact equality and unavoidable overflow. Cluster tests preserve ligatures/combining sequences/bidi; baseline metrics use a deliberately distinct pinned face. All alignment combinations have independent position assertions. Native tests compare all render intents at the unchanged SSIM/audio tolerances. Migration fault-injection, lifecycle rollback and canonical parity suites cover persisted/public compatibility.
- Coherence: public types live in model/text_layout.rs:52, core validation in validation/text_layout.rs:3, pure fitting in evaluated_scene/text_layout.rs:9, shaping in fonts/shaping.rs, raster painting in render_artifact/text.rs, and diagnostics are copied from resolved scenes at renderer.rs:465. MCP only declares typed schemas (schemas.ts:108,360). Architecture tests and strict Clippy pass. No provider/UI-owned semantics or new dependency edge was introduced.

The review found and resolved explicit baseline drift before closure. Encoding fidelity was raised only for opt-in layout, preserving existing contract tolerances and legacy encoding; design and documentation record the decision. No unresolved semantic or coverage mismatch remains. Contract approval is the user's explicit approval recorded in proposal.md; native/Zod/MCP parity is verified automatically.

### Review scorecard before archival

| Dimension | Result |
| --- | --- |
| Completeness | 15/16 tasks complete; the remaining task is synchronization/archive/final gate itself |
| Correctness | 5/5 requirements and 14/14 scenarios covered; required implementation suites passed |
| Coherence | Core ownership, compatibility, design and canonical consumer parity agree |

No critical issues, warnings or unresolved suggestions remain. Ready to synchronize and archive under the repository's required verification order. Final merge-readiness gate evidence is recorded after archival.

## Final lifecycle evidence

All 16 tasks are complete. The four advanced-text-layout requirements and shared rendering requirement were synchronized into the living specifications. The verified change was archived as `2026-09-19-advanced-text-layout-auto-fit` on 2026-09-19.

- `bunx @moonrepo/cli@2.3.3 run root:openspec-validate`: exit 0 after archival.
- `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`: exit 0, 27 specs passed, 0 failed after archival.
- Full final gate output: `C:/Users/matia/AppData/Local/Temp/opencut-35-postarchive-gates.log`.
- `git diff --check`: exit 0. No test logs, downloaded tools or generated artifacts were added to the repository.

Native rendering evidence uses temporary FFmpeg 7.1.1 because installed FFmpeg 8.1/9.0 do not accept the repository's existing filter_complex_script invocation. No system tool settings or compatibility command syntax were changed. Required affected native tests ran with real fonts/FFmpeg; the earlier failed/interrupted attempts are explicitly recorded above and are not counted as passing evidence.
