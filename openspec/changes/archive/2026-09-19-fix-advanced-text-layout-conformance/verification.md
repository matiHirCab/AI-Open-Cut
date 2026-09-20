# Corrective conformance evidence

Status: complete. Implementation verified, living specifications synchronized, corrective change archived, and protected Moon plus strict all-spec gates passed. User artifact approval received on 2026-09-19. The original issue-35 archive is preserved.

## Corrective scenario traceability

| Scenarios | Core implementation | Independent automated evidence |
| --- | --- | --- |
| G1 | evaluated_scene logical-box anchors and raster translation | logical_box_pixel_conformance: authored 100x70 box at (50,40), decoded red background bounds (50,40)-(149,109), with and without -20px shadow |
| G2 | text_logical_box, local_transform_matrices, legacy_anchor shared by animated sampling | logical_box_mapping_ignores_paint_margins_and_fractional_raster_rounding: all nine anchors, fractional bounds, italic overhang, stroke/shadow, rotation/anisotropic scale and independent parent matrix; advanced animation_conformance compares explicit linear-value oracle samples |
| W1 | fonts/shaping LayoutWidth and logical diagnostic widths | fractional_reported_bounds_retain_one_line_and_authored_size: MMMMM, size 30, tracking 0.1, both word/cluster and none/shrink |
| W2 | saved width at word-break opportunities; logical width independent of bidi placement | logical_widths_match_pinned_metrics_for_bidi_mixed_faces_and_word_breaks: independent font-table advances, bidi, bold/mixed faces, exact/next_down bounds and backtracked word widths; existing exhaustive integer-mode expectations |
| D1, D2 | validation/styled_text typed operation and component-track traversal | retained_invalid_layout_rejects_reopen_and_migration_without_publication: schema 20/21, add/update/component-create/component-update, negative tracking and unusable padded bounds; recursive file inventory and byte equality |
| D3 | canonical validation without draft materialization | valid_stale_layout_draft_survives_migration_without_replay: current state unchanged, retained draft bytes unchanged, later RevisionConflict; existing valid draft, document and paint rollback regressions |
| B1 | checked private GlyphBudget | candidate_budget_is_inclusive_checked_and_atomic: actual 16,777,216 equality, one excess, usize overflow, unchanged charge on rejection |
| B2 | one budget per expanded-scene measurement | expanded_text_budget_rejects_before_output_inspection_or_allocation: actual component/repeater expansion yields three two-glyph occurrences, six accepted and five rejected via test-only limit |
| B3 | unchanged glyph-count charge semantics | newline_only_candidates_retain_zero_glyph_accounting: 1000 candidates, three empty lines, zero budget succeeds; existing per-candidate glyph/line boundary tests retained |
| R1 | shared core preflight geometry and diagnostics | native_rich_text_render_conformance: frame/range/draft/export, all fit modes, fractional bounds, negative margins, slots/repeaters, exact diagnostic equality, SSIM >=0.99, PCM RMS <=0.0001 and timing <=one frame; legacy exact comparisons retained |
| R2 | preflight before renderer output operations | expanded_text_budget_rejects_before_output_inspection_or_allocation: frame/range/export, existing destination, no exists/request_id/create_dir/write/rename/remove events, destination bytes and preview inventory unchanged |

Paths are under crates/editor-core/src and its existing integration tests. The test-only numeric limit reaches the same private budget/preflight code as production. There is no public or runtime override.

## Original fourteen scenarios retained as acceptance obligations

| Original scenario | Evidence rechecked in required suites |
| --- | --- |
| Opt in and inherit existing styles | canonical_layout_boundaries; advanced-text-layout.test.ts; legacy fixture comparisons |
| Validate all boundaries and locations | canonical_layout_boundaries; component/edit fixtures; new retained draft matrix |
| Track and wrap multilingual clusters | advanced_tracking_preserves_clusters_and_line_boxes; advanced_wrap_modes_honor_hard_breaks_and_complete_clusters; new W1/W2 |
| Position and paint a bounded text block | fractional_box_alignment_uses_padding_and_line_boxes; rounded_background_is_confined_to_padding_box_without_filling_effect_margins; new G1/G2 |
| Reject unusable padding boxes | canonical_layout_boundaries; retained draft matrix; MCP workflow invalid padding rollback |
| Resolve each fit mode and exact ties | fit_modes_choose_largest_exact_integer_and_preserve_authored_values; W1/W2 |
| Report unavoidable overflow | unavoidable_overflow_and_global_work_fail_before_rasterization |
| Bound fitting work and reject missing bounds | canonical_layout_boundaries; shaping work limits; B1-B3/R2 |
| Edit through aliases and lifecycle operations | advanced_layout_aliases_history_drafts_replacements_and_rollback; document lifecycle tests; verifyRichTextWorkflow |
| Reject stale missing and incompatible edits | alias/history rollback tests; valid_stale_layout_draft_survives_migration_without_replay; MCP workflow |
| Migrate history atomically and preserve legacy pixels | schema_20_layout_migration_preserves_complete_current_and_history; generation transaction failure tests; new retained draft matrix; native legacy pixel comparisons |
| Keep public contracts compatible | Rust/headless and bridge canonical contract parity, schema tests, MCP integration and packaged smoke |
| Verify layout across every render intent | native_rich_text_render_conformance and advanced animation oracle |
| Reopen without original fonts and fail safely | native font-resolution tests, managed integrity failures, B1-B3/R2 |

## Check history

Complete logs are outside the repository under C:/Users/matia/AppData/Local/Temp/.

- Initial red regressions reproduced W1 (two lines instead of one) and D1 (opening succeeded with negative retained tracking): opencut-corrective-red-width.log and opencut-corrective-red-draft.log.
- During test development, a padding fixture omitted required padding fields, producing a deserialization error; completed the fixture and verified INVALID_ARGUMENT plus byte equality. The geometry fixture initially omitted historical required fields and then selected the plain legacy preparation path; corrected those setup errors. An intermediate core run therefore had 316 passing tests, one geometry-fixture failure and seven ignored tests (opencut-corrective-core-unit.log).
- Focused final geometry/metric tests: exit 0, four passing (opencut-corrective-geometry-final.log).
- Focused rich-text persistence suite: exit 0, fourteen passing, including retained nested layouts and valid stale drafts (opencut-corrective-rich-final.log).
- Expanded shared-budget preflight: exit 0 (opencut-corrective-budget.log).
- First strict workspace Clippy: exit 0 (opencut-corrective-clippy.log). Dependency proc-macro-error2 emits a future-compatibility notice; workspace lint has no suppressed warning.
- Final Rust formatting, strict Clippy and workspace tests: all exit 0 (opencut-corrective-rust-final.log).
- Interim strict OpenSpec validation: 28 passed, exit 0; protected Moon exit 1 naming only this active change, an expected rejection rather than a passed gate (opencut-corrective-prearchive-gates.log).
- Native font-resolution suite: exit 0, sixteen tests including configured native rendering (opencut-corrective-native-final.log). The first native conformance run passed the exact shadow pixel repro but failed the advanced animation fixture's growth assertion because the wide centered box clipped glyphs at the canvas edge. Narrowed only that fractional test box to keep the growth oracle in frame; tolerances and implementation unchanged. Native conformance recheck passed with exit 0 (opencut-corrective-native-recheck.log), including all configured visual/audio checks and the exact shadow-position regression.
- After the native test-fixture correction, formatting and Clippy again passed, but the repeated workspace run failed in the existing process_tree_sampler_isolated_helper: sampled child working-set increase did not reach its 32 MiB assertion (opencut-corrective-rust-recheck.log). Its parent test reported the failure; 318 other core unit tests passed and seven helpers/optional tests were ignored. This sampler passed in the prior complete workspace run. No sampler code or assertion is changed; the isolated sampler recheck subsequently passed unchanged (opencut-corrective-sampler-recheck.log). A final complete workspace run uses --test-threads=2 to reduce contention without skipping tests; that complete workspace run passed, as did final formatting and strict Clippy (opencut-corrective-rust-accepted.log; all exits 0).
- Bridge typecheck/lint/unit (396)/contracts (330 plus Rust consumers)/MCP (11)/packaged smoke (6): all exit 0 (opencut-corrective-bridge-final.log). Python: ten unittest and five pytest tests, exit 0 (opencut-corrective-python-final.log).
- Conformance review caught zero logical width being accidentally subjected to the positive raster-size constraint in the new affine helper. A newline-only transformed-layout regression reproduced it (opencut-corrective-zero-width-red.log). Logical dimensions now permit zero while raster dimensions retain their original positive constraint; unit geometry and native blank-frame coverage added. The zero-width geometry regression passed after correction (opencut-corrective-zero-width-green.log). Final Rust formatting/Clippy/workspace and native conformance/font suites passed after this compatibility correction (opencut-corrective-rust-accepted.log and opencut-corrective-native-accepted.log; all exits 0). Final bridge rerun passed all six commands (opencut-corrective-bridge-accepted.log; all exits 0); Python evidence remains applicable.
- Final post-archive OpenSpec gates: protected Moon exit 0 and strict all-spec exit 0, 27 passed (opencut-corrective-postarchive-gates.log). Evidence was obtained for this correction rather than relying on the original issue-35 report.

## Conformance review

Conformance review completed after required implementation checks passed. No public fields, operations, capabilities, schema version, encoding policy or ownership dependency edge is changed by this correction. Legacy geometry and shaping arithmetic remain on their original paths. ShapedLayout.background already stores the logical fractional dimensions and the raster-space box origin, so evaluated geometry reuses it without duplicating offset fields. Draft validation occurs under locked loading before font publication and the persistence transaction; it does not replay retained operations.

## Final implementation check results

All commands ran sequentially with Rust 1.97.0. Native checks used the verified temporary FFmpeg/FFprobe 7.1.1 and pinned DejaVuSans.ttf fixture. Logs are under C:/Users/matia/AppData/Local/Temp/.

| Check | Exit | Log |
| --- | --- | --- |
| cargo fmt --check --all | 0 | opencut-corrective-rust-accepted.log |
| cargo clippy --workspace --all-targets -- -D warnings | 0 | opencut-corrective-rust-accepted.log |
| cargo test --workspace -- --test-threads=2 | 0 | opencut-corrective-rust-accepted.log |
| cargo test -p opencut-editor-core native_rich_text_render_conformance -- --nocapture | 0 | opencut-corrective-native-accepted.log |
| cargo test -p opencut-editor-core --test font_resolution -- --nocapture | 0 | opencut-corrective-native-accepted.log |
| bridge: bun run typecheck; bun run lint; bun run test:unit | 0 each | opencut-corrective-bridge-accepted.log |
| bridge: bun run contracts:check; bun run test:integration; bun run test:smoke | 0 each | opencut-corrective-bridge-accepted.log |
| worker: bun run ../agent-bridge/scripts/run-python-tests.ts | 0 | opencut-corrective-python-final.log |
| git diff --check | 0 | direct command output; only existing CRLF normalization notices |

The workspace core unit result is 319 passed, zero failed and seven existing intentional ignores: five subprocess helpers exercised by their parent tests, plus explicit reference-capture/report-only jobs. Native conformance ran with tools configured; sixteen font tests passed. Bridge totals: 396 unit tests, 330 TypeScript contract tests plus governed Rust consumers, eleven MCP tests and six packaged tests. Python totals: ten unittest and five pytest tests. No required suite was skipped. The intermittent process sampler failure remains documented as a test stability limitation; no assertion or implementation was weakened to obtain the final clean run.

## OpenSpec conformance assessment

- Completeness: all implementation, regression and pre-archive verification tasks complete. All five corrective requirements and twelve corrective scenarios have automated evidence, alongside the original fourteen acceptance scenarios.
- Correctness: independent box-coordinate/pixel expectations, pinned-font metric and exact-boundary oracles, retained-file byte inventories, and actual expanded-scene budget tests establish the corrected behavior. The zero-width logical-box edge discovered during review is resolved and covered by matrix and native blank-frame assertions.
- Coherence: core ownership and dependency direction preserved; canonical draft validation is reused without stale replay; geometry reuses private evaluated layout data; integer fit search and production limits remain unchanged. No public contract/schema/encoding modification is introduced by this correction.
- Findings: no unresolved implementation, design or scenario-coverage mismatch. The earlier process sampler failure is recorded as an existing test stability limitation, with unchanged focused and complete workspace reruns passing.
- Pre-archive gates after all implementation suites: strict OpenSpec exit 0, 28 passed; protected Moon exit 1 naming only fix-advanced-text-layout-conformance. This is expected pre-archive rejection, not gate success (opencut-corrective-prearchive-final.log).

Synchronization and archival completed under the approved plan. Five corrective requirements were merged into the two living specifications, and the correction was archived separately as 2026-09-19-fix-advanced-text-layout-conformance. Both post-archive gates passed; all 23 tasks are complete. The original issue-35 archive remains unchanged. No required check or implementation task remains outstanding.
