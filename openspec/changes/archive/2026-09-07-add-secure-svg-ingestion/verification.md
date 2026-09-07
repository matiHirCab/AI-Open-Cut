# Verification: add-secure-svg-ingestion

Implementation scope was explicitly approved in this task on 2026-09-07. The public contract remains protocol 1 with additive operations/capabilities; persisted schema 15 has the approved backup-based downgrade boundary. Contract-owner routing remains @matiHirCab in CODEOWNERS and the ownership catalog. No GitHub review or merge is represented as having occurred.

## Completeness, correctness and coherence

All five SVG requirements and the clarified intermediate schema-14 migration requirement have implementation and automated evidence. No unresolved implementation/specification mismatch was found. Synchronization, archival and the protected Moon gate are complete; results are recorded below.

| Requirement/scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| Fail-closed ingestion: valid static art; hostile/resource content | validation/svg.rs structural parser, decoded allowlists, bounded source and categorical errors | canonical_svg_ingestion_and_atomic_failures; hostile_xml_and_unsupported_features_never_normalize; headless svg_contract_reaches_core_and_preserves_batch_atomicity; MCP SVG workflow |
| Geometry/complexity: viewport, styles, command grammar, exact and exceeded bounds | validation/svg.rs; evaluated_scene/shapes.rs; render_artifact/shapes.rs | geometry_styles_and_viewbox_have_independent_oracles; every_numeric_and_work_budget_is_bounded; svg_composes_in_linear_light_and_clips_before_item_opacity; native SVG independent pixel oracle |
| Transactions: aliases, failure rollback and lifecycle | model.rs AddSvg/SvgItem; timeline.rs; core validation; existing store transaction orchestration | svg_alias_lifecycle_and_reopen; canonical_svg_ingestion_and_atomic_failures; normalized_svg_is_closed_and_revalidated_in_history_components_and_drafts; headless and both MCP smoke workflows |
| Persistence: migration and forged/future documents | model/svg.rs strict object decoding; model schema 15; migrations.rs source-schema guards; validation scopes | svg_migration_current_history_and_future_rejection; normalized_svg_is_closed_and_revalidated_in_history_components_and_drafts; existing supported_migrations_recover_every_publication_phase and transaction fault tests |
| Shared rendering/public evidence: output intents, lifecycle and readiness | common evaluated shape pipeline with ordered SVG subdraws; headless capability sets; typed bridge registration | complete native_golden_render_conformance (including SVG); native headless lifecycle; Transform2D native suite; canonical contracts gate; 382 TS unit tests; source and packaged MCP smoke workflows |
| Intermediate schema-14 activation | unchanged version-only shape activation followed by schema 15 under the same transaction | updated shape_items migration expectations; all supported/mixed-history migration suites; deterministic legacy golden references |

SVG parsing has no resource adapter, font lookup, FFmpeg syntax generation or file input. Only normalized reference-free geometry reaches evaluation. Existing ownership edges suffice; architecture tests pass without exceptions. Source XML stays out of persisted SVG items and parser diagnostics. Draft creation input remains the existing typed edit journal, validated through core before materialization. SVG adds no asset ownership or GC reference.

The native fixture uses an independent integer-coordinate pixel oracle, a shared SVG document with overlapping colored subdraws, centered viewport mapping, a retimed component, synthetic nonempty audio, original/moved/undo/redo/reopened states, and materialized draft previews. Existing shape and legacy golden references were not regenerated. The linear-light alpha unit oracle also checks source clipping and excess raster density.

## Executed checks

- `cargo fmt --check --all`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace -- --test-threads=1`: passed across desktop, core, architecture, public contracts and headless. The six pre-existing ignored helper/update/report entry points retain their intended harness behavior; configured native verification was executed separately.
- Bridge `bun run typecheck`, `bun run lint`, `bun run test`: passed; 382 unit tests.
- Bridge `bun run contracts:check`: passed, including headless tests, core vector/shape/SVG fixtures and 318 TS parity tests.
- Bridge `bun run test:integration`: passed, 10 tests including the SVG workflow.
- Bridge `bun run test:smoke`: passed, 5 packaged-runtime tests including the SVG workflow.
- From apps/kokoro-tts, `bun run ../agent-bridge/scripts/run-python-tests.ts`: passed, 10 Kokoro and 5 transcription worker tests.
- `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact`: passed in required native mode, 239.39 seconds.
- `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact`: passed in configured native mode.
- `cargo test -p opencut-editor-core --test transform2d`: passed all 13 tests in configured native mode.
- `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`: passed before archival, 21 items.
- `git diff --check`: passed.

Native verification used the existing local FFmpeg/FFprobe 8.1.2 toolset and DejaVuSans fixture, with OPENCUT_GOLDEN_REQUIRED=1 and all three dependency paths configured. The PATH FFmpeg build had removed filter_complex_script; the compatible local toolset resolved that prerequisite without production changes. An initial parallel workspace run failed the existing process-tree memory sampler; its isolated rerun and the entire serial workspace rerun passed. A concurrent TS provider cancellation timeout also passed on the complete rerun. No test threshold or warning suppression was weakened.

## Finalization

On 2026-09-07, synchronized the SVG capability and migration clarification into living specifications and archived this change. `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` passed through the unchanged protected policy: 231 policy tests, 21 living specifications, and CI parity gate validation. All 17 tasks are complete; no required check remains failed or skipped.
## PR #115 CI correction (2026-09-07)

Run 34130357925 failed strict Clippy on Linux, macOS and Windows with `chunks_exact_to_as_chunks` at the two polygon/polyline pair-iteration sites. Local validation had used Rust 1.93.0; CI reported the newer lint. Under approved tasks 2.2 and 6.1, both sites now use `as_chunks::<2>()` with the unchanged preceding even-length/minimum-point checks. Point order, coordinates, budgets and failure behavior are unchanged; no warning suppression, toolchain configuration or public contract change was introduced.

OpenSpec verification: the correction conforms to Explicit SVG geometry and complexity and the approved normalization design; existing canonical polygon/polyline and budget scenarios remain the automated evidence. No specification delta is needed. Local formatting, strict workspace Clippy, workspace tests and whitespace checks passed. CI contract parity, native render parity, integration/smoke and OpenSpec jobs passed on the preceding commit. The corrected commit will rerun the complete CI matrix.
The next CI run (34131933721) passed library compilation and exposed the same lint in the SVG RGBA pixel test. That final constant-size `chunks_exact` occurrence now uses `as_chunks::<4>()` and compares the same pixels. A repository-wide Rust search found no remaining constant-size calls. Strict workspace Clippy passed after this test-only correction.
