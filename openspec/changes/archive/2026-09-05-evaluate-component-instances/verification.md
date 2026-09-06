# Verification: evaluate-component-instances

Verified on 2026-09-05 using the openspec-verify-change workflow. The user approved the proposal/design/deltas before implementation. Final contract/consumer review and archival were explicitly approved by the user after this report was presented. The approved deltas are synchronized into living specs, the change is archived, and the final archive-only Moon gate passes.

## Summary

| Dimension | Evidence |
| --- | --- |
| Completeness | 21/21 tasks complete; final owner approval recorded, specs synchronized, change archived and Moon validation passed |
| Correctness | All 11 normative requirements have automated coverage across the 21 scenarios; mapping below |
| Coherence | Core owns validation, migrations, editing, effective slots, derived clocks, scene expansion and render rules; transports delegate to core; architecture tests pass |

## Reviewable result

- `add_component_instance` and `component_instance_update` are additive headless edits/MCP tools, including aliases and drafts. Generic movement, duplication, static transforms, visibility, ordering, parenting and deletion retain root semantics.
- Schema 13 activates root overlay instances; schema 12 and retained history migrate atomically without changing content. Old-schema root instances and unknown future versions remain rejected.
- A bounded pure evaluator expands occurrences into the existing private scene with fractional composed clocks, effective slots, hierarchical matrices/order and isolated occurrence IDs. The process/path binding sidecar stays separate.
- Visuals retain local animation, internal transitions, media trims and styles. Audio uses bounded pitch-preserving tempo stages and explicit sample delay after clipping, preserving instance start time through mixing.
- Rich text retains styled runs and colors through measurement and rendering. Bold/italic resolve sibling font faces inside the configured font directory; unavailable faces fail with the existing DEPENDENCY_UNAVAILABLE error before artifact work. No implicit component fit or clipping was added.
- Canonical schemas, runtime catalogs, ownership lists, status capability and bridge/component/slot documentation are synchronized. The preparatory motion-graphics catalog remains fixture-only.

## Requirement and scenario traceability

Paths below are repository-relative. Existing tests are retained where unchanged lower-level rules are reused; new tests exercise their composition through root instances.

| Requirement / scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| Typed atomic root instance editing: aliased creation/update, atomic rejection, generic edits/drafts | `model.rs`, `timeline.rs`, `validation.rs`, existing store/draft transaction pipeline | `tests/component_evaluation.rs`: `canonical_instance_operations_are_closed`, `root_instances_aliases_atomic_failures_and_history`, `generic_edits_locks_drafts_and_unsupported_operations`; native draft materialization compared with committed preview; bridge `instance-workflow.ts` |
| Composed half-open clocks: fractional mapping and invalid mapped values | `evaluated_scene.rs`: `EvaluatedInstance`, occurrence/clock preflight; canonical `validate_instance`; `render_plan.rs` local expressions | `nested_fractional_clock_affine_and_repeated_identity`, `canonical_half_open_clock_cases_and_hidden_nonfinite_clock`; canonical semantic failure fixtures; native animated/faded nested visual fixture and retimed audio test |
| Instance-local slots/identity: independent overrides | Reused `resolve_component_slots`, path identities, rich-run metadata and preparation | `every_canonical_slot_kind_is_resolved_in_root_occurrences`, `root_slots_keep_independent_rich_runs_colors_and_defaults`, repeated identity test; all existing template slot constraint/special-key/media retention tests; native colored rich-run rendering |
| Bounded expansion: inclusive limits and unreachable invalid content | Saturating occurrence preflight, finite clock checks, expanded layer/resource/transition/pre-merge voiceover budgets, existing keyframe/geometry bounds | `bounded_shared_graph_rejects_exponential_expansion` covers 65535/65536/overflow including hidden trees; expanded visual/audio/transition tests cover exact 4096 and overflow; existing resource boundary test also runs through instance traversal; `bounds_pre_merge_voiceover_activity_ranges` covers 10000 and overflow, including repeated overlapping occurrences; existing per-channel keyframe boundary tests; hidden missing-definition test; unused invalid slot/resource tests |
| Hierarchical transforms/stacking: coordinate oracle, order and overflow | Instance/group/local matrix composition, per-descendant opacity, recursive order keys, existing root clipping/geometry finalizer | `component_dimensions_affine_oracle_and_contiguous_order` checks different sizes, normalized position, noncentral anchors, rotation/skew and local z-index containment with 1e-9 corner tolerance; repeated clock/matrix test; existing independent group/affine and geometry boundary tests; native nested pixel comparison |
| Shared audiovisual rendering: intent agreement, publication/path safety, visibility | Shared evaluated scene and render plan; bounded atempo; precise mapped transitions/automation/voiceover; existing safe preparation/publication | Native `nested_preview_range_and_export_share_pixels` consumes canonical animation/fade fixture, compares materialized draft exactly and frame/range/export at SSIM >=0.99; `retimed_audio_preserves_pitch_and_preview_export_pcm` checks trimmed 2x audio, 440 Hz pitch, initial silence and PCM RMS <=0.0001; `native_rich_text_runs_render_colors_and_missing_style_fails_cleanly`; `audio_expansion_limits_and_instance_vs_group_visibility`; tempo factor boundary test; existing artifact/process/path fault suites and full native golden conformance |
| Stored local timelines: empty/populated persistence and scoped references | Canonical component validation retained, root activation added | Existing `tests/components.rs` and headless component protocol tests; root instance alias/history tests; local identity/matrix tests |
| Stored slots preserve unrelated output: unreachable valid and invalid definitions | Original flat evaluation remains the entry path without root instances; canonical graph/slot validation retained | Existing `native_unused_definitions_preserve_frame_range_export_and_draft_output`, unused-slot render rejection tests, and full native golden suite |
| Atomic schema 13 activation: history/reopen and invalid/interrupted migration | `migrations.rs`, source document decoding, existing locked transaction/recovery | `old_schema_cannot_smuggle_root_instances`; `supported_migrations_recover_every_publication_phase` now includes schema 12 with populated nested definitions and explicit slot fields; existing current/history/future-version/migration preservation tests; new instance undo/redo/reopen assertions |
| Governed runtime activation: negotiate and parity | Runtime catalog, schema-13 catalogs, ownership index, headless status, bridge schemas/registrars | Canonical Rust/TypeScript operation acceptance tests; status/protocol parity; complete MCP structural schema and annotations comparison; `bun run contracts:check`; final owner approval recorded |
| Typed root workflows: real clients | Thin standalone registrars, shared batch/draft typed schemas | `instance-workflow.ts` reused by source integration and packaged smoke: create/update/aliases, input/reference errors, stale revision, rollback, render jobs, history/reopen and locked instance rejection |

## Commands and results

| Check | Result |
| --- | --- |
| `cargo fmt --check --all` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace -- --test-threads=1` | PASS, including native golden conformance and all enabled native integration render tests |
| `cargo test -p opencut-editor-core --test component_evaluation` after the final draft-preview assertion | PASS: 7 tests, native media enabled |
| `bun run contracts:check` from apps/agent-bridge | PASS: TypeScript typecheck, headless Rust tests, 20 canonical contract tests |
| `bun run lint` | PASS: 48 files |
| `bun run test:unit` | PASS: 84 tests / 14 files |
| `bun run test:integration` | PASS: 9 tests, including instance workflow and locks |
| `bun run test:smoke` | PASS: 4 packaged tests, including instance workflow and locks |
| `bun run ../agent-bridge/scripts/run-python-tests.ts` from apps/kokoro-tts | PASS: 10 unittest tests and 5 pytest tests |
| `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` | PASS: synchronized specs and active change validated (18 items), followed by archive-only Moon validation (17 living specs) |
| `moon run root:openspec-validate` (pinned 2.3.3) | PASS after approved archival: normalization, policy tests, strict spec validation and parity gate policy all passed. The earlier expected active-change failure is resolved |
| `git diff --check` | PASS |

Local diagnostic logs are ignored under `target/issue24-*.log`; no generated binaries/media/caches are part of this change. FFmpeg 7.1.1 and ffprobe were supplied through OPENCUT_FFMPEG_PATH/OPENCUT_FFPROBE_PATH, the checked-in DejaVuSans font through OPENCUT_TEST_FONT_PATH, and OPENCUT_GOLDEN_REQUIRED=1. The installed FFmpeg 9 rejects the repository's existing filter_complex_script option, so the supported 7.1.1 build was used without changing unrelated renderer compatibility. Moon 2.3.3 was downloaded into the ignored tool directory and matched its published SHA-256.

An earlier concurrent Rust run failed the existing process-memory sampler assertion under load. The complete serial rerun passed without changing/skipping that assertion. The five existing ignored library tests are explicitly designated helper processes/benchmarks, not waived conformance checks; their owning tests exercise the required helper subprocesses. No required new scenario is classified as technically impossible to automate.

## Lifecycle completion

The user explicitly approved the final contract changes and archival with "Approve" after the implementation and verification report were presented. This satisfies the designated contract/consumer review gate identified in the approved change.

Using openspec-sync-specs and openspec-archive-change, all eleven approved requirement blocks were synchronized across seven capabilities (one new capability and six updates), preserving unrelated requirements and scenarios. The change was archived to `openspec/changes/archive/2026-09-05-evaluate-component-instances` on 2026-09-05.

The final `moon run root:openspec-validate` passed on the archive-only tree with the repository-pinned Moon 2.3.3. Plugin network access was required by the local sandbox; no policy check was bypassed. All 21 tasks are complete. No unresolved implementation/spec mismatch or completion gate remains.
