# Verification report: add-track-mattes

Verified 2026-10-06 using the repository openspec-verify-change workflow and independent Sol Medium review. Proposal, design, seven deltas, tasks and approved mechanism addenda are the authority.

| Dimension | Decision |
| --- | --- |
| Completeness | All implementation tasks complete; 17 requirements / 41 scenarios mapped. Lifecycle archival and postarchive gate follow below. |
| Correctness | All mapped local witnesses passed; no unresolved local findings. |
| Coherence | Core inward ownership, immutable evaluated facts and pure artifact execution follow ADR-0003. |

## Actual validation

- Workspace fmt and Clippy with `-D warnings` passed; workspace tests: 1187 passed, zero failed, nine declared ignores. Five ignored subprocess helpers have passing parents; three review-only reference capture/refresh tools were not executed; the ignored performance helper was subsequently explicitly executed and passed.
- Mandatory native core suite: five passed, zero skipped; covers all four intents and independent raw/full-plate pixel oracles. Release native golden, explicit performance report and required rules checks at 960/1280/1920 passed.
- Cache core15, instrumented worker4 and bridge1 passed; default headless binary restored, rebuilt and tested afterward (6+47+3 passed).
- Bridge typecheck/lint passed; fresh unit suite562 passed with three declared native opt-in skips. Dedicated required MCP native witness actually executed separately: one passed, zero skipped, within unchanged60-second deadline. Real aliases, reorder, undo/redo, reopen, stale conflicts and audio-bearing matteOnly provider passed. PCM192512 bytes matched exactly, nonzero RMS0.08816, 10frames, video/audio1second, start0.
- Protocol3, contracts310 Rust plus421 TypeScript, integration23 and packaged smoke20 passed. Earlier contract/integration/packaged executions were narrowly requalified after only the excluded dedicated native test changed; shared inputs and166 Rust/Cargo sources remained identical. They were not represented as repeated executions.
- Hermetic Python12 unittest plus5 pytest passed. Inherited pure26, owner12 and TLS allocator3 witnesses explicitly passed; source-cache/no-mask/all-intent proofs remain intact.
- Protected prearchive Moon exited1 solely because add-track-mattes remained active, after387 policy tests and strict43 validations passed. This expected rejection is not a passing gate.

## Findings and limitations

No unresolved local correctness or coherence findings. Earlier semantic RED, fixture errors, omitted runner rules scope, architecture ownership failure and public luma codec-boundary failure remain preserved in external logs. The public oracle now first captures genuine prepared PAM (red luma92), then compares final output to a complete independently authored plate through the unchanged encoding graph (red90); tolerance remains1 and deadline60seconds. No lint suppression, guard/budget weakening or predecessor repinning was introduced.

Mac/Windows conditional directory-lookup controls and mandatory native CI execution must still pass on the published exact commit. Human CODEOWNER review remains pending on the draft PR. Local Linux evidence does not establish other-platform runtime success. The1GiB certificate bounds application-owned live payload, not total process RSS.

## Requirement and scenario traceability

### Canonical matte model graph and renderer contracts / Share all public representation and failure cases

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `apps/agent-bridge/tests/track-mattes.test.ts`: `preserves canonical references and presence-sensitive edits through every wrapper`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `rejects malformed nested references, null visibility and excessive UTF8 IDs`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `requires nonnull stored references and only defaults on ineligible DTOs`.
- `apps/headless/tests/protocol.rs`: `matte_aliases_scoped_graph_and_final_atomic_provider_deletion_roundtrip`.
- `apps/headless/tests/protocol.rs`: `canonical_raw_matte_duplicates_reject_single_batch_and_draft_atomically`.
- `crates/editor-core/tests/track_mattes.rs`: `public_matte_fields_are_observable_and_omission_null_and_visibility_are_distinct`.
- `crates/editor-core/tests/track_mattes.rs`: `canonical_references_utf8_bound_and_raw_duplicates_match_public_domain`.
- `apps/agent-bridge/tests/smoke.test.ts`: `authors scoped track mattes through actual MCP aliases, atomic DAG edits and drafts`.
- `apps/agent-bridge/tests/packaged-smoke.test.ts`: `authors scoped track mattes through actual MCP aliases, atomic DAG edits and drafts`.

### Distinct matte model and complete rendering readiness / Reject incomplete readiness and audit digest scope

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `apps/headless/tests/protocol.rs`: `health_succeeds_when_editor_is_ready_and_rendering_is_degraded`.
- `apps/headless/tests/protocol.rs`: `ready_renderer_advertises_canonical_linear_composition_in_protocol_v1`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `requires all58 exact new field shapes and strict reference definition before MCP projection`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `rejects duplicated capabilities and preserves unapproved matching fields and annotations`.

### Exact matte MCP predecessor projection / Reject missing additions and unrelated drift

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `apps/agent-bridge/tests/track-mattes.test.ts`: `requires all58 exact new field shapes and strict reference definition before MCP projection`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `rejects duplicated capabilities and preserves unapproved matching fields and annotations`.
- `apps/agent-bridge/tests/contracts.test.ts`: `preserves unauthorized matching fields and annotations in the exact predecessor projection`.

### Active mask authoring contract and capability / Detect supported mask models through unavailable rendering

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `apps/headless/tests/protocol.rs`: `health_succeeds_when_editor_is_ready_and_rendering_is_degraded`.
- `apps/headless/tests/protocol.rs`: `ready_renderer_advertises_canonical_linear_composition_in_protocol_v1`.

### Active mask authoring contract and capability / Reject cross-language catalog drift

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/mask_models.rs`: `canonical_cases_and_ordered_stacks_use_the_public_owner_and_preserve_rejected_bytes`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `pins all eight exact marker-only predecessor catalogs without discarding drift`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `requires all58 exact new field shapes and strict reference definition before MCP projection`.

### Active mask authoring contract and capability / Synchronize current catalog schema markers without feature drift

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `apps/agent-bridge/tests/track-mattes.test.ts`: `pins all eight exact marker-only predecessor catalogs without discarding drift`.
- `apps/agent-bridge/tests/contracts.test.ts`: `expands the approved additive MCP capability catalog deterministically`.

### Active mask authoring contract and capability / Match active authoring and matte schema contracts

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `apps/agent-bridge/tests/track-mattes.test.ts`: `pins all eight exact marker-only predecessor catalogs without discarding drift`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `preserves canonical references and presence-sensitive edits through every wrapper`.
- `apps/headless/tests/protocol.rs`: `matte_aliases_scoped_graph_and_final_atomic_provider_deletion_roundtrip`.

### Bounded exact MCP predecessor conformance proof / Preserve the complete current and predecessor proof

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `apps/agent-bridge/tests/contracts.test.ts`: `expands the approved additive MCP capability catalog deterministically`.

### Bounded exact MCP predecessor conformance proof / Preserve malformed and unrelated drift rejection

Spec: `specs/contract-governance/spec.md`. Passed local witnesses:

- `apps/agent-bridge/tests/track-mattes.test.ts`: `requires all58 exact new field shapes and strict reference definition before MCP projection`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `rejects duplicated capabilities and preserves unapproved matching fields and annotations`.
- `apps/agent-bridge/tests/contracts.test.ts`: `preserves unauthorized matching fields and annotations in the exact predecessor projection`.

### Explicit current layer pipeline / Observe crop effects and affine ordering

Spec: `specs/linear-light-compositing/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/temporal_fixture_tests.rs`: `linear_crop_ordered_effects_ancestor_affine_and_opacity_have_independent_oracle`.
- `crates/editor-core/src/evaluated_scene/temporal_fixture_tests.rs`: `crop_mask_then_ordered_effects_and_nonuniform_ancestor_have_independent_oracle`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.

### Explicit current layer pipeline / Blend mixed current sources in evaluated order

Spec: `specs/linear-light-compositing/spec.md`. Passed local witnesses:

- `crates/editor-core/src/renderer/golden/linear.rs`: `native_linear_every_current_source_family_uses_normal_scene_blend`.
- `crates/editor-core/tests/mask_rendering.rs`: `native_animated_mask_family_board_preserves_all_intents_and_independent_linear_colors`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_alpha_luma_chain_hidden_offscreen_mask_effect_affine_and_audio_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_pinned_text_provider_real_fitting_and_glyph_interior_all_intents`.

### Explicit current layer pipeline / Preserve absent-stage defaults

Spec: `specs/linear-light-compositing/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/mattes.rs`: `default_matte_metadata_bypasses_graph_and_empty_values_preserve_scene_exactly`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.

### Explicit current layer pipeline / Activate authored masks before effects

Spec: `specs/linear-light-compositing/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/temporal_fixture_tests.rs`: `crop_mask_then_ordered_effects_and_nonuniform_ancestor_have_independent_oracle`.
- `crates/editor-core/tests/mask_rendering.rs`: `native_animated_mask_family_board_preserves_all_intents_and_independent_linear_colors`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.

### Explicit current layer pipeline / Activate isolated matte after transforms

Spec: `specs/linear-light-compositing/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes_native.rs`: `native_alpha_luma_chain_hidden_offscreen_mask_effect_affine_and_audio_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `provider_gain_precedes_luma_and_recipient_gain_follows_coverage`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `chained_provider_mattes_and_gain_match_hand_multiplied_luma`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_pinned_text_provider_real_fitting_and_glyph_interior_all_intents`.

### Normative coordinate and compositing semantics / Resolve equal z-index layers

Spec: `specs/motion-graphics-architecture/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/stacking.rs`: `stacking_lifecycle_and_track_reorder_compatibility`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `ordered_provider_source_over_is_color_noncommutative`.

### Normative coordinate and compositing semantics / Evaluate an inherited visual

Spec: `specs/motion-graphics-architecture/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/inherited_animation_timing.rs`: `timing_edits_are_alias_aware_atomic_and_reversible`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_component_local_mattes_bind_two_occurrences_and_original_clocks_all_intents`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `actual_provider_tasks_preserve_large_integer_root_and_fractional_ping_pong_sampling`.

### Normative coordinate and compositing semantics / Distinguish current and future pipeline stages

Spec: `specs/motion-graphics-architecture/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/architecture.rs`: `motion_graphics_adr_locks_required_architecture_semantics`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `default_matte_metadata_bypasses_graph_and_empty_values_preserve_scene_exactly`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_alpha_luma_chain_hidden_offscreen_mask_effect_affine_and_audio_all_intents`.

### Normative coordinate and compositing semantics / Keep scope occurrence and paint order independent

Spec: `specs/motion-graphics-architecture/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/mattes.rs`: `scoped_components_bind_same_local_ids_to_distinct_provider_groups_deterministically`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `ordered_provider_source_over_is_color_noncommutative`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_component_local_mattes_bind_two_occurrences_and_original_clocks_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_repeater_copy_shutters_before_aggregate_and_nested_recipient_shutter_all_intents`.

### Single inward owner for matte dependencies / Enforce owner and handoff boundaries

Spec: `specs/motion-graphics-architecture/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/architecture.rs`: `private_owner_dependencies_match_the_approved_matrix`.
- `crates/editor-core/tests/architecture.rs`: `standard_out_of_line_modules_are_analyzed_recursively`.
- `crates/editor-core/tests/architecture.rs`: `motion_graphics_adr_locks_required_architecture_semantics`.

### Atomic schema 34 scoped matte adoption / Adopt all retained generations and resources

Spec: `specs/project-persistence/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes.rs`: `genuine33_current_undo_redo_and_own_base_draft_adopt34_without_semantic_changes`.
- `crates/editor-core/tests/track_mattes.rs`: `retained_component33_defaults_adopt_and_undo_redo_without_read_rewrites`.
- `crates/editor-core/src/store.rs`: `matte_only_and_reference_edits_preserve_staged_resources_and_fault_generations`.
- `crates/editor-core/src/store.rs`: `matte_draft_mutations_keep_legacy_migration_atomic_and_commit_removes_draft`.

### Atomic schema 34 scoped matte adoption / Reject premature or malformed retained data atomically

Spec: `specs/project-persistence/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes.rs`: `premature33_stored_and_draft_field_presence_rejects_before_any_inventory_change`.
- `crates/editor-core/tests/track_mattes.rs`: `premature_component_fields_and_late_future_history_reject_without_adoption`.
- `crates/editor-core/tests/track_mattes.rs`: `existing_matte_graph_failures_do_not_publish_unrelated_retained33_adoption`.
- `crates/editor-core/src/store.rs`: `matte_journal_draft_updates_validate_original_sources_and_candidates_before_replay`.

### Source-matched matte drafts and journal recovery / Validate stale available and unavailable draft bases

Spec: `specs/project-persistence/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes.rs`: `stale_matte_drafts_validate_own_available_base_and_never_replay_unavailable_base`.
- `apps/headless/tests/protocol.rs`: `stale_matte_draft_preview_conflicts_without_artifacts_or_replaying_current`.
- `apps/agent-bridge/tests/track-matte-native.test.ts`: `renders alpha and luma mattes through actual MCP artifacts with independent color and revision controls`.

### Source-matched matte drafts and journal recovery / Preserve recovery across publication phases

Spec: `specs/project-persistence/spec.md`. Passed local witnesses:

- `crates/editor-core/src/store.rs`: `matte_journal_draft_updates_validate_original_sources_and_candidates_before_replay`.
- `crates/editor-core/src/store.rs`: `matte_only_and_reference_edits_preserve_staged_resources_and_fault_generations`.
- `crates/editor-core/src/store.rs`: `matte_draft_mutations_keep_legacy_migration_atomic_and_commit_removes_draft`.

### Shared evaluated track matte semantics / Compare analytic and actual native all-intent output

Spec: `specs/rendering-export/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes_native.rs`: `native_alpha_luma_chain_hidden_offscreen_mask_effect_affine_and_audio_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_repeater_copy_shutters_before_aggregate_and_nested_recipient_shutter_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_component_local_mattes_bind_two_occurrences_and_original_clocks_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_pinned_text_provider_real_fitting_and_glyph_interior_all_intents`.
- `apps/agent-bridge/tests/track-matte-native.test.ts`: `renders alpha and luma mattes through actual MCP artifacts with independent color and revision controls`.

### Shared evaluated track matte semantics / Validate dependencies before cache hits or output

Spec: `specs/rendering-export/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.
- `crates/editor-core/tests/track_mattes.rs`: `matte_frame_excess_rejects_before_missing_backend_workspace_and_output_changes`.
- `crates/editor-core/tests/track_mattes.rs`: `hidden_referenced_provider_safe_endpoints_do_not_hide_unsafe_continuous_mask_interior`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `forged_descriptor_last_use_work_request_and_peak_fail_before_callback`.

### Shared evaluated track matte semantics / Require both defaults for exact bypass

Spec: `specs/rendering-export/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/mattes.rs`: `default_matte_metadata_bypasses_graph_and_empty_values_preserve_scene_exactly`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.

### Closed scoped matte references and visibility / Author through standalone and creation-alias batches

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes.rs`: `public_matte_fields_are_observable_and_omission_null_and_visibility_are_distinct`.
- `apps/headless/tests/protocol.rs`: `matte_aliases_scoped_graph_and_final_atomic_provider_deletion_roundtrip`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `preserves canonical references and presence-sensitive edits through every wrapper`.
- `apps/agent-bridge/tests/track-matte-native.test.ts`: `renders alpha and luma mattes through actual MCP artifacts with independent color and revision controls`.
- `apps/agent-bridge/tests/smoke.test.ts`: `authors scoped track mattes through actual MCP aliases, atomic DAG edits and drafts`.
- `apps/agent-bridge/tests/packaged-smoke.test.ts`: `authors scoped track mattes through actual MCP aliases, atomic DAG edits and drafts`.

### Closed scoped matte references and visibility / Preserve defaults and eligibility failures

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes.rs`: `strict_matte_wire_and_source_schema_guards_preserve_old_defaults`.
- `crates/editor-core/tests/track_mattes.rs`: `canonical_references_utf8_bound_and_raw_duplicates_match_public_domain`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `rejects malformed nested references, null visibility and excessive UTF8 IDs`.
- `apps/agent-bridge/tests/track-mattes.test.ts`: `requires nonnull stored references and only defaults on ineligible DTOs`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `default_matte_metadata_bypasses_graph_and_empty_values_preserve_scene_exactly`.

### Composition-scoped finite matte dependency DAG / Reject hidden scoped cycles and dangling edges

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes.rs`: `matte_graph_failures_are_atomic_and_delete_clear_batch_validates_final_candidate`.
- `crates/editor-core/tests/track_mattes.rs`: `component_transient_matte_dag_is_deferred_only_until_final_batch_or_draft_candidate`.
- `crates/editor-core/tests/track_mattes.rs`: `hidden_and_unused_scoped_dags_reach_each_inclusive_edge_and_depth_bound`.
- `crates/editor-core/tests/track_mattes.rs`: `existing_matte_graph_failures_do_not_publish_unrelated_retained33_adoption`.

### Composition-scoped finite matte dependency DAG / Accept exact bounds and atomic reference replacement

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes.rs`: `hidden_and_unused_scoped_dags_reach_each_inclusive_edge_and_depth_bound`.
- `crates/editor-core/tests/track_mattes.rs`: `matte_graph_failures_are_atomic_and_delete_clear_batch_validates_final_candidate`.
- `crates/editor-core/tests/track_mattes.rs`: `component_transient_matte_dag_is_deferred_only_until_final_batch_or_draft_candidate`.
- `apps/headless/tests/protocol.rs`: `matte_aliases_scoped_graph_and_final_atomic_provider_deletion_roundtrip`.

### Isolated premultiplied provider coverage / Distinguish colored alpha luma and stage order

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `alpha_and_linear_luma_have_independent_color_oracles`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `isolated_luma_does_not_sample_the_destination_background`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `provider_gain_precedes_luma_and_recipient_gain_follows_coverage`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `chained_provider_mattes_and_gain_match_hand_multiplied_luma`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_alpha_luma_chain_hidden_offscreen_mask_effect_affine_and_audio_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.

### Isolated premultiplied provider coverage / Use matte-only and inactive providers

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `empty_hidden_provider_is_zero_and_support_never_grows`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_alpha_luma_chain_hidden_offscreen_mask_effect_affine_and_audio_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.

### Deterministic occurrence and sampled matte semantics / Bind two component occurrences and staggered copies

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/mattes.rs`: `scoped_components_bind_same_local_ids_to_distinct_provider_groups_deterministically`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_component_local_mattes_bind_two_occurrences_and_original_clocks_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_repeater_copy_shutters_before_aggregate_and_nested_recipient_shutter_all_intents`.

### Deterministic occurrence and sampled matte semantics / Distinguish overlapping copy averaging order

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `overlapping_copies_average_before_aggregation_and_draw_individuals_once`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_repeater_copy_shutters_before_aggregate_and_nested_recipient_shutter_all_intents`.

### Deterministic occurrence and sampled matte semantics / Preserve nested shutter and exact clock sampling

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/mattes.rs`: `actual_provider_tasks_preserve_large_integer_root_and_fractional_ping_pong_sampling`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `weighted_duplicate_samples_and_exact_live_task_reuse_do_not_resample`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `continuous_duration_one_depth_32_preserves_weighted_samples_without_provenance_product`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_repeater_copy_shutters_before_aggregate_and_nested_recipient_shutter_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_component_local_mattes_bind_two_occurrences_and_original_clocks_all_intents`.

### Preflight matte work and live memory / Reject work memory and nested multiplicity overflow

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/mattes.rs`: `actual_main_frame_work_exact_boundary_and_excess_count_all_completed_copies`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `actual_main_uncached_request_limit_is_shared_across_scoped_provider_groups`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `exact_request_cap_and_repeated_materialization_are_charged_without_large_planes`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `exact_work_live_limits_and_checked_overflow_have_independent_admission_controls`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `actual_outer_and_nested_schedule_capacities_cannot_be_hidden_by_len`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `continuous_actual_graph_refines_excess_classes_at_integer_collision_regions`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `continuous_duration_one_depth_32_preserves_weighted_samples_without_provenance_product`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `actual_spare_font_payload_capacity_and_caller_clone_join_shared_fixed_memory`.

### Preflight matte work and live memory / Preserve exact absence and complete readiness

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/mattes.rs`: `default_matte_metadata_bypasses_graph_and_empty_values_preserve_scene_exactly`.
- `crates/editor-core/tests/track_mattes.rs`: `matte_frame_excess_rejects_before_missing_backend_workspace_and_output_changes`.
- `apps/headless/tests/protocol.rs`: `health_succeeds_when_editor_is_ready_and_rendering_is_degraded`.
- `apps/headless/tests/protocol.rs`: `ready_renderer_advertises_canonical_linear_composition_in_protocol_v1`.

### Preflight matte work and live memory / Preserve existing mask reservations under dependency reuse

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/evaluated_scene/mattes.rs`: `transitive_shutter_leaf_samples_share_mask_work_before_materialization`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `ordinary_and_mask_segments_share_root_scene_limit_with_duplicate_shutter_ticks`.
- `crates/editor-core/tests/track_mattes.rs`: `hidden_referenced_provider_safe_endpoints_do_not_hide_unsafe_continuous_mask_interior`.
- `crates/editor-core/src/evaluated_scene/mattes.rs`: `continuous_transitive_effect_work_joins_existing_owner_instead_of_resetting_per_provider`.

### Preflight matte work and live memory / Reject invalid premultiplied provider data

Spec: `specs/track-mattes/spec.md`. Passed local witnesses:

- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `bad_premultiplied_pixels_gain_and_late_callback_preserve_destination_bits`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `forged_descriptor_last_use_work_request_and_peak_fail_before_callback`.
- `crates/editor-core/src/render_artifact/mattes/tests.rs`: `actual_outer_and_nested_schedule_capacities_cannot_be_hidden_by_len`.

### Mandatory native track-matte CI conformance / Execute actual native core and MCP conformance on the CI head

Spec: `specs/repository-validation/spec.md`. Passed local witnesses:

- `crates/editor-core/tests/track_mattes_native.rs`: `native_component_local_mattes_bind_two_occurrences_and_original_clocks_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_alpha_luma_chain_hidden_offscreen_mask_effect_affine_and_audio_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_pinned_text_provider_real_fitting_and_glyph_interior_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_repeater_copy_shutters_before_aggregate_and_nested_recipient_shutter_all_intents`.
- `crates/editor-core/tests/track_mattes_native.rs`: `native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents`.
- `apps/agent-bridge/tests/track-matte-native.test.ts`: `renders alpha and luma mattes through actual MCP artifacts with independent color and revision controls`.

### Mandatory native track-matte CI conformance / Reject weakened mandatory matte execution

Spec: `specs/repository-validation/spec.md`. Passed local witnesses:

- `scripts/validate-ci-gates.test.ts`: `rejects ${label} mandatory matte command: ${command}`.
- `scripts/validate-ci-gates.test.ts`: `rejects an instrumented headless build before mandatory MCP matte proof`.


## Lifecycle result

Seven living capabilities synced;445unmentioned requirement blocks and both complete parent native oracle blocks preserved. Archived only this change2026-10-06. Postarchive unchanged protected Moon passed with387policy tests and43strict validations, zero failures. All24tasks complete. Exact-head remote CI and human CODEOWNER review remain pending.
