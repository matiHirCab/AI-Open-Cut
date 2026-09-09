## Approval and assessment

The user approved proposal, design, delta specs and tasks on 2026-09-09 with “Approve”. The user also approved the final contract review on 2026-09-09; see contract-review.md. Verification used openspec-verify-change, including status and apply context for this change.

| Dimension | Result |
| --- | --- |
| Completeness | All 19 tasks complete. Final contract review approved, accepted deltas synced, change archived, and Moon gate passed. |
| Correctness | All five requirements and sixteen scenarios have automated evidence below. No uncovered normative requirement or automation exemption identified. |
| Coherence | Core owns normalization, validation, migration and evaluation. Transports expose the canonical typed surface. Existing slot limits and rendering resources remain compatible. |

## Scenario evidence

Paths are repository-relative. Every listed test passed on 2026-09-09. Core tests are in crates/editor-core/tests unless specified otherwise; bridge tests are in apps/agent-bridge/tests.

| Requirement / scenario | Implementation and automated evidence |
| --- | --- |
| Canonical bounded text documents: Preserve ordered Unicode runs | model.rs RichTextDocument and validation.rs validate_text_document; rich_text_documents.rs `canonical_documents_and_limits_preserve_unicode_and_reject_invalid_input` consumes canonical fixtures, retaining exact runs and projection. |
| Reject malformed and excessive documents | Same canonical test; `persisted_current_and_retained_documents_fail_closed`; bridge rich-text-documents.test.ts strict field/null/Unicode tests; template_slots.rs finite-value and unused-definition validation. |
| Accept inclusive limits and literal text | Core canonical test covers 256 runs and 4096 UTF-8 bytes, supplementary Unicode boundaries and literal markup-like content. Existing slot endpoint tests retain scalar-count compatibility. |
| Compatible reversible document edits: Use legacy and document inputs | timeline.rs AddText/UpdateItem; rich_text_documents.rs `edits_preserve_projection_aliases_history_and_atomic_failures`, `legacy_component_text_requests_remain_compatible_with_strict_persistence`; bridge rich-text workflow. |
| Reject conflicting or invalid targets | Core edit and lifecycle tests cover conflicts, missing references, non-text/locked/incompatible targets and stale revisions; protocol rich-text test and MCP workflow assert typed errors and atomic state. |
| Resolve aliases and roll back a batch | Core edit test, headless protocol `rich_text_documents_roundtrip_batches_drafts_and_failures`, and MCP rich-text workflow cover single-revision alias success and failed-batch rollback. |
| Preserve documents through lifecycle operations | Core `documents_survive_copy_split_move_trim_components_and_drafts` and edit test cover copy/component/split/move/trim, drafts, history and reopen. |
| Atomic schema 18 migration: Migrate mixed supported state and history | model.rs source-version preprocessing and existing migrations/store transaction; core `migration_upgrades_current_and_both_history_stacks_without_rewriting_reopen`, `every_supported_source_version_preserves_simple_text`, plus template_slots.rs `schema11_nested_history_migrates_and_schema12_fields_are_required`. |
| Reject invalid or future source data | Core `persisted_current_and_retained_documents_fail_closed`, existing store invalid-current/history tests and model migration tests cover source-version failures and authoritative byte preservation. |
| Recover every migration interruption | store.rs `supported_migrations_recover_every_publication_phase` and `supported_migration_before_journal_failure_preserves_generation` now include schema 17 text in the existing complete publication-fault matrix. |
| Additive rich text transport parity: Discover and round-trip documents | Canonical capability and typed request/output schemas; headless protocol rich-text test; rich-text-workflow.ts runs in integration and packaged smoke. |
| Preserve old clients and reject malformed requests | Core legacy component request regression; existing component-workflow.ts retains plain no-document records; bridge rich-text unit/workflow and protocol tests cover simple, document, batch and draft input. Native persisted text still requires a document. |
| Verify canonical cross-language evidence | contracts:check consumes rich-text-documents-v1.json, component definitions, headless capability and MCP surface catalogs through governed Rust/TypeScript tests. Provider contracts remain unchanged; hermetic Python tests passed. |
| Shared stored rich text evaluation: Preserve legacy rendered output | evaluated_scene.rs preserves the semantically plain path; renderer/golden/rich_text.rs compares legacy schema-17 and current evaluated scenes and exact frame pixels using fixed fonts, then frame/range/draft/export visual and audio parity. Existing rule-card conformance passed without baseline recapture. |
| Render stored styling and independent slot overrides | EvaluatedScene stored runs and typed slot override map; `root_slots_keep_independent_rich_runs_colors_and_defaults`, `rich_text_bindings_reach_local_and_outer_repeater_copies_without_scope_leakage`; native rich-text fixture checks visible run colors, exact draft pixels, frame/range/export SSIM >= 0.99 and decoded audio RMS <= 0.0001. |
| Fail safely on unavailable styled fonts | Native rich-text fixture uses an isolated directory without the requested bold face and asserts DEPENDENCY_UNAVAILABLE. Existing renderer font selection/confinement and path safety tests passed. |

New native conformance is included in the full required native golden driver, not only the dependency-optional workspace run.

## Check results

| Command | Result |
| --- | --- |
| `cargo fmt --check --all` | Passed. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed. |
| `cargo test --workspace -- --test-threads=1` | Passed, including migration/fault recovery, architecture and protocol tests. Seven pre-existing opt-in/ignored core tests retain their status. Native dependency-gated drivers were additionally forced below. |
| `cargo test -p opencut-editor-core native_golden_render_conformance -- --nocapture` with `OPENCUT_GOLDEN_REQUIRED=1` | Passed: full native driver, 430.34 seconds. |
| `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact` with required native resources | Passed. |
| `bun run typecheck` (apps/agent-bridge) | Passed. |
| `bun run lint` (apps/agent-bridge) | Passed, including formatting. |
| `bun run test` (apps/agent-bridge) | Passed: 390 tests across 20 files. |
| `bun run contracts:check` (apps/agent-bridge) | Passed: governed Rust suites and 326 TypeScript tests across 7 files. |
| `bun run test:integration` (apps/agent-bridge) | Passed: 11 tests. |
| `bun run test:smoke` (apps/agent-bridge) | Passed: 6 packaged tests. |
| `bun run ../agent-bridge/scripts/run-python-tests.ts` (apps/kokoro-tts) | Passed: 10 Kokoro tests and 5 Whisper tests. |
| `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` | Passed: 24 items. |
| `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` | Passed after approval, specification sync and archive: 24 items passed and CI parity gate policy is valid. The earlier active-change rejection is resolved. |
| `git diff --check` | Passed. |

Native runs used cached FFmpeg/ffprobe 8.1.2 at local-data/transform2d-tools/github-build/ffmpeg-8.1.2-essentials_build/bin and checked-in DejaVuSans.ttf. A focused native rich-text run also passed with FFmpeg 7.1.1. System FFmpeg lacked the existing filter_complex_script option; supported cached tools resolved this environment issue without changing production invocation. No baseline pixels, decoded audio or plan expectations were recaptured; only the rule-card recipe checksum follows required schema-18 document additions.

The parallel workspace attempt exposed an existing process-memory sampler assertion under concurrent native load; the complete serial run passed. Earlier failures in rich root affine routing and legacy component request normalization were corrected and all affected final suites rerun successfully. Temporary command logs are under Windows TEMP/opencut-rich-* and are not repository artifacts.

## Finalization

Final contract approval was received on 2026-09-09. No implementation, specification, design or test mismatch remains. All 19 tasks are complete. Accepted requirements were synced into rich-text-documents, project-persistence, agent-bridge and rendering-export. The change was archived at openspec/changes/archive/2026-09-09-introduce-rich-text-document. The post-archive Moon gate passed all 24 specification items and the CI parity policy check (task duration 2.939 seconds). The earlier archive-only failure is resolved. No outstanding critical findings, warnings or suggestions remain.
