# Verification: add-initial-motion-preset-pack

Verification skill: `.codex/skills/openspec-verify-change/SKILL.md`. Status/context captured in uncommitted `issue46-verify-status.json` and `issue46-verify-context.json` logs. Planning artifacts and all four deltas were reviewed. Independent Sol medium delegated implementation review at b076f9a9 accepted the core implementation, canonical fixtures and governed consumers; all mandatory local implementation and native technical checks passed. No human GitHub CODEOWNER review is represented by this record.

## Completeness

Implementation covers the five approved IDs, canonical fixed expansions, complete per-property effective provenance, schema30 current/component/history migration, atomic channel/blur application, typed headless/MCP consumers and source/packaged workflows. All15 tasks are checked with terminal required evidence. The unmerged prerequisite is PR134 head0dd191d2240718e9550712f412fa03812f3f342c; issue45 is not closed.

## Requirement and scenario ledger

| Requirement / approved scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| Discoverable typed initial motion pack: old/new clients; failure/capability parity | headless main capability; bridge schemas and timeline forwarding; manual MCP structural definitions | headless motion_pack wire/lifecycle tests; TS animation-presets/contracts tests; shared preset-workflow source and packaged smoke |
| Versioned primitive compilation: seed; unsupported identity | timeline animation_presets compile_scalar/compile, model strict union | canonical_expansion_defaults_and_undo_reopen_are_exact; motion_pack_invalid_bounds_and_identity_leave_documents_and_resources_unchanged |
| Explicit channel collision policy: disjoint reject; selected replace; legacy preservation | complete candidate channel merge | collision_replace_order_and_raw_clear_are_explicit; replacement_preserves_order_static_state_and_legacy_collisions; motion_pack_partial_collisions_replace_order_and_nonimpact_blur_preservation |
| Descriptive persisted provenance: effective save; retired reopen; malformed rejection | model decoder; validation validate_provenance | motion_pack_canonical_primitives_complete_sources_and_history_are_exact; motion_pack_descriptive_retirement_membership_and_premature_generations_fail_closed; existing malformed/duplicate source tests |
| Bounded safe expansion: numeric/shape rejection; merged/retained limits | validate_parameters and complete candidate validation | motion_pack_invalid_bounds_and_identity_leave_documents_and_resources_unchanged; motion_pack_raster_budget_failure_rolls_back_aliased_prefix_and_existing_draft; existing scene/channel budget tests |
| Blur-sensitive provenance: blur change/undo; raw reconciliation | blur_changed/reconcile_raw_tracks | motion_pack_impact_blur_reject_replace_retired_labels_and_undo_are_exact; motion_pack_component_replacement_clears_only_blur_dependent_sources |
| Closed pack semantics: fixed expansion; minima/odd times; invalid shape/derived bounds; repeat seams | bounded pure compiler, checked integer phases; existing evaluator | motion_pack_canonical_primitives_complete_sources_and_history_are_exact; motion_pack_phase_minima_and_odd_times_use_fixed_oracles; motion_pack_raw_shape_errors_preserve_original_duplicate_fields; compiler fixed samples |
| Compatible atomic application: partial rollback; subset order; failures/history | staged candidate mutation and existing editor transaction | motion_pack_partial_collisions_replace_order_and_nonimpact_blur_preservation; raster-budget alias rollback; groups/instances/lifecycle; headless batch/undo/reopen |
| Shared primitive rendering: fixed equivalents | unchanged channel/blur renderer | native_motion_pack_matches_fixed_primitives_frames_range_draft_export_and_fractional_clocks, required native environment; AV range compares decoded video and audio |
| Atomic schema30 migration: unchanged scalar content; generation-wide fail closed; journal recovery | model pre30 envelope guards; migrations; existing locked journal | motion_pack_schema29_scalar_current_components_and_history_migrate_once_without_relabeling; motion_pack_schema29_component_and_retained_sources_fail_without_writes; retained raw decoder tests; schema29_pack_publication_faults_preserve_complete_generations |


| Reconciled typed discoverable preset parity: closed unions; schema30 status; six-entry catalog | additive capabilities and manual strict request/saved/output schemas | headless raw wire/lifecycle; scalar and pack source/packaged MCP workflow; TS contract/status checks |
| Reconciled atomic schema29 staging: staged schema30; unchanged scalar/current/components/history; current idempotence | schema29 intermediate migration followed by schema30 publication | schema_28_migrates_current_components_and_all_history_once; motion_pack_schema29_scalar_current_components_and_history_migrate_once_without_relabeling; premature retained generation tests |
| Reconciled fail-closed generations: malformed state; pre30 pack tags; future schemas/backup rollback | whole-generation validation before journal writes | malformed_retained_provenance tests; five-pack component/undo/redo pre30 rejection matrix; future current/undo/redo tests; schema29_pack_publication_faults_preserve_complete_generations |

These rows cover 13 normative requirements and 34 scenarios across four delta capabilities. Shared preexisting ownership tests retain scalar collision, locks, aliases, batch limits, draft restrictions, split/copy/trim lifecycle, raw component reconciliation and preview/export semantics. The scoped-target regression is covered for scalar and all five packs by motion_pack_and_scalar_preserve_unrelated_scoped_channel_targets. Canonical pack fixture was authored before consumers and reviewed independently; no live-registration generation or render reference/tolerance changes.

## Coherence and limitations

Core owns all semantic compilation/validation/persistence. Transports only decode/forward; no new dependency edge, renderer branch, UI/provider behavior or dependency. Scalars retain compiler1 and exact old expansion; packs use compiler2. Saved looping parameters require materialized iterations; descriptive historical identities are not dispatched. Native opt-out cases in ordinary tests are not render evidence. Native execution uses the documented Linux subreaper helper, not a claim of bare-POSIX portability.

## Formal conformance assessment

| Dimension | Result |
| --- | --- |
| Completeness |15/15 tasks;13/13 normative requirements;34/34 scenarios mapped |
| Correctness |Approved expansion/migration/lifecycle/transport and fixed rendered oracles conform |
| Coherence |Owning layers preserved; scalar behavior retained; no new renderer/dependency edge |

CRITICAL: none. WARNING: none. SUGGESTION: none. No verification dimension was skipped. All mandatory local implementation and native gates passed; ready for synchronization/archive. Protected postarchive gates are the next delivery requirement. Exact-head remote CI and duration audit will be tracked after authorized draft creation, separately from local conformance. Optional weekly full rendering is not claimed passed.

## Retained failed verification evidence

The first full workspace run failed three test assertions: two stale schema29 expected migration outputs and a new undo fixture selecting a snapshot before the source existed. Test-only corrections update current schema expectations and select a tagged source snapshot. The failed driver/workspace logs are retained with `first-failed` names; corrected full workspace rerun passed. Earlier implementation diagnostics are listed in tasks.md; not every overwritten intermediate log is claimed retained.

Packaged smoke first run passed8 tests and failed the new pack workflow: its standalone MCP collision request accidentally included the batch-only `operation` discriminator, so strict MCP validation correctly rejected the shape before core and supplied no core structured result. The test now omits that field (as the existing scalar standalone workflow does) and asserts `isError` plus the canonical core collision result. No consumer/product contract was changed. Failed packaged/driver logs are retained; affected type/lint/unit, packaged and source MCP checks passed.


## Mandatory evidence checkpoint

Full corrected workspace/fmt/strict Clippy PASS (driver timestamps22:23–22:28UTC); Python PASS; TypeScript type/lint PASS;433 hermetic unit cases PASS plus one optional native opt-out; cross-language contracts PASS; packaged9 PASS; MCP15 PASS. Native preset3 PASS (scalar visual, scalar audio and all-five pack including decoded AV range),259.23s; required headless lifecycle PASS; native release baseline and baseline report validation PASS. Full native cache/worker/channel/font and all three PR render shards remain in progress. Full logs are uncommitted in `/tmp/review-logs/issue46-*`.

One source MCP attempt was killed with exit137; unchanged retry passed. Logs preserve both attempts; resource exhaustion is an inference, not a proven cause. The execution connection restarted but saved files and terminal driver logs survived. No failed required check is treated as a pass without a subsequent passing run.


Older-reader future-schema conformance is also automated in accepted prerequisite0dd: PROJECT_SCHEMA_VERSION=29 and retired_source_reopens_without_dispatch_and_future_schema_fails_closed sets its future version to30, asserts INTERNAL_ERROR and unchanged project bytes. That exact older-source test passed in correction-c454-workspace.log and the accepted0dd CI suite. This is compiled older-schema test evidence; no separate manually launched downgrade binary probe is claimed. Complete backup rollback remains the documented path, not an in-place downgrade feature.


## Final native matrix evidence

Required channel101/font16/transform13, core cache13, worker3, bridge-worker1 and restored headless5+38+2+2 PASS in issue46-required-native-driver.log. All three required PR shards PASS:960x5405 states/25 renders1793.96s;1280x7205 states/25 renders3020.25s;1920x1080 PR5 semantic states/6 renders1598.44s.56 total required render operations, unchanged fixture references/tolerances. Peak process trees reported1,095,262,208 /1,536,393,216 /2,765,053,952bytes respectively. Driver exited0. Full workspace889 passed,9 existing ignored; ordinary native opt-outs are not counted as native evidence. Independent delegated acceptance remains valid after test-only standalone shape correction and documentation/format changes. No implementation blocker remains. Earlier pending checkpoints are historical snapshots, superseded by this terminal result.

## Archive and protected delivery gates

Final delegated reconciliation review accepted all13 requirements/34 scenarios. All four capabilities synchronized, preserving every preexisting scenario; change archived2026-10-03 with all artifacts and15 tasks complete. Postarchive `moon run root:openspec-validate`, strict all-spec validation (37/37) and CI policy bootstrap PASS. No active-change exception remains. Exact-head remote CI is pending publication at this immutable prepublication record; its terminal evidence is maintained separately to avoid changing the verified published head solely to record CI.
