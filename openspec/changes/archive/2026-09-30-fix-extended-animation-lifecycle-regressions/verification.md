# Verification record

Status: explicitly approved on 2026-09-30; D1-D3 corrections implemented; all required implementation checks passed; conformance verified; living specs synchronized, correction archived and all final protected checks passed. Approval was the user message "approve" after all four artifacts were presented.

Planning validation: pinned OpenSpec `validate --all --strict --no-interactive` exited 0 with 34 items passed; full log is `C:/Users/matia/AppData/Local/Temp/opencut43-rereview/correction-artifacts-validation.log`. CLI status confirms all four planning artifacts exist. `git diff --check` exited 0. These checks establish artifact structure only; they do not approve or verify implementation. The protected archive-only gate is not claimed passed while this change is active.

## Independently reproduced review failures

Baseline HEAD: 08a543f62a471b704715f2fa6ae32ed93c94f517, plus the complete existing uncommitted issue #43 implementation and first correction. Full review record and logs: `C:/Users/matia/AppData/Local/Temp/opencut43-rereview/review.md` and `focused-with-control.log`. External probe source is retained in `focused-source.log` and `encoder-source.log`; disposable probe executables and project fixtures were removed after review.

| Finding | Trigger and observed evidence | New scenarios / planned verification |
| --- | --- | --- |
| D1, P1, store.rs:1543-1551 | Valid draft at revision 2 refers to later deleted item; current extended effects activate candidate replay; history eviction at revision 103 produces ITEM_NOT_FOUND on edit, read and discard. Legacy-only control succeeds through revision 115. | Evict a valid stale draft base during ordinary edits; Migrate a stale draft whose base is no longer retained. Owned public API, migration, byte-preservation and applicable-base controls, tasks 2.1-2.3. |
| D2, P2, render_plan.rs:119-122 | Component begins at root 1000 ms, local fade-out 0-500 ms. Root 1250 ms analytic gain is 0.5: legacy frame/range/export center red is 127; identity vignette sampled branch produces 0 in all three intents. | Preserve an offset occurrence fade under an identity effect; Preserve scaled nested and repeated transition clocks. Independent gains/legacy controls, all intents, tasks 2.4-2.5. |
| D3, P2, render_process.rs:618-622 | Fault encoder consumes stdin, writes 8000 characters plus quoted private Windows path, exits 7. Core visual_prepare error has 8070-byte excerpt with raw absolute path. Bridge already sanitizes its secondary result. | Encoder emits private paths and oversized stderr; Preserve existing bridge diagnostics compatibility. Core/headless injection, safe excerpts and cleanup, tasks 2.6-2.7. |

The focused external suite exits 101 with three expected failures; these are confirmed defects, not successful regression evidence. Missing fixture fields in earlier probe attempts were corrected before the final control run and are not additional findings.

## Fresh required check results

All logs below are under `C:/Users/matia/AppData/Local/Temp/opencut43-lifecycle-fix/`. Native commands have `.command` and `.exit` sidecars. Exact commands are also retained in tasks.md. Local tools are Rust 1.97.0, Bun 1.4.0, Moon 2.3.3 and pinned OpenSpec 1.5.0.

| Command / check | Exit and observations | Full logs |
| --- | --- | --- |
| Owned lifecycle integration target with required native tools | 0, five tests, including real frame/range/draft/export and inherited controls | lifecycle-final.log; initial lifecycle-red.log (101) |
| Owned exact transition / encoder subprocess tests | 0 each | transition-exact-final.log; encoder-final.log; initial encoder-red.log (101) |
| cargo fmt --check --all | 0 on final code | fmt-stable.log; earlier fmt.log |
| cargo clippy --workspace --all-targets -- -D warnings | 0; only the existing dependency future-incompatibility notice, no warning suppression introduced | clippy-stable.log; earlier clippy.log |
| cargo test --workspace | 0; core 391 passed, nine explicitly ignored capture/helper/report cases, affected integration/transport suites passed | workspace-final.log; earlier workspace.log |
| Bridge Biome formatter / typecheck / lint | 0 each | ts-format.log; typecheck.log; lint.log |
| Bridge test:unit | Initial 1, provider cancellation 20-second timeout under concurrent load; serial rerun 0, 426 passed with unchanged timeout, one native opt-in skipped and covered separately | unit.log; unit-serial.log |
| bun run contracts:check | 0, canonical Rust consumers and 359 TS tests | contracts-final.log; earlier contracts.log |
| Bridge test:integration | 0, 13 MCP tests | integration-final.log; earlier integration.log |
| Bridge test:smoke | Initial 1, packaged group workflow 30-second timeout under concurrent load; isolated rerun 0, all eight tests passed with unchanged timeout | smoke-final.log; initial smoke.log and isolated smoke-serial.log |
| Official hermetic Python worker runner | 0, ten unittest and five pytest cases | python.log |
| Required native animation_channels / font_resolution / transform2d | 0, actual 46 / 16 / 13 tests, no native dependency skips | native-final.log; earlier native.log |
| Exact native headless lifecycle | Initial 101, Windows target executable in use during integration; retry 0 after that load completed | native-headless-final.log; initial native-headless.log and native-headless-retry.log |
| Actual headless sampled encoder fault wire | 0 | native-headless-encoder.log |
| Required core raster cache | 0, 13 tests | cache-core-final.log; earlier cache-core.log |
| Instrumented worker cache / build / bridge cache | 0 each, worker four tests, bridge one native test | cache-worker-final.log; cache-build-final.log; cache-bridge-final.log |
| Default headless rebuild / ordinary verification | 0 each, restored after instrumentation | default-restore-final.log; default-headless-final.log |
| Release native golden conformance | 0, all seven corpora against unchanged references, 265.40 seconds on final production code | golden-final.log; fresh golden-report-final.json; superseded golden.log |
| Fresh external golden performance report validator | 0 | golden-report-validation-final.log; superseded golden-report-validation.log |
| Strict all-spec validation before archival | 0, 34 items | strict-prearchive-final.log; earlier strict-prearchive.log |
| Protected Moon gate before archival | Expected 1 naming only this active change; 377 policy tests and all 34 specification items passed | moon-prearchive-final.log; earlier moon-prearchive.log |
| PR rules-screen matrix on final code | 0 at all three resolutions, 25 + 25 + 6 actual renders; all required semantic states, no references changed | rules-final-960x540.log; rules-final-1280x720.log; rules-final-1920x1080.log |
| Living-spec synchronization | Three normalized requirement blocks equal the approved deltas, with all earlier scenarios preserved | spec-sync-final.log |
| Final protected Moon gate | 0, policy tests and 33 living specs pass after archival | moon-postarchive.log; metadata rerun moon-final.log |
| Final strict all-spec validation | 0, 33 items | strict-postarchive.log; metadata rerun strict-final.log |
| Isolated Windows CI-policy bootstrap | 0, unchanged reviewed bootstrap and Moon boundary | bootstrap-postarchive.log; metadata rerun bootstrap-final.log |
| Final whitespace / preservation audit | 0; HEAD, protected gates, tracked golden references and starting canonical/CODEOWNERS diffs preserved | diff-postarchive.log; metadata rerun diff-final.log; preservation-final.log |

The final 960x540 PR resolution passed all 25 renders/five states in 2967.51 seconds (`rules-final-960x540.log`). The final 1280x720 PR resolution also passed all 25 renders/five states in 2894.78 seconds (`rules-final-1280x720.log`). The final 1920x1080 PR resolution passed its six renders/all five semantic states in 1575.50 seconds (`rules-final-1920x1080.log`). All 56 required PR renders passed; both living specs are synchronized, this correction is archived, and all final protected gates passed. No failed initial check is hidden or treated as a successful run. Workspace-final ran the final production implementation; the later test-only cumulative-budget strengthening passed in preflight-visibility-final and strict all-target Clippy, without another production change. Unchanged TypeScript and Python inputs reuse their passing evidence; native consumer checks were rerun after the final production change.

## Preserved scenario conformance

The complete modified requirement blocks preserve eleven earlier scenarios in addition to the six new lifecycle scenarios. Fresh native animation and workspace suites exercise the earlier coverage; archival does not drop those requirements.

| Requirement | Preserved scenarios | Existing owned coverage rerun here |
| --- | --- | --- |
| Atomic schema-27 migration | Current/history/drafts; invalid retained state; recovery/reopen; invalid legacy target with matching base; valid stale matching base | schema_27_migrates_current_history_and_drafts_and_reopen_does_not_rewrite; malformed_retained_generations_and_future_versions_never_publish; premature_extended_draft_fields_reject_migration_without_touching_any_authoritative_bytes; review_schema26_legacy_drafts_validate_matching_retained_base; store supported_migrations_recover_every_publication_phase / supported_migration_before_journal_failure_preserves_generation / font_and_legacy_draft_activation_recovers_every_publication_phase |
| Shared extended rendering | All properties/intents; transformed geometry/audio; failure before output; existing regressions; offset identity path; zero-rotation path origins | native_compound_path_gradient_crop_and_effect_channels_share_nonzero_range_samples; native_rotation_effects_share_frame_range_and_export_samples; nested_rotation_clocks_and_scoped_effects_render_at_loop_turns; native_extended_draft_and_export_preserve_decoded_audio_and_timing; invalid_render_samples_fail_before_inspecting_or_replacing_destination; native_review_identity_effect_preserves_offset_path_in_every_intent; review_legacy_shape_affine_preserves_origin_for_identity_extensions; native transform/font/cache and golden suites |

Tracked canonical catalog and CODEOWNERS diffs exactly match the starting tracked patch (`canonical-baseline.log`). The correction did not edit their existing issue #43 changes or introduce a public/persisted shape, capability, dependency edge or stable error change. Contract parity is fresh evidence of their consumers. No new remote CODEOWNER or branch-protection approval is claimed. Protected execution files and golden references have no diff against HEAD.

## Evidence discipline

Fresh correction logs are retained at `C:/Users/matia/AppData/Local/Temp/opencut43-lifecycle-fix/`, including the starting tracked patch/status, initial red tests and diagnostic investigation logs. Native invocations also retain command and exit sidecars. Earlier compilation/fixture failures (JSON macro depth, missing transform, invalid nested duration, wrong test-only error enum) were corrected before the final owned tests; they are test development observations, not additional product defects.

### Owned coverage and implementation mapping

| Decision / scenario | Automated evidence | Implementation |
| --- | --- | --- |
| D1 ordinary base eviction | lifecycle_evicted_draft_base_preserves_reads_edits_conflicts_and_discard: 110 public edits, legacy and extended controls, revision 115, absent base verified, reads/fresh facade/conflicts/discard and authoritative bytes | store load_project_data requires a matching base before semantic replay; catalog/resource checks precede missing-base continuation |
| D1 missing-base schema adoption | lifecycle_schema26_missing_draft_base_preserves_draft_and_generation: current and both history sides, draft bytes, no-write conflicts/reopen | Existing transactional migration and matching-base validation preserved |
| D1 malformed/matching-base/fault controls | Existing review_schema26_legacy_drafts_validate_matching_retained_base, premature_extended_draft_fields_reject_migration_without_touching_any_authoritative_bytes, schema_27_migrates_current_history_and_drafts_and_reopen_does_not_rewrite, store migration fault suite | Existing draft structural/font/resource and transaction owners |
| D2 offset fade / every intent | native_lifecycle_offset_transition_preserves_analytic_gain_in_all_intents: real draft facade, independent red=127 expectation, legacy/extended/draft frame/range/export, no authoritative writes | EvaluatedVisualLayer visible_at and transition_gain; sampled raster multiplies gain exactly once; sampled backend no further transition filter |
| D2 scaled/nested/repeated clocks | native_lifecycle_scaled_nested_and_repeated_transitions_match_legacy_and_analytic_gains: independent root/scaled/nested/repeated red expectations, decoded frame/range/export plus verified legacy controls | Canonical occurrence rate/offset and root visibility used in preparation and overlay |
| D2 fractional, crossfade, exact boundaries | sampled_transition_gain_preserves_fractional_nested_clocks_and_exact_boundaries: actual evaluated nested .75*root-50 clock, self-crossfade product, half-open visibility, integer endpoints above 2^53 | Existing SampleTime exact/fractional segment selection consumed by transition_gain |
| D2 preflight / failure before output | sampled_preflight_uses_root_visibility_for_effect_work_before_preparation: late intrinsic media measurement, inactive sample accepted, active unsafe sample rejected; three safe 71,714,816-pass layers fit, four exceed 268,435,456 | Sample preflight and raster preparation share visible_at; existing cumulative effect_budget remains unchanged |
| D3 oversized Unicode/private diagnostics | sampled_encoder_failure_has_safe_bounded_diagnostics_and_reaped_child: consumes stdin, exit 7, bounded Unicode tail, Windows/POSIX quoted paths, UTF-8 safe excerpt and executable removable | Existing stderr_excerpt helper reused in prepare_visual_stream |
| D3 artifact failure integrity | native_lifecycle_encoder_failure_cleans_workspace_and_preserves_destination_and_state: partial encoder output, overwrite destination preserved, authoritative bytes unchanged, no remaining .opencut-work directory | Existing RenderWorkspace drop and export publication retained |
| D3 headless/bridge compatibility | native_sampled_encoder_failure_is_safe_on_the_headless_wire: real worker and typed FFMPEG_FAILED/nonretryable visual_prepare exit 7 safe excerpt; canonical parity and existing bridge diagnostic tests required | Headless transports safe core diagnostics; bridge sanitization unchanged |

Investigation refined D2's cause: incorrect local-span visibility already produced transparent sampled rasters at offset root times, independently of transition filters. Removing the filter alone was insufficient; the final correction handles both approved occurrence timing invariants. Temporary diagnostic prints and constant gains were removed. This evidence supersedes the review's narrower causal inference while preserving its confirmed wrong-output finding.

Final conformance inspection also found the same local/root visibility comparison in sample preflight. `preflight-visibility-red.log` independently reproduced a missed active effect-budget rejection (exit 101); `preflight-visibility-green.log` passes after using the canonical root visibility helper. `preflight-visibility-final.log` strengthens that test with independently calculated cumulative-budget controls. This is the same approved D2 clock/failure invariant, not a new limit or compatibility policy. A first 960x540 rules run passed before this last production change; the next resolution caught the developing test fixture's compilation errors. Those logs are retained and are superseded by a complete stable-code rerun. The earlier golden/transport/cache results are retained; final reruns all passed on the frozen production code.

Do not mark tasks complete based on prior passing suites. Record exact owned test names, commands, tool versions, native dependencies, exit status and full external log paths after each correction. Preserve failure logs as well as successful reruns. Use analytic transition gain, actual decoded media and independent public lifecycle operations; shared-plan parity alone is insufficient.

Required native checks use pinned Rust 1.97.0, FFmpeg/FFprobe 7.1.1 and canonical DejaVu Sans SHA256 AE7B7855E115A5966D8B1B3F80F254CCC117EC86F9965E202EE2940453837280. No golden update/recapture variables are permitted. Fresh performance reports are observations, not universal resource guarantees.

## OpenSpec conformance scorecard

The $openspec-verify-change review read proposal, design, both complete delta specs and tasks, and inspected owned implementation plus independent failure/decoded output evidence. No requirement, design, ownership or test mismatch remains.

| Dimension | Verified result |
| --- | --- |
| Completeness | Three requirements, all 17 scenarios mapped; all implementation tasks/checks complete. Final status: 22/22 tasks complete. Prearchive review had 20/22 complete, with only the authorized ordered sync/archive and post-archive policy milestones remaining at that point. |
| Correctness | 3/3 requirements and 17/17 scenarios have owned automated coverage, including eleven preserved scenarios and six added lifecycle scenarios. Red/green evidence establishes each corrected defect; green suites alone were not used as proof. |
| Coherence | D1-D3 follow ADR 0003 and existing persistence, occurrence-clock and process diagnostic owners. No new shape, schema, dependency edge, renderer intent, public code/retryability or protected-gate change. |

Critical implementation issues: none. Warnings or suggestions requiring implementation: none. All lifecycle milestones and final protected checks passed. Unavailable execution evidence is listed separately below.
## Limits

The new owned corrections pass focused local Windows execution; prior review logs remain historical evidence and are not substituted for the required fresh suites. Linux/macOS execution, real POSIX containment, remote CI/branch protection, weekly full rules-screen scope and remote CODEOWNER approval remain unavailable unless newly observed. Portable POSIX path-string sanitization can be tested locally without claiming a POSIX runtime run. Structural CLI artifact readiness does not constitute approval or conformance.
