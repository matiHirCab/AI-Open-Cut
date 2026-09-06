# Verification: fix-shape-rendering-and-decoding

Concrete proposal, design, tasks and delta specifications were explicitly approved by the user with “Approve” on 2026-09-06. The three original regression tests were added first and each failed against the pre-fix implementation. They now pass.

## Assessment

The openspec-verify-change workflow compares completeness, correctness and coherence below. Implementation and required regression checks are complete. No correctness or coherence discrepancies remain. Living shape-items and vector-primitives specifications are synchronized and this follow-up is archived. The post-archive Moon policy gate passes. All 20 tasks are complete; no required check is failed or skipped. The original add-shape-items archive is preserved. No public field, schema version, dependency edge, tool signature or stable error was changed.

| Dimension | Evidence |
| --- | --- |
| Completeness | All three findings have failing-before/passing-after regressions, implementation and documentation. All 20 tasks, including verification, synchronization, archival and the final policy gate, are complete. |
| Correctness | All four delta requirements map to automated evidence below. Tests cover exact geometry, independent pixel/style equivalence, malformed raw input and side-effect preservation. |
| Coherence | Density, geometry and resource checks remain in evaluated_scene; coverage and local paint sampling remain in render_artifact; render_plan consumes evaluated sampling transforms. Lossless buffering is internal to model and replays typed Serde visitors. Existing core architecture tests pass. |

## Requirement and scenario evidence

| Requirement / scenarios | Implementation and tests |
| --- | --- |
| Closed bounded shape geometry: all seven variants; invalid shapes; anchor/clipping | Existing shape_items catalog, numeric-boundary and mutation suites remain green. `shape_anchor_is_independent_of_stroke_padding_and_offscreen_origin` retains its independent asymmetric anchor oracle. |
| Closed bounded shape geometry: fractional/degenerate anchors | `fractional_shape_anchor_equivalence` compares exact matrices for the two 0.5-pixel rectangle placements. `zero_extent_anchor_fallback_applies_only_to_paths` checks horizontal/vertical lines, fractional ellipse axes and move-only path fallback using independently chosen local anchor points. `affine` now preserves positive extents exactly. |
| Canonical shape evaluation and bounded raster work: nested occurrences, expensive work, exact vector semantics | Existing occurrence identity, component clocks, parent opacity, scene-segment overflow and fill/stroke/gradient unit oracles pass. `shape_density_is_composed_and_refinement_is_idempotent` checks skewed/nonuniform magnification, repeat finalization and maximum animated scale. `density_surface_limits_and_singular_value_oracles` checks independent singular values, exact 4096-square raster acceptance and the next larger raster rejection. `invalid_and_expensive_shapes_fail_before_artifact_io_for_all_facades` now includes density-induced overflow with zero artifact/process effects. |
| Scale-aware shape coverage: equivalent magnified geometry | `magnified_ellipse_preserves_output_coverage` compares the entire PAM output of a 2x2 ellipse at density 50 against 100x100 at density 1, within one 8-bit level. Native `transformed_conformance` checks exact decoded image equality, opaque center and black exterior corners. `magnified_thin_shape_checks_output_padding` checks composed 100x10 magnification of a thin rectangle without inflating its one-raster-pixel padding. |
| Scale-aware shape coverage: transformed styles and interval fidelity | `density_preserves_local_gradient_and_dashed_stroke_metrics` compares whole rasters against directly scaled geometry, gradient endpoints, width, dash length and phase. Native `transformed_conformance` exercises animated gradient scale, rotated/skewed/nonuniform dashed stroke, parent scale and component scale, comparing frame/range/export at 0, 500 and 900ms with unchanged SSIM >= 0.99. Static source matrices and animated sampling both compensate density. |
| Scale-aware shape coverage: reject excessive work | Finite raster dimensions and per-shape area are checked before allocation; checked aggregate byte accounting derives from the existing 16,777,216-pixel surface and 4096-layer limits, counting three RGBA buffers. No new arbitrary memory budget was introduced. Existing inclusive layer and scene-segment tests plus the density surface/no-artifact tests cover those composed limits. |
| Canonical JSON representation enforcement: strict nested records, every activated consumer | `raw_shape_edits_reject_duplicate_vector_fields` checks raw single and batch operations with identical and invalid-first/valid-last fields. `raw_shape_nested_records_and_update_fields_stay_strict` covers colors, points, gradient stops, radii, path points and strokes in creation/update payloads. `raw_shapes_reject_duplicates_in_current_and_retained_documents` checks current data, component definitions and undo/redo files, including byte preservation on failed loading. The real headless `raw_shape_duplicates_fail_before_single_batch_or_draft_mutation` checks INVALID_ARGUMENT duplicate-field rejection for single, batch and draft requests, including a valid first operation and unchanged project/history bytes. |
| Canonical JSON representation enforcement: compatibility/conflicts | Existing raw/Value vector fixtures, valid shape fixtures, optional/null tests, schema 1–13/current/history migrations, stale revisions, drafts, components and atomic rollback suites pass. Buffered object entries survive compatibility inspection and migration edits until typed vector deserializers run. Already-parsed Values retain their established inability to recover discarded duplicate keys. |

## Executed checks

| Check | Result |
| --- | --- |
| `cargo fmt --check --all` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS: 379 tests, 0 failed, 6 existing intentionally ignored capture/benchmark helpers across 22 suites. The subsequently added independent gradient/dash density test also passes separately. |
| `cargo test -p opencut-editor-core density_preserves_local_gradient --lib` | PASS: additional independent style equivalence regression |
| Native shape conformance | PASS: new transformed/frame/range/export comparisons and existing persisted-draft checks |
| `cargo test --release -p opencut-editor-core native_golden_render_conformance --lib -- --nocapture` | PASS in 60.44s with OPENCUT_GOLDEN_REQUIRED=1, local FFmpeg/ffprobe 8.1.2 essentials and checked-in DejaVuSans.ttf; legacy rule-card and flat audiovisual gates also pass |
| Bridge `bun run typecheck`, `bun run lint`, `bun run test` | PASS: 380 unit tests |
| Bridge `bun run contracts:check` | PASS: native headless/vector/shape checks and 316 TypeScript parity tests |
| Bridge `bun run test:integration` | PASS: 10 source MCP tests |
| Bridge `bun run test:smoke` | PASS: 5 packaged MCP tests |
| Worker `bun run ../agent-bridge/scripts/run-python-tests.ts` | PASS: 10 unittest and 5 pytest tests |
| Pinned OpenSpec `validate --all --strict --no-interactive` | PASS: 21 items before archival |
| `git diff --check` | PASS |
| Post-archive `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` | PASS: normalization, 231 policy tests, 20 OpenSpec items and CI parity gate policy |

## Fixture review and resolved failures

The shape-only evaluated-plan snapshot was deliberately recaptured to reflect raster density, compensated matrices and scale-dependent contours. Its recipe hash and original RGB oracle were preserved. The candidate image had mean absolute channel difference 0.12824/255 from that original; the required native gate passes against the original image and unchanged tolerances. The preview was visually inspected. All pre-existing legacy golden references remain unchanged.

The first native regression attempt lacked its temporary previews directory, and the first thin-shape regression used an invalid direct scale then an invalid stack index. These test setup errors were corrected; their focused and full checks pass. An initial Clippy useless-format finding was fixed. No failing checks were suppressed. No performance benchmark helpers were presented as conformance evidence.

Logs and the reviewed preview are in ignored `local-data/shape-fixes-verification/`; generated media and tool caches are not committed.
