# Conformance verification — 2026-09-08

Workflow: openspec-verify-change. Scope: the approved change and approved literal-ID draft correction; preserve pre-existing issue-31 work. Four requirements and eleven scenarios were reviewed against implementation and automated evidence.

## Correctness and coherence

| Requirement | Implementation | Scenario evidence |
| --- | --- | --- |
| Aliased repeater descriptor replacement | timeline::resolve_operation_aliases has a dedicated UpdateItem arm: item ID before optional source ID; scope is untouched. Draft format and execution remain unchanged. | repeaters integration replacement_aliases_commit_once_and_preserve_literal_id_drafts and replacement_alias_failures_roll_back_bytes_and_history cover resolved state, revision, both aliases, missing/forward errors, precedence, literal scope, trailing rollback, byte preservation, drafts, conflicts, undo/redo/reopen. |
| Retained repeater validation domains | evaluate_project_inner validates standalone effective definitions with fresh domain results, then complete root projection. InstanceTraversal retains hidden/clipped ordinary references and local/outer copy metadata; final validation precedes publication. Shared shape measurement and checked accumulators measure composed occurrences; known-size non-vector affine validation uses the refinement helper. | repeater_visual_layer_preflight_accepts_4096_and_rejects_4097 covers root, hidden tracks/repeaters/instances, clipped instances, unused definitions, independent domains, declaration order, later-domain rejection and hidden missing assets. retained_repeater_surface_boundaries_precede_materialization, repeater_preflight_rejects_non_finite_parent_conjugation and scene_shape_budget_accepts_exact_segment_and_raster_limits_only cover raster, matrix, segment and byte boundaries. Existing nested RichText, scope isolation, numeric eleven-sibling order and independent repeaters remain exercised. |
| Bounded ordinary evaluation without redundant snapshots | expand_flat_repeaters and its call are removed. A single common expansion path stores original-layer indices and metadata; only the final publication loop clones generated visible layers. | ordinary_components_do_not_materialize_repeater_copies and test-only thread-local materialization assertions. Invalid retained-domain/layer/raster cases assert zero clones; renderer::golden::invalid_render_work_preserves_project_and_files verifies rejection without backend execution, project changes or filesystem artifacts. |
| Transport parity for aliased repeater replacement | No adapter substitution or public schema changes. Shared source/packaged repeater workflow submits aliases unchanged to core. | Headless repeater protocol cases cover aliases, draft literal IDs, failures, byte rollback and history across fresh processes. Shared MCP workflow covers successful replacement, missing/forward aliases, trailing failure, project reopening, undo/redo and rendered preview in source integration and packaged smoke. |

## Compatibility

No corrective edits to schema version 17, protocol version 1, public request/response declarations, error catalogs, MCP catalogs, contracts/repeaters-v1.json, persisted representations or generated-ID formatting. Source indexing skips invisible bases for visible-copy IDs, preserving prior visible identity semantics. Historical proposal/tasks notes identify the earlier verification gaps without deleting its recorded evidence.

## Completeness

All 20 tasks are complete. Strict change/catalog validation passed before archive (24/24). Final format, strict workspace Clippy, full workspace/native tests, all bridge gates and Python tests passed; detailed counts and native tool configuration are in tasks.md. Required full golden and focused repeater tests executed successfully. The three deltas were synchronized and the change was archived to openspec/changes/archive/2026-09-08-fix-repeater-aliases-and-retained-preflight. Post-archive root:openspec-validate passed with Moon 2.3.3, including 231 policy tests and strict catalog 23/23; independent final catalog and whitespace checks passed.

## Findings

The audit corrected missing retained-asset reference validation, known-size non-vector affine checks, default RichText metadata and hidden-instance audio publication. The complete final gates pass after those corrections. No unresolved implementation/spec mismatch identified in the reviewed scenarios; no critical issues, warnings or suggestions remain. Sync, archival and post-archive checks are completed with fresh execution evidence rather than inferred from historical verification.

## Qualification — 2026-09-08

The preceding conclusion is limited to the scenarios actually exercised. A later review confirmed two uncovered defects: generated copies can exceed the 4,096 transition-fact limit, and slot overrides can introduce audio into a supposedly visual-only source. This invalidates any interpretation of the historical conclusion as complete repeater conformance. Fresh implementation and verification are tracked in [fix-repeater-transition-budget-and-effective-audio](../2026-09-08-fix-repeater-transition-budget-and-effective-audio/proposal.md); the original test results above are retained unchanged.
