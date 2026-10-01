# Correction verification

## Superseding lifecycle evidence — 2026-09-30

The F1-F5 checks below remain evidence of their named scenarios. A later independent review reproduced three additional cases they did not exercise: draft-base eviction, inherited sampled transition/visibility clocks, and encoder diagnostic privacy/size. See the explicitly approved [lifecycle correction verification](../2026-09-30-fix-extended-animation-lifecycle-regressions/verification.md) for new owned regressions and actual check results. In particular, the earlier stale-draft fixture retained a matching base, and shared-intent parity alone did not establish analytically correct transition gain. No historical approval or log is replaced.

Status: verified, synchronized and archived on 2026-09-30. All required implementation checks and final protected gates passed. Approval is recorded in proposal.md. Existing tracked work is preserved in the external baseline patch `C:/Users/matia/AppData/Local/Temp/opencut43-fix-baseline.patch`; original review logs remain unchanged.

| Finding | Requirement/scenarios | Owned regression evidence |
| --- | --- | --- |
| F1 | Bounded extended candidate certification: effects-only scale overshoot; retained/expanded scale | `review_effects_only_scale_spring_rejects_before_publication`, `review_effects_only_safe_scale_remains_accepted`, `review_unused_component_scale_is_certified_without_rotation`, `review_inherited_scale_envelope_does_not_require_rotation`: local/hidden/retained candidates, continuous spring >4 and inherited fractional clocks, edit/draft byte rollback |
| F2 | Shared extended visual channel rendering: offset legacy path identity effect; zero rotation | `review_legacy_shape_affine_preserves_origin_for_identity_extensions`: independent positive/negative origin, nonidentity legacy position/scale, inherited transform/opacity oracles. `native_review_identity_effect_preserves_offset_path_in_every_intent`: independent red-pixel extent, draft/frame/range/export, SSIM and undo/redo/reopen. Existing Transform2D native tests retain explicit controls |
| F3 | Atomic schema-27 migration: invalid legacy-only draft; valid stale draft | `review_schema26_legacy_drafts_validate_matching_retained_base`: entirely schema-26 current/history, missing target ITEM_NOT_FOUND, project/history/draft byte preservation, valid stale matching base and no-write reopen. Existing migration publication-fault tests cover managed resources/recovery |
| F4 | Bounded certification / tiny correlated crop; deterministic sampling / tiny endpoints and holds | `review_correlated_crop_floor_rejects_both_axes_atomically`: X/width and Y/height, edit/batch/draft rollback. `review_tiny_correlated_hold_crop_remains_accepted`: candidate-level safe hold control on both axes. `review_fractional_rotation_and_tiny_crop_keep_canonical_samples`: exact endpoints/holds vs fractional interpolation floor. Existing correlated-linear acceptance |
| F5 | Deterministic compound visual sampling: large integer rotation; holds/loops; compound/fractional timing | `review_integer_rotation_uses_actual_extended_consumers`, `review_root_relative_clock_and_legacy_scalars_keep_integer_progress`, `review_integer_holds_and_reflected_loops_preserve_seams`, `review_compound_tint_uses_integer_and_fractional_consumers`, `review_compound_paths_and_gradients_preserve_large_and_fractional_clocks`: analytic scalar/compound values at 2^53+1 and near-u64 maximum, integer relative clocks and fractional inheritance |

Required commands, compatibility restrictions, lifecycle order and platform-evidence limits are defined in tasks.md and design.md. Full new logs will remain outside the repository. No passing review-era result will substitute for checks whose inputs change during these fixes.

## Preserved scenario coverage

The four modified requirements contain 24 scenarios. Existing tests continue to establish the unchanged scenarios; new tests above establish the correction scenarios. Test names below are in `tests/extended_visual_animation.rs` unless another source is specified. That file is included by the required `animation_channels` target.

| Requirement | Preserved scenarios | Automated evidence |
| --- | --- | --- |
| Bounded extended candidate certification | Unsafe intermediate crop; safe correlation; bounded deterministic work; legacy preservation | `crop_certification_rejects_spring_overshoot_without_publication_and_accepts_correlated_linear_crop`; `node_quota_is_inclusive_and_left_subdivision_is_deterministic` in extended_certification.rs; `exhausted_candidate_analysis_preserves_batch_and_draft_generations`; workspace legacy animation/migration suites |
| Deterministic compound visual sampling | Structured curves/loops; unsafe intermediate samples | `canonical_extended_samples_match_independent_values`, `extended_fractional_clocks_preserve_loop_phase_and_compound_values`, `spring_envelopes_contain_all_stationary_points_and_endpoint_jumps` in animation.rs; canonical compound fixtures and gradient/crop rejection cases |
| Atomic schema-27 extended visual animation migration | Current/history/drafts; invalid retained state; fault recovery and deterministic reopen | `schema_27_migrates_current_history_and_drafts_and_reopen_does_not_rewrite`; `malformed_retained_generations_and_future_versions_never_publish`; `premature_extended_draft_fields_reject_migration_without_touching_any_authoritative_bytes`; `supported_migrations_recover_every_publication_phase`, `supported_migration_before_journal_failure_preserves_generation`, `font_and_legacy_draft_activation_recovers_every_publication_phase` in store.rs |
| Shared extended visual channel rendering | Every property/intent; transformed geometry/audio; fail before output; existing output | `native_compound_path_gradient_crop_and_effect_channels_share_nonzero_range_samples`, `native_rotation_effects_share_frame_range_and_export_samples`, `nested_rotation_clocks_and_scoped_effects_render_at_loop_turns`, `native_extended_draft_and_export_preserve_decoded_audio_and_timing`, `invalid_render_samples_fail_before_inspecting_or_replacing_destination`; required native goldens, rules-screen, Transform2D/fonts and cache suites |

## Correction details and compatibility

All implementation changes remain in editor-core: integer/fractional sample selection in animation.rs; continuous scale and clamp-aware correlated crop certification in extended_certification.rs plus its shape-density consumer; legacy shape origin correction in extended_visual.rs and shared shapes.rs affine algebra; migration replay in store.rs. There are no new dependency edges, parallel transport validators, warning suppressions, schema versions or declarations. Rust/TypeScript/headless/MCP catalogs, capability identifiers, error codes/retryability, budgets and CODEOWNERS are unchanged by this correction. Canonical parity checks independently verify the existing issue implementation. This approval is for the correction; no new remote CODEOWNER approval is claimed.

Workspace testing exposed the already-migrated corrupt-draft recovery fixture. Legacy-only candidate replay is therefore gated on actual schema migration; existing extended-state validation remains unchanged on schema-27 reads. Both the new schema-26 regression and `dangling_persisted_references_fail_closed_with_deterministic_classes` pass. No new stale-base selection policy was introduced.

The final stale-draft control deletes its target from current state while the matching retained base still contains it. Migration succeeds and stale access returns REVISION_CONFLICT, so an incorrect replay against current state cannot satisfy the test. The focused result is in `opencut43-fix-stale-base-selection.log`; affected formatting/Clippy/workspace, native animation and contract checks were rerun after this test-only strengthening and all exited zero. Rust library code, release golden/rules tests, references, toolchain and environment remain unchanged by that fixture edit, so their relevant passing evidence remains valid.

The crop safe-hold regression exposed overbroad rejection in the first correction: matching held channels do not apply an interpolation floor. Correlation remains valid for fully held channels, while below-floor interpolated extents disable the shortcut. Both safe hold axes and unsafe linear axes now pass their independent expectations.

Large-origin conformance controls also exposed an invalid temporary-position workaround in the first path correction: an authored origin of ±600000 with legacy scale 2 exceeded Transform2D position bounds after deriving a translation. The final correction selects the original legacy zero anchor in the shared shape affine algebra instead of changing Transform2D position/anchor values. Independent large positive/negative and inherited coordinate/opacity oracles pass. `opencut43-fix-red-derived-position.log` preserves the failed intermediate correction; `opencut43-fix-final-focused-evaluated.log` and `opencut43-fix-final-focused-native.log` contain the passing controls. Earlier complete runs with the `stable` prefix are intermediate evidence, superseded by final required reruns after this correction; final implementation verification passed.

## Evidence integrity and limits

- Baseline tracked diff is preserved in `opencut43-fix-baseline.patch` outside the repository; this is not claimed as a backup of original untracked files. Original review reproductions remain in `opencut43-review/`.
- Red evidence: `opencut43-fix-red-integration.log` (F1/F3/F4), `opencut43-fix-red-evaluated.log` (F5), `opencut43-fix-red-affine-valid.log` (F2 with corrected painted fixture), and `opencut43-fix-red-tiny-hold.log` (the first crop correction's safe-hold rejection). Early empty-filter/bad-fixture attempts are not counted as evidence.
- `opencut43-fix-rust-required.log` retains the initial recovery compatibility failure. The first unit and integration runs retain timeout failures under concurrent work; they are not counted as passing evidence. Interrupted native runs retain their partial logs and are not counted as complete parity evidence.
- Native path placement compares an independently copied baseline PNG so renderer artifact reuse cannot mask shifts. Legacy and extended pipelines have different color quantization: the fixture uses exact pixel bounds and bounded RGB MSE for that comparison. Its SSIM >=0.99 assertion compares the corrected frame against range/export; it does not assert legacy-to-extended RGB equality. Failed preliminary RGB/SSIM assertions remain in intermediate logs.
- Large-u64 exactness is established through actual evaluated scalar/compound consumers with Rust integer literals and analytic values. Huge-duration exports are not attempted; ordinary-window native intent parity is separate evidence.
- Local Windows evidence does not attest Linux/macOS correctness, POSIX process containment, remote Actions, branch protection, or the weekly full 1920x1080 run. Required local PR scope includes all three resolutions; ordinary opt-in skips require the separate fail-closed native/instrumented runs.

All required implementation suites and lifecycle gates passed; full results are recorded below.

## Required local checks

All logs are under `C:/Users/matia/AppData/Local/Temp/`. Rust uses `RUSTUP_TOOLCHAIN=1.97.0`; Bun is 1.4.0, Moon 2.3.3, OpenSpec 1.5.0. Native checks use the reviewed FFmpeg/FFprobe 7.1.1 paths and DejaVuSans.ttf SHA256 `AE7B7855E115A5966D8B1B3F80F254CCC117EC86F9965E202EE2940453837280`. Required native invocations never enable golden-update or recapture flags. Native golden/animation/cache required flags are enabled for their dedicated runs.

| Check | Result | Full log |
| --- | --- | --- |
| Rust format, strict workspace Clippy, workspace tests | Exit 0 each; 388 core library tests pass, 9 deliberate helpers/capture/report tests ignored in ordinary workspace run | opencut43-fix-final-stale-control-rust.log |
| Actual evaluated regressions | Exit 0; 8 tests including inherited origins | opencut43-fix-final-focused-evaluated.log |
| Focused lifecycle and safe controls | Exit 0; 7 tests including the actual native path regression | opencut43-fix-final-focused-native.log |
| TS formatter, typecheck, lint | Exit 0 each; 82 files checked without fixes | opencut43-fix-ts-checks.log |
| Bridge unit | Exit 0; 426 pass, 1 native opt-in skip covered by instrumented run separately | opencut43-fix-final-bridge-test-unit.log |
| Canonical Rust/TS/MCP contract parity | Exit 0; canonical native suites and 359 TS tests | opencut43-fix-final-stale-control-contracts.log |
| MCP integration | Exit 0; 13 tests | opencut43-fix-final-bridge-test-integration.log |
| Packaged smoke | Exit 0; 8 tests | opencut43-fix-final-bridge-test-smoke.log |
| Official hermetic Python worker runner | Exit 0; 10 unittest and 5 pytest | opencut43-fix-python.log |
| Release golden conformance and report validation | Exit 0 each; all seven corpora and fresh Windows report | opencut43-fix-final-golden.log; opencut43-fix-final-render-report.log |
| Required native animation/Transform2D/fonts and headless lifecycle | Exit 0; 41 animation, 16 font and 13 Transform2D tests plus actual headless lifecycle | opencut43-fix-final-native-tests.log; opencut43-fix-final-stale-control-native.log (latest 41 animation tests); opencut43-fix-final-native-headless.log |
| Required PR rules-screen 960x540/1280x720/1920x1080 | Exit 0 each; 25/25/6 renders, five semantic states per resolution; unchanged references | opencut43-fix-final-rules-960x540.log; opencut43-fix-final-rules-1280x720.log; opencut43-fix-final-rules-1920x1080.log |
| Core raster cache, instrumented worker/build/bridge, restored default build/headless tests | Exit 0 throughout; 13 core, 3 worker and 1 bridge tests; default build restored and 40 headless tests pass | opencut43-fix-final-cache-core.log; opencut43-fix-final-cache-worker.log; opencut43-fix-final-cache-build.log; opencut43-fix-final-cache-bridge.log; opencut43-fix-final-default-build.log; opencut43-fix-final-default-headless.log |
| Pre-archive strict OpenSpec and protected Moon gate | Strict exit 0, 34 items; final protected pre-archive rejection names only this active change (not a passed gate) | opencut43-fix-prearchive-final-strict.log; opencut43-fix-prearchive-final-moon.log |
| Post-archive protected Moon, strict all-spec and isolated Windows bootstrap | Exit 0 each; 33 living specifications pass, policy bootstrap succeeds | opencut43-fix-final-moon.log; opencut43-fix-final-strict.log; opencut43-fix-final-bootstrap.log |

The ordinary workspace's ignored subprocess helpers are exercised by their parent tests. Three review-only reference-capture/refresh tests remain deliberately unexecuted because goldens must not change. The ignored external report validator is executed explicitly in the release native sequence. Report-only performance observations do not establish universal latency/memory guarantees.

## Pre-archive conformance assessment

Completeness: 23/25 tasks complete; only synchronization/archival and the mandatory post-archive gate remain. Correctness: all four modified requirements and all 24 scenarios have the owned evidence mapped above, including independent failing reproductions and passing controls. Coherence: the approved design is followed in editor-core, with unchanged public contracts, schema, budgets, ownership and stale-draft policy. No unresolved implementation mismatch remains. The lifecycle tasks are pending by the required verification order, not omitted implementation. User approval covers the full lifecycle. Platform and automation limits remain as recorded above.

## Final lifecycle assessment

Completeness: 25/25 tasks complete. Correctness: 4/4 modified requirements and 24/24 scenarios mapped to conformance evidence. Coherence: approved ownership and compatibility design followed; no unresolved implementation mismatch. The four modified requirements were merged into animation-channels, project-persistence and rendering-export without removing existing scenarios. This change is archived at openspec/changes/archive/2026-09-30-fix-extended-animation-review-regressions, including its metadata, proposal, design, tasks and deltas. The original archive retains its historical evidence and links this superseding record. Protected CI files and golden fixtures have no changes against HEAD; no commits, pushes or PRs were created. The local Windows and automation limits above remain applicable.
