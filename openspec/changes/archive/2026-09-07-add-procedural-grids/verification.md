# Verification: add-procedural-grids

Date: 2026-09-07. Issue: https://github.com/matiHirCab/AI-Open-Cut/issues/30.

The user approved the proposal, design, five delta specifications and tasks before implementation. This report records implementation verification using openspec-verify-change. The user subsequently approved the resulting contracts on 2026-09-07 with "Approve" in response to the final designated contract-review request. This records approval in this task, not a GitHub review submission.

## Scorecard

| Dimension | Result |
| --- | --- |
| Completeness | All implementation, documentation, conformance, contract approval, verification and archive tasks are complete. All required final gates passed. |
| Correctness | All eight requirements have automated behavior evidence across the twenty scenarios, as mapped below. The human contract-review clause is satisfied by the subsequent explicit approval in this task. |
| Coherence | Core owns descriptors, geometry, validation, migration and rendering semantics. Transport adapters forward typed edits. Existing dependency direction, shared stroke/paint behavior and error catalog are preserved. |

## Requirement and scenario evidence

Paths below are relative to the repository root.

| Requirement / scenarios | Implementation and automated evidence |
| --- | --- |
| Strict bounded grid descriptors: accept every pattern/paint; reject malformed/unsafe descriptors | `model/grid.rs` and `validation/grid.rs` under `crates/editor-core/src`; `contracts/procedural-grids-v1.json`. `canonical_grid_fixtures_and_atomic_failures`, `grid_replacement_definitions_drafts_and_strict_raw_decoding`, `duplicate_grid_records_fail_in_every_persisted_container` in `crates/editor-core/tests/procedural_grids.rs`; TypeScript `tests/procedural-grids.test.ts` and shared vector-primitives strict paint/stroke tests. |
| Canonical grid geometry and paint coordinates: lattice oracles; paint/edge semantics | `crates/editor-core/src/evaluated_scene/shapes/grids.rs::independent_grid_geometry_oracles` checks lines, angles, lattice membership, endpoints and dot circles. `render_artifact/shapes.rs::grid_intersections_local_paints_and_fractional_viewport` checks alpha accumulation, common gradient coordinates and fractional viewport coverage. Shared vector tests cover dash phase/reset, caps, joins and density; grid marks compile through that same contour/stroke path. Native grid conformance exercises transformed output. |
| Bounded procedural expansion and evaluated behavior: exact boundaries; derived work; deterministic composition | `grid_exact_mark_boundaries`, `grid_sampling_and_segment_budgets_fail_closed` and native grid conformance. Pre-expansion validation and evaluated-scene preflight cover hidden/unused descriptors, actual occurrence magnification, shared segment/density/surface budgets. Existing component/evaluated-scene tests cover recursive traversal, inheritance, clocks and common aggregate budgets; the new native fixture adds retimed component grids, drafts, resized outputs, hidden excessive magnification and static/constant-animation equality. No grid asset references are introduced. |
| Transactional procedural grid editing: visual lifecycle; replacement/definitions | `crates/editor-core/src/timeline.rs`; `grid_track_parent_aliases_visual_lifecycle_and_failures`, `grid_replacement_definitions_drafts_and_strict_raw_decoding`, `grid_alias_lifecycle_and_reopen`. These cover track/parent validation, complete replacement, omitted/null/wrong-target fields, definitions/drafts, supported visual edits, unsupported audio, history and reopening. |
| Atomic alias-aware grid transactions: alias creation/editing; failure atomicity | Core grid tests exercise aliases, references, revisions, locks, trailing failure and unchanged project/history. Headless protocol grid tests and `apps/agent-bridge/tests/grid-workflow.ts` exercise actual transport creation aliases, stale revisions and failed batches after grid creation, with equal state before/after failure. |
| Atomic schema 16 grid activation: mixed generations; invalid current/history; interrupted activation | `crates/editor-core/src/migrations.rs`, `grid_migration_current_history_and_future_rejection`, existing shape/SVG migration tests, `store::tests::supported_migrations_recover_every_publication_phase` and `supported_migration_before_journal_failure_preserves_generation`. Existing recovery tests iterate supported schema versions through the current version. All ran in the workspace suite. Source-version grid gating precedes relabeling; the schema-15 step only changes the version. |
| Typed discoverable grid workflows: source/packaged clients; transport failures; readiness | Canonical operation, MCP schema, capability and ownership catalogs; headless protocol tests; bridge contract/schema tests and existing readiness combinations. `grid-workflow.ts`, invoked by source and packaged smoke suites, creates all patterns/paints, replaces descriptors with every valid fixture, rejects invalid fixtures/stale revisions/trailing batch failure, edits aliases, undoes/redoes/reopens and completes a frame job. |
| Shared complete procedural grid rendering: every intent; fail before preparation; legacy output | `crates/editor-core/src/renderer/golden/grids.rs` runs frame/range/export at 240x120 and 160x90, root/draft scene equality, independent pixel assertions, repeatability, audio RMS/timing and excessive hidden work rejection. Full `native_golden_render_conformance` retains the established legacy, unsafe-input, backend-readiness and publication coverage. Grid range encoding uses export-equivalent medium/CRF23 quality to satisfy SSIM >= 0.99; `render_process::tests::grid_range_fidelity_preserves_legacy_encoding` protects legacy range settings. |

## Executed checks

| Check | Result |
| --- | --- |
| `cargo fmt --check --all` | Passed. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed. |
| `cargo test --workspace -- --test-threads=1` | Passed. Existing ignored helper/fixture-recorder tests remain ignored; required native conformance ran separately with backend enforcement. Subsequent additions to grid tests and the encoding-command test passed targeted runs. |
| Bridge `bun run typecheck` and `bun run lint` | Passed, including final MCP workflow additions. |
| Bridge `bun run test` | 384 tests passed in 18 files. |
| Bridge `bun run contracts:check` | Passed Rust/headless fixture suites and 320 TypeScript tests in 5 files, including the registered new grid suites. |
| Bridge `bun run test:integration` | 10 passed; rerun after final grid replacement/rollback workflow additions. |
| Bridge `bun run test:smoke` | 5 passed; rebuilt packaged executable and reran after final workflow additions. |
| `bun run apps/agent-bridge/scripts/run-python-tests.ts` | 10 Kokoro and 5 Whisper hermetic worker tests passed. |
| `cargo test -p opencut-editor-core native_grid_render_conformance -- --nocapture` | Passed with compatible FFmpeg/FFprobe and required backend enabled. |
| `cargo test -p opencut-editor-core native_golden_render_conformance -- --nocapture` | Passed, 229.51 seconds, including the new grids fixture and existing native golden suite. |
| Pinned OpenSpec 1.5.0 strict active-change and all-spec validation | Passed; all-spec run: 22 items passed. |
| `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` | Passed after approval, synchronization and archive: 22 living specifications validated and CI parity gate policy passed. The earlier active-inventory rejection is resolved. |
| `git diff --check` | Passed at final handoff. |

Native test environment used absolute paths resolved from:

```text
OPENCUT_FFMPEG_PATH=local-data/transform2d-tools/github-build/ffmpeg-8.1.2-essentials_build/bin/ffmpeg.exe
OPENCUT_FFPROBE_PATH=local-data/transform2d-tools/github-build/ffmpeg-8.1.2-essentials_build/bin/ffprobe.exe
OPENCUT_TEST_FONT_PATH=crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf
OPENCUT_GOLDEN_REQUIRED=1
```

The installed system FFmpeg did not support the repository's existing `filter_complex_script` invocation; the compatible local binaries above were used without changing that production behavior. Initial parallel workspace runs hit the existing short-lived process-sampler test under load; the full serial rerun passed. The first native grid comparison exposed SSIM 0.98425 with legacy range CRF28 versus export CRF23; the documented grid-only quality adjustment resolved it. None of these initial failures is treated as passing without its successful rerun.

Ignored local logs: `local-data/grid-workspace-tests.log`, `grid-contracts.log`, `grid-integration.log`, `grid-packaged-smoke.log`, `grid-native-golden.log` and `grid-moon.log`.

## Review and closure

1. **Designated contract review (task 5.5).** Review `contracts/procedural-grids-v1.json`, `contracts/headless-protocol-v1.json`, `contracts/mcp-surface-v1.json`, `contracts/contract-ownership-v1.json` and their governed Rust/TypeScript consumers listed in `.github/CODEOWNERS` and the ownership catalog. The owner is `@matiHirCab`. Public additions are `add_grid`, `timeline_add_grid`, `update_item.grid`, the grid item variant, `grid_items` and `grid_rendering`; persisted activation is schema 16 with no downgrade. The MCP catalog grows substantially because it expands the same strict tagged descriptor and paint alternatives through nested public schemas. Canonical parity passes; tests do not regenerate the catalog. Pre-implementation approval is recorded in proposal.md. The subsequent user "Approve" response to the final review request explicitly approves these resulting contracts and resolves task 5.5.
2. **Closure (tasks 6.5 and 6.6).** Designated review is approved with no requested revisions, and verification is finished. All five deltas have been synchronized into living requirements. The change is archived at `openspec/changes/archive/2026-09-07-add-procedural-grids/`. The final pinned Moon gate passed all 22 living specifications and the CI parity policy. `git diff --check` passed.

No implementation/specification mismatch was found in the automated behavior review. All checks passed. The approved and verified change is synchronized, archived and complete, with no unresolved verification issues.
