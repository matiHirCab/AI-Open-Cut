# Final implementation verification — 2026-10-03

## Draft CI helper correction

Draft PR136 first head `17a7832c351b8681b7f937fb7f1b7fa7e3805080` exposed Clippy's `chunks_exact_to_as_chunks` diagnostic in two new native-test helpers on all three OS jobs. The helpers now iterate the same complete fixed-size arrays and decode the same little-endian samples, preserving ordering and ignored suffixes. No runtime behavior, specification, assertion, oracle, tolerance or CI policy changed; no lint suppression was added.

Independent Sol medium implementation review approved this equivalent rewrite in `/tmp/issue47-ci-helper-review.md`; this is delegated agent review, not human CODEOWNER approval. Final correction checks passed: formatting, strict workspace/all-target Clippy, complete workspace tests, both required real native edit-render tests (2/2, 163.70s), and protected OpenSpec bootstrap. Evidence is `/tmp/issue47-ci-chunk-fmt-final.log`, `/tmp/issue47-ci-chunk-clippy-final.log`, `/tmp/issue47-ci-helper-workspace.log`, `/tmp/issue47-ci-chunk-native-final.log`, and `/tmp/issue47-ci-helper-bootstrap.log`. The application and contracts are unchanged from the fully verified implementation. Exact corrected-head remote CI is tracked in external delivery evidence; the first failed head is not counted as passing CI.

The root completed the repository OpenSpec verification workflow using the CLI artifact graph and apply context, inspected approved artifacts, source and scenario traceability, and independent Sol medium reviews. The reviewer notes below are dated evidence snapshots; this final section closes their pending implementation checks. Human CODEOWNER review is still pending draft delivery and is not claimed.

| Dimension | Final assessment |
| --- | --- |
| Completeness | 11/11 implementation, verification and lifecycle tasks complete; living specifications synchronized and this change archived. Nine normative requirements and all 36 scenarios have automated coverage. |
| Correctness | No unresolved requirement/scenario mismatch or known defect after independent reviews and mandatory verification. |
| Coherence | Retained source functions, canonical shared samplers/validation, store-owned recovery validation, atomic historical migration and governed consumer ownership match the approved design. |

No critical, warning or suggestion item remains for this implementation. Reserved schema30 is the approved integration boundary, with a validated adapter required when motion-pack work integrates; it is documented and tested rather than silently relabeled. Living specifications synchronized and verified change archived. Post-archive strict validation, protected Moon gate and isolated CI bootstrap passed (37 strict items and 377 policy tests). Draft publication, designated review request and exact-head CI remain explicitly tracked in delivery-checklist.md and external delivery evidence.

## Executed verification

Full logs are preserved outside the checkout at `/tmp/issue47-logs` and in the delivery evidence bundle. Earlier failures and superseded/cancelled runs remain preserved; they are not counted as passing gates.

| Check / exact invocation | Result and log |
| --- | --- |
| `cargo fmt --check --all` | Passed; `verified-format.log`. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed; `verified-clippy.log`. Only the existing upstream future-incompatibility notice remains, with no suppressed local warning. |
| `cargo test --workspace` | Passed on final source/test tree; `verified-workspace.log`. |
| Bridge `bun run typecheck`, `bun run lint`, `bun run test` | Passed; `final-ts-typecheck.log`, `final-ts-lint.log`, `final-ts-unit-with-init-reaping.log`: 433 pass, one existing native opt-in skip separately executed below. Relevant TypeScript inputs remain unchanged. |
| Bridge `bun run contracts:check` | Passed including required Rust consumer targets and 363 tests across 14 TypeScript files; `verified-contracts.log`. |
| Bridge `bun run test:integration` | Passed all 15 real MCP workflows with unchanged timeouts; `verified-mcp-integration.log`. Serial retry resolved earlier resource-contention timeouts. |
| Bridge `bun run test:smoke` | Passed all nine tests against real release-packaged artifacts; `verified-packaged-smoke.log`. |
| Relevant hermetic Python worker unit/pytest commands | 10 unittest plus five pytest pass; `python-hermetic.log`. Worker inputs unchanged. |
| Native `cargo test -p opencut-editor-core --test animation_channels` | Required native flag enabled; 100 pass including the new independent visual/audio parity tests; `stable-native-animation-complete.log`. |
| Native `cargo test -p opencut-editor-core --test transform2d --test font_resolution` and headless lifecycle exact test | Passed; `stable-native-transform-font.log`, `stable-native-lifecycle.log`. |
| `cargo test --release -p opencut-editor-core --lib renderer::golden::native_golden_render_conformance -- --exact --nocapture` | Passed all reviewed suites and sampled capture; `final-release-golden-library.log`. Same authoritative harness selected through its library target; goldens/tolerances unchanged. |
| `cargo test -p opencut-editor-core --lib renderer::golden::validate_external_performance_report -- --ignored --exact` | Passed strict actual report validation; `final-report-validation.log`. |
| Required raster cache core, instrumented headless worker/build, native bridge, default rebuild and headless compatibility sequence | Passed 13 core, three instrumented worker and one native bridge test, then default headless restoration; `stable-native-cache-*.log`, `stable-default-*.log`. |
| Focused migration/recovery/architecture/crop/shape tests | Passed post-correction architecture21, journal5 including69 no-write cases, migration phases, mixed-clock crop and shape10; external `issue47-recovery-boundary-*.log`, `issue47-crop-correlation-final.log`, `issue47-shape-retained-adjusted.log`. |
| `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` | 37 items pass before archive; `final-prearchive-strict-all.log`. |
| `moon run root:openspec-validate` before archive | 377 policy tests plus strict validation pass; rejected only this active change as expected, not gate success; `verified-prearchive-policy.log`. |

Environment evidence: pinned Bun1.4.0/Rust1.97.0/Moon2.3.3; reviewed repository DejaVu font hash; actual FFmpeg/FFprobe; an init/subreaper wrapper adopts and reaps orphan children in this container, without changing assertions or timeouts; desktop links use environment-only aliases to installed runtime libraries. Moon uses the actual pinned binary where this container's nested proto shim otherwise recursively resolves itself. No workflow, policy, dependency pin, test tolerance or golden baseline was weakened. Renderer evidence remains relevant after the later recovery-only ownership correction, independently covered by the pre-replay journal matrix and final workspace checks; final release golden ran after that runtime correction. Remote CI results are separate delivery evidence.

---

# Issue 47 independent final conformance review

Reviewer: delegated agent `/root/implementation_review`; this is agent source/conformance review, not human CODEOWNER approval. Reviewed final runtime and governed contracts against `openspec/changes/preserve-animation-edit-semantics`, including reconciled animation-channels, animation-loops and animation-presets lifecycle deltas. Base: `8c6c7c93`. No repository source edits made by this reviewer.

## Outcome and evidence boundary

No known runtime correctness defect or unmapped normative scenario remains in the reviewed implementation. The final mixed-retained-clock crop coverage gap is closed: `evaluated_scene/extended_visual/review_regressions.rs:556::retained_crop_correlation_requires_equal_clocks_and_rejects_atomically` directly certifies equal-offset complementary channels successfully, rejects mismatched offsets with INVALID_ARGUMENT, and verifies Core edit rollback preserves logical state and exact project/history bytes. The independent expected local t=0 sum is0.4+1=1.4; retained sourceDuration1001 correctly bounds the terminal1000key. The focused final run passed in `/tmp/issue47-crop-correlation-final.log` (one test, zero failed). Earlier transient compile-failure logs remain failures and require stable full-gate reruns.

Completion approval remains pending mandatory stable-tree verification, OpenSpec verification/synchronization/archive and protected post-archive gates, publication and exact-head CI. This report does not claim those pending gates passed. Earlier failed workspace/contracts runs are genuine failures; corrected preset tests must be rerun through those full gates.

Inspected passing focused evidence: `/tmp/issue47-edit-tests.log` (11 edit integration tests), `/tmp/issue47-clock-tests.log` (4 source sampler tests), `/tmp/issue47-invalid-journal-tests.log` (journal test with 69 combinations), `/tmp/issue47-journal-tests.log` (reserved schema30 recovery), `/tmp/issue47-migration-tests.log` (2 publication-fault migration tests), `/tmp/issue47-negative-budget5.log` (negative-clock resource budget), `/tmp/issue47-native-audio-final.log` (native audio), `/tmp/issue47-presets-adjusted.log` (29 corrected preset tests), `/tmp/issue47-crop-correlation-final.log` (direct mixed-clock certification and atomic rollback). Native visual passed earlier within `/tmp/issue47-native-render4.log`, which also contains the then-failing audio test and must not be described as a passing complete suite. The stable native/integration gates are underway at report time.

Inspected stable strict Clippy success in `/tmp/issue47-logs/stable-clippy.log`; TS unit evidence in `/tmp/issue47-logs/final-ts-unit-with-init-reaping.log` is 433 passed and one existing skipped test, using the actual init/subreaper environment. Root owns type/lint and complete mandatory verification evidence.

## Ownership references

* Edit source retention: `crates/editor-core/src/timeline.rs`, TrimItem line1367, SplitItem line1646, DuplicateItems line1767, `remap_animation_self_targets` line2082, `retain_animation_clock` line2099. New/replacement channels and preset compilation omit clocks; replacement legacy keyframes clear only legacy clock.
* Source samplers: `animation.rs`, source mapping line406/417, integer loop map line425/430, fractional map line465, scalar/compound samplers line330/337/510/550, retained legacy activity line919. Relative selected segments explicitly clear clock to avoid applying it twice.
* Contracts: `model/animation_channels.rs::AnimationClock`, `model.rs::VisualProperties`, `validation/animation_channels.rs`; canonical `contracts/animation-channels-v1.json`, project-schema claims in governed fixtures, MCP surface and ownership catalogs; bridge `schemas.ts`; guide `docs/animation-channels.md`.
* Evaluation/rendering: `evaluated_scene.rs::evaluate_keyframes`, media audio retained flag, common legacy clock propagation; `evaluated_scene/extended_visual.rs::sample_scalar`; `render_plan.rs::retained_time_expression` and `append_audio_layer`. Retained non-instance audio uses local automation then physical delay then global ducking; inherited and unclocked branches remain unchanged.
* Certification: `extended_certification.rs::source_scalar_bounds` line623, full-source ancestor/effect envelopes, and `correlated_sum` line631 requiring equal clocks; fractional crop envelopes use mapped scalar bounds; V `retained_crop_correlation_requires_equal_clocks_and_rejects_atomically` proves equal-clock acceptance and mismatched-clock rejection/rollback. Shared legacy count helpers in `validation.rs` preserve root 10k/property and component 10k/item limits.
* Persistence: `model.rs::reject_animation_clocks` line479; `migrations.rs::migrate_project_documents` migrates cloned current/all history atomically and explicitly supports1–29/current31; `persistence.rs::validate_transaction` performs structural/identity/supported-version validation. `recover_transaction` then invokes its required `Fn(&Project,&History)->Result<(),CoreError>` callback before its only recovery replay call, mapping callback failures through `as_recovery_error` to PROJECT_RECOVERY_FAILED. The sole caller, `store.rs::prepare_project_data` line1631, supplies the canonical retained-source validator for current plus every undo/redo snapshot; that validator traverses root/component items. This preserves the approved ownership direction: persistence has no dependency on validation and cannot omit the required callback.

Test abbreviations below are owning test files/functions, not claims every named test has completed in the latest full run:

* E = `tests/animation_edit_semantics.rs`.
* N = `tests/animation_edit_render.rs`.
* S = `animation.rs::retained_clock_tests` and baseline animation sampler tests.
* C = `tests/animation_channels.rs` (also imports E and N into existing protected native target).
* P = `tests/animation_presets.rs`.
* J = `store.rs` journal/migration regression tests.
* V = `evaluated_scene/extended_visual/review_regressions.rs`.
* B = bridge `animation-channels.test.ts` and `animation-edit-workflow.ts`, invoked by MCP integration and packaged smoke.

## Delta requirement/scenario traceability

### animation-edit-semantics: Preserve continuous source animation through editing

Owning behavior: timeline retention and target remapping; canonical source samplers; preset compiler produces new unclocked property channels only. Original records and provenance remain descriptive compilation attribution.

| Scenario | Automated coverage and independent evidence |
| --- | --- |
| Split nonlinear source segments and loops | S `retained_clock_preserves_arbitrary_curves_finite_phases_and_fractional_interiors` covers arbitrary Bézier/spring/linear/hold, repeat and ping-pong offsets; E split/trim/history; N independent Bézier cubic, legacy quadratic and finite ping-pong expected pixels across intents. |
| Trim and duplicate retained animation | E `split_trim_duplicate_preserve_exact_records_history_reopen_and_independent_replacement`; S negative clock/source held endpoints and offset tables; E standalone implicit duplicate; N raw duplicate/draft audio envelopes. |
| Preserve all active value types and audio | E `every_visual_item_retains_typed_and_legacy_source_records_and_group_restrictions`, E compound self targets; S scalar/RGBA/point/path/gradient exact source comparisons; N gain/legacy-volume/video visual-clock audio and compound tint. Existing C incompatible targets remain rejected. |
| Replace one preset property independently | E `replacing_one_preset_property_keeps_other_retained_typed_and_legacy_clocks`; P replacement/static/collision tests. |
| Remap copied self-owned graphic channels | E `compound_self_targets_remap_on_split_duplicate_and_effect_targets_stay_local` covers path points, path trim, gradient and tint; narrow helper changes only owning graphic ID, preserving kind/scope and effect IDs. |

### animation-edit-semantics: Bounded retained animation contracts

Owning behavior: strict clock deserialize plus checked mapped-window/source bounds; existing whole channel/legacy value/order/count/curve/loop/target validation shared by mutation and journal recovery. Typed source keys strictly below source duration; legacy keys may equal it.

| Scenario | Automated coverage and independent evidence |
| --- | --- |
| Reject malformed retained source records | E canonical structural cases, source/window/nonempty errors and persisted current/history rejection; B canonical clock cases for typed and common visual schemas. C baseline malformed channel/curve/loop/complexity constraints continue to apply. |
| Reject invalid retained-clock recovery journals before replay | J `invalid_retained_clock_journal_leaves_all_authoritative_bytes_unchanged`: current/undo/redo × root/component, window/nonempty/source/order/spring/target/loop/legacy-order and root/component count cases; exact current/history/journal bytes unchanged; error remains PROJECT_RECOVERY_FAILED. |

### animation-edit-semantics: Transactional edits and marker lifecycle

Owning behavior: existing transactional edit staging/revisions/history/aliases and marker helpers; trim clears marker only on numeric start change, split clears both, duplicate shifts expression.

| Scenario | Automated coverage and independent evidence |
| --- | --- |
| Commit and recover an alias batch | E `marker_alias_batch_retained_clock_history_and_later_failure_preserve_lifecycle` exercises alias-created animated trim and subsequent split/duplicate marker/undo/redo/reopen. B batch-created source+split advances once, then trim/duplicate/undo/redo/reopen validates records; baseline C alias transaction/history tests cover ordered multioperation staging. Coverage is compositional; the exact create+trim+split+duplicate sequence is not one single bespoke test. |
| Roll back validation and reference failures | E `clock_failures_and_edit_errors_are_atomic`, `locked_clock_edits_preserve_existing_track_error_and_history`; fresh alias rollback reaches intended boundary split after retention and compares durable bytes. B rejected alias batch asserts VALIDATION_FAILED/retryable:false and unchanged state. Existing inherited publication-boundary resource inventory tests retain coverage. |
| Preserve an independently duplicated animated audio item | E `standalone_duplicate_materializes_equivalent_implicit_clock_only_on_copy` proves original absent clock unchanged and copy equivalent clock0; N raw-duplicate gain/volume/visual media proves nonzero placement, silence and local phase. |

### animation-edit-semantics: Shared renderer and contract conformance

Owning behavior: shared evaluated/source samplers, offset-before-loop FFmpeg expressions, retained non-instance audio physical placement, unchanged inherited/unclocked branches. N is imported in C so protected native gate sets required tools and executes it; missing tools fail explicitly under required flag.

| Scenario | Automated coverage and independent evidence |
| --- | --- |
| Preserve retained audio placement and global ducking | N `native_edited_audio_keeps_independent_gain_and_legacy_volume_envelopes`: separate valid gain and volume fixtures; real video with only visual retained clock; original/implicit copy/split/trim/duplicate/materialized draft, silence gaps, later range1500–2000, music/quiet voiceover ducking. Independent sine power/analytic envelope and probed source packet clock follow existing eval=frame semantics; tolerance0.002 unchanged. |
| Compare edited output with independent source expectations | N visual test uses independent cubic/quadratic/ping-pong equations, independent linear-light sRGB tint conversion, constant spatial reference and centroid+MSE assertions for frame/range/draft/export; inherited timeScale0.751 retains fractional samples. Native expression tests independently cover Bézier/spring and loop equations; V independently checks legacy quadratic/fractional and ping-pong turn/exhaustion. |

### project-persistence: Atomic schema31 retained-clock migration

Owning behavior: current/component/all retained history migration under existing lock/staged journal publication; pre31 raw clock rejection, reserved30 rejection before replay, unknown future rejection. Schema30 pack provenance stays reserved; later integration requires validated adapter and preserved tagged provenance/history, not relabeling.

| Scenario | Automated coverage and independent evidence |
| --- | --- |
| Migrate current components and retained history | E `schema29_migrates_root_components_and_all_retained_history_atomically`, exact keys and component source records retained, repeated reopen stable; E edits/undo/redo/reopen restores explicit clocks. P corrected schema28 migration expects current schema constant31. |
| Fail closed and retain disk state | E reserved/future/pre31-clock/unsafe-window current/history no-write matrix; J reserved30 journal current/undo/redo; J full invalid retained-source matrix; J `supported_migrations_recover_every_publication_phase` includes29 and existing historical schemas, plus pre-journal failure preserving generation. |

### animation-channels: Bounded channel keyframes (modified)

Owning behavior: `validate_channels` uses source duration only when valid clock present, preserving all original sequence/value/curve/complexity/target constraints.

| Scenario | Automated coverage |
| --- | --- |
| Reject invalid channel sequence | C canonical limits and parameterized/channel validation tests; E invalid retained source bounds/nonempty/window; J invalid retained order/count before replay. |
| Reject deferred timing and malformed curves or loops | C canonical curve cases, `parameterized_curve_bounds_and_shapes_are_enforced`, loop edits; J invalid spring/loop/target retained journal cases; strict serde and bridge schemas reject unknown fields/variants. |
| Preserve existing curve payloads | C canonical fixtures/Rust roundtrip and B canonical wire fixtures; S fixed linear/hold and loop samples; clocks omitted retain default behavior. |
| Validate retained source keys beyond an edited window | E source-record preservation after split/short trim; P corrected trim/split tests now accept retained source keys; E sourceDuration bound rejection and C original out-of-item-duration rejection. |

### animation-channels: Deterministic channel sampling (modified)

Owning behavior: checked integer/fractional source map exactly once; existing Bézier40-step inversion/spring closed forms and canonical bounds; samplers shared by evaluated scene/native expressions.

| Scenario | Automated coverage |
| --- | --- |
| Sample a channel at a boundary | S source seam/exact key tables; N frame/range/draft/export source expected equations; C baseline native same-channel intent tests. |
| Sample each parameterized curve | S `bezier_fixed_samples_and_exact_boundaries`, `spring_regimes_have_fixed_scalar_results`, retained arbitrary curve/phase table; render-plan native independent/precision tests. |
| Bound spring overshoot | C curve bounds; S spring envelopes and fixed values; existing native parameterized bound tests. Retention delegates to these unchanged clamps. |
| Preserve an unanimated project | Existing golden and deterministic plan baseline tests; source map omittedclock identity and unchanged unclocked audio branch. Full stable golden result remains pending. |
| Preserve retained curve interiors and fractional source time | S retained scalar/compound fractional tables; V retained legacy+typed independent samples; N nested0.751 fractional independent pixel expectations. |

### animation-loops: Deterministic item-local loop evaluation (modified)

Owning behavior: source offset maps once before unchanged loop source mapping; first-key/span anchor and finite budget stay source-based; fractional map avoids integer rounding; constant-work modulo/finite arithmetic.

| Scenario | Automated coverage |
| --- | --- |
| Sample exact forward seams and exhaustion | S `loop_phases_are_item_local_and_exact_at_seams` and retained repeat offsets; C `native_later_cycle_frame_range_and_export_agree`; native render-plan loop scalar expectations. |
| Sample ping-pong turn and return | S arbitrary Bézier/spring retained ping-pong table and fixed loop phases; V fractional turn/exhaustion; N finite ping-pong independent output. |
| Preserve independent properties | C independent visual/audio loop fixtures and absent static fallback, S retained loop/unlooped table; N simultaneous unlooped Bézier + ping-pong opacity + legacy scale. |
| Preserve fractional repeat phase | S extended fractional clock test and existing inherited timing/golden fractional-loop fixtures; render-plan native fractional expression tests. |
| Preserve fractional turns and finite completion | V exact independently expected fractional turn/completion table; S retained fractional table with nonzero firstkey; baseline native fractional loop expression expectations. |
| Preserve existing integer and independent audio sampling | S exact integer loop phases and large-integer regression; C baseline native audio loop parity; N independent audio packet-clock envelopes retain original eval=frame sampling. |
| Retain midcycle phase and finite exhaustion through edits | S offsets175/350/700 × repeat/ping-pong finite/infinite × integer/fractional source times; E source records retained through edits; N half-open source finite-ping-pong output and later windows. |

## Preset test corrections and compatibility assessment

All four reported stale expectations are legitimate consequences already covered by approved behavior, not reasons to change runtime back:

1. `unrelated_edits_copies_and_duration_failures_preserve_known_source` previously expected a short trim or mid-preset split to fail because original keys no longer fit the item. Retained-source validation deliberately permits them. Corrected test now verifies preserved keys, source clocks and original compilation provenance; genuine zero-duration/boundary split failures stay VALIDATION_FAILED and unchanged bytes.
2. `successful_split_preserves_only_exact_local_channels_and_labels` previously compared channels byte-for-byte without new clocks. Both source records are preserved, but left clock0/right clock500 must now be present; corrected test asserts those records and unchanged provenance.
3. Schema28 current/component/history migration now ends at schema31, not literal29. Corrected expected schema uses PROJECT_SCHEMA_VERSION.
4. Legacy slot named `animationPresetProvenance` migration still preserves that unrelated user slot; only the expected current-schema literal changes29→31. Raw pre31 clock detection is targeted at actual timeline item fields, avoiding false rejection of unrelated slot data.

The new fields are additive public contracts; schema31 is the deliberate persisted-reader boundary. Known exception is reserved30 until motion-pack integration. Original animation channels without clocks keep their old validation/output. Historical preset provenance continues to describe original compilation rather than edited-window key locality, expressly covered by the approved change.

## Remaining work at report time

* Mixed-clock crop rejection/equal-clock acceptance regression is complete and focused-pass verified; rerun mandatory full checks invalidated by its final addition.
* Rerun full workspace and contracts gates after four preset test corrections; earlier failure logs remain evidence, not passing gates.
* Await complete stable native, MCP integration, packaged smoke, golden and remaining mandatory checks.
* Update tasks accurately from real evidence, perform OpenSpec conformance verification, synchronize/archive, then run protected post-archive gate and strict all-spec validation.
* Draft publication/designated human reviewer request and exact-head CI remain delivery work; no merge/deploy authorized or claimed.

## Final recovery ownership boundary amendment review

The mandatory architecture gate correctly caught the earlier forbidden persistence→validation dependency. The final correction is independently source-reviewed: persistence accepts a required owner-neutral callback, structurally validates the journal, invokes the callback on the decoded transaction before replay, and preserves existing PROJECT_RECOVERY_FAILED translation. Store orchestration is the sole recovery caller and injects canonical validation for the current project and every retained undo/redo snapshot; domain constraints remain shared, with no parallel validation or added owner dependency. The design explicitly records this boundary. `git diff --check` for persistence/store passes. `/tmp/issue47-recovery-boundary-architecture.log` reports21 passed/0 failed, including the actual approved dependency-matrix test. Post-boundary evidence is now inspected and passing: `/tmp/issue47-recovery-boundary-journal.log` reports5 passed/0 failed, including reserved30 recovery, the69-combination retained-source matrix, older schema18 recovery, pre-journal migration failure and preset resource recovery; `/tmp/issue47-recovery-boundary-migration.log` reports1 passed/0 failed for all supported publication phases. These focused passes validate the corrected boundary; mandatory full stable checks remain separate completion gates.

## Approved animation-presets lifecycle reconciliation

The complete MODIFIED `Provenance follows primitive lifecycle` requirement in the new animation-presets delta was independently approved by `/root/spec_review` and the exact approval is recorded in approval.md. This is a specification reconciliation with existing primitive trust rules and the approved issue47 retained-clock behavior; it introduces no new runtime behavior. Earlier blanket short-duration/split failure wording contradicted retained-source preservation and is now qualified by effective source bounds, edit boundaries and candidate safety.

Owning code: `timeline.rs::SetAnimationChannels` unconditionally clears all provenance; `timeline/animation_presets.rs::apply` replaces the identity's record and produces an unclocked fresh compiled channel; `reconcile_raw_tracks` line91 clears submitted records and copies only previously known records for the same item/property when serialized complete channels match exactly. Complete-channel serialization includes retained clocks, target records, loops and signed binary64 representations, so a changed explicit clock cannot gain a trusted old label. Scope trust stays with component create/update call sites and their prior tracks. Split/trim/duplicate preserve known attribution by preserving primitives/effective source clocks. Existing item deletion and history mechanisms keep records together with items.

| Lifecycle scenario | Owning automated evidence |
| --- | --- |
| Clear labels through the raw setter and restore by undo | P `collision_replace_order_and_raw_clear_are_explicit` verifies identical setter replacement clears source labels, undo restores exact prior item and redo clears again; P `replacement_preserves_order_static_state_and_legacy_collisions` includes empty raw setter. No clock field changes this unconditional-clear path. |
| Preserve copies and unrelated edits | P `unrelated_edits_copies_and_retained_duration_edits_preserve_known_source` checks move/static/visibility/trim/copy/deletion/undo source attribution; P `root_group_component_and_parenting_use_existing_compatibility` and replacement tests cover parenting/base/static behavior. E exact retained duplicate and standalone implicitclock materialization prove equivalent source-clock copying without changing original records. |
| Reconcile raw component replacement | P `raw_component_replacement_strips_forgery_and_preserves_only_known_exact_channels` covers new component submitted-label stripping, known unchanged channel preservation despite forged supplied label, changed channel clearing, signed-zero identity changes and undo. The tested serialized-channel equality is complete and now inherently includes clock fields; no separate trust bypass or normalization was added. |
| Preserve existing duration and split outcomes | P corrected `unrelated_edits_copies_and_retained_duration_edits_preserve_known_source` retains genuine zero-duration and boundary-split atomic failure assertions; E malformed retained source/window and bounds/locked/stale errors; C unclocked channel timing/curve limits; P malformed/orphan provenance and candidate budget rollback tests. Accepted raw primitive changes clear their identity through setter/reconcile behavior rather than implicit retiming. |
| Preserve preset source attribution through retained edits | P `successful_split_preserves_exact_source_channels_clocks_and_labels` asserts unchanged original channels plus left0/right500 clocks and descriptive parameters; P retained-duration/copy test asserts source keys, sourceDuration and provenance; E split/trim/copy composition, source remapping, history/reopen and sampler tests establish preserved evaluation without recompilation. |
| Compile one replacement on its fresh local clock | E `replacing_one_preset_property_keeps_other_retained_typed_and_legacy_clocks` asserts replaced opacity channel has no retained clock, unrelated PositionX complete channel and legacy clock remain identical; P collision replacement asserts new source record is replaced instead of accumulating history and static values remain unchanged. |

The adjusted preset suite passed29/29 in `/tmp/issue47-presets-adjusted.log`. Final full workspace/contracts/native evidence remains root-owned; this spec-only reconciliation does not itself count those gates as passed.

## Final shape legacy split expectation correction

Independent review approves the test-only correction in `crates/editor-core/tests/shape_items.rs::shape_legacy_keyframes_and_mixed_migration_history`. The former assertion expected the split-right legacy key to be resampled/rebased at local0 to the split endpoint. Issue47 deliberately retains the exact original source keys and evaluates them through leftclock0/rightclock500 instead. The corrected test compares every original source key/value/easing record on BOTH pieces, checks each retained legacy clock against the original source duration, restores the exact original item with undo, restores exact split tracks with redo, and checks deterministic full-project reopen from a separate EditorCore owner. The fixture's source keys0/1000 fit its original duration1000, including the permitted legacy terminal key. Existing mixed older-history migration assertions remain intact.

This correction adds coverage for Preserve continuous source animation / split nonlinear legacy sources, Transactional edits and marker lifecycle / undo-redo-reopen, and retained-source schema history compatibility. It does not add a helper sampler, change an API or change runtime behavior; it aligns the old test with already approved retained-source semantics. `git diff --check` on the file passes. The full10-test focused target at `/tmp/issue47-shape-retained-adjusted.log` is now independently inspected and passing:10 passed/0 failed, including `shape_legacy_keyframes_and_mixed_migration_history`. Root must rerun the final workspace and strict Clippy gates invalidated by this test change before completion.
