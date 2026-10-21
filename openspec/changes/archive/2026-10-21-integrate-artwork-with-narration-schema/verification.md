# Integration verification ledger

## Scope and approval

Parent approval is recorded in approval.md with the four original artifact hashes. The implementation follows the approved schema48 boundary, strict main47 narration, component46 package compatibility, exact historical inverses, and selected-derived-artwork rejection. The full existing channel-style-packs requirement delta and proposal capability inventory were added to synchronize its explicit capture-source list with the already-approved Combined conformance requirement's ordinary capture-in-schema48 scenario; portable format46 and rejection constraints are unchanged. Tasks checkboxes track performed work only.

Initial and refreshed refs: feature aeea97a749132cb9acddb455af7262ffb56d546f; main 8cb21a6bab2f5a0134ab6af3f91a018f0ae9bde8. PR229 is open and ready (draft=false), despite older body wording. No WIP was present, and unrelated work, #208, #228, restricted Windows disposal work and PR233 were untouched. Original artwork archive/pins and main narration archive/pins are byte-unchanged. No workflow changes relative to main.

## Scenario mapping

- Migration/current/history strictness: existing artwork schema46 migration test, existing narration schema46/current/history/draft/forged/future tests, and new main47_narration_and_retained_history_survive_artwork48_without_identity_loss. Genuine main47 manifests, native tracks, speech intent and assets survive adoption and artwork commit/history; invalid current/retained47 artwork and missing/null main47 narration preserve complete inventories. Development files are explicitly unsupported, not claimed nonexistent.
- Artwork semantics/atomicity: original raster pixel/alpha/background oracles, provenance roots/cycles/hash guards, source/history/revision/fault tests and genuine plus explicit hermetic actual CLI acceptance remain enforced.
- Package compatibility: existing CP1 test now covers source46/47/48 while preserving component46 and future-component47 rejection. New package46_refuses_selected_derivative_without_mutation_and_ignores_unrelated_derivatives verifies selected-derived refusal, independent ordinary selection, library/project/history/source/derivative preservation. Only selected recipe references are considered for portability; unrelated catalog provenance roots are excluded from that selection, without changing canonical project ownership.
- Contract proofs: artwork-integration-pins.json captures independently fetched main47 and original artwork47 raw hashes before consumer edits. New integration tests recover both complete predecessors and reject missing/duplicate/narrowed schema/operation changes. Historical artwork and narration pin files, captures, old exact inverses and expanded digest assertions are unchanged; explicit integration adapters normalize only approved additions before strict older stages. Raw numeric lexemes are retained.

## Environment

Exact Bun1.4.0, Rust1.97.0 with rustfmt/Clippy and Moon2.3.3 installed into /tmp. Frozen bridge dependencies installed without lockfile changes. CARGO_INCREMENTAL=0 and dev/test debug information disabled to fit this 8.8GiB host; no test assertion, optimization requirement, timeout or ignore changed. Build products are under /tmp/opencut-target, with a temporary uncommitted repository target symlink for existing native-test paths. Existing runtime X11 libraries are exposed through temporary linker-name aliases under /tmp/opencut-native-libs; system/source files are unchanged.

The host PID1 does not reap exited orphan children. Initial unchanged channel-CLI lifecycle test left zombies and failed process-existence assertions. The isolated test launcher /tmp/opencut-test-subreaper.py adopts/reaps exited orphan descendants only; it never signals live children or changes tests. With that environment correction, unchanged channel tests passed12/12 and the full unchanged unit suite passed. Missing native headless-path failures from the initial premature full unit run are retained; final evidence uses the built actual binary at the expected path.

## Terminal passing evidence so far

All full logs remain uncommitted under /tmp.

- Focused native artwork/package tests:12 artwork +3 CP1 passed, zero failed/ignored; /tmp/opencut-integration-package-regression.log.
- Focused native combined narration/artwork scenario passed; /tmp/opencut-integration-combined-native.log. Original focused artwork/narration/profile/speech suites passed; /tmp/opencut-integration-focused-rust.log.
- Strict workspace Clippy exit0; /tmp/opencut-integration-clippy-2.log. Existing third-party future-incompatibility notice retained; no suppression.
- Rust formatting exit0; /tmp/opencut-integration-fmt-final.log.
- TypeScript typecheck exit0; /tmp/opencut-integration-typecheck-6.log.
- TypeScript lint exit0; /tmp/opencut-integration-lint-2.log.
- Full unchanged TypeScript unit suite exit0:1089 passed,15 existing optional skips across95 files; /tmp/opencut-integration-unit-2.log.
- Full source MCP integration exit0:44/44, zero failed/skipped; /tmp/opencut-integration-source-mcp.log.
- Genuine FFmpeg/FFprobe actual artwork CLI/core acceptance exit0; /tmp/opencut-integration-artwork-genuine.log. Separate explicit hermetic backend exit0; /tmp/opencut-integration-artwork-hermetic.log. Both enforce literal pixels, exact hashes, source/history preservation, conflicts, discard and real open-stdin cancellation/oversize assertions; hermetic evidence is not installed-media readiness.
- Hermetic Python:12 unittest and12 pytest passed; /tmp/opencut-integration-python-kokoro.log and /tmp/opencut-integration-python-whisper.log.
- Full repository policy suite exit0:627 passed; /tmp/opencut-integration-policy-2.log. Initial direct run failed one Moon environment fixture before isolated Moon/proto/cache homes were configured; retained /tmp/opencut-integration-policy.log.
- Pinned strict all-spec validation exit0:80/80; /tmp/opencut-integration-strict-final.log.
- Prearchive protected Moon executes policy tests and80 specs successfully and rejects solely this active change; /tmp/opencut-integration-moon-prearchive-2.log. This is expected prearchive rejection, never a protected-gate pass.

## Retained failures and pending obligations

Initial Clippy rejected an OR-pattern style and initial TypeScript checks exposed integration inverse composition/raw-byte problems; all corrected with unchanged strict guards and retained logs. Initial full workspace test compilation failed because desktop linker development names were unavailable; /tmp/opencut-integration-workspace.log. Runtime libraries already existed; temporary linker aliases resolve that prerequisite without source or system edits. The complete rerun is pending.

Initial explicit ignored desktop review was invoked without its required configured FFmpeg/FFprobe environment and failed immediately before test work; /tmp/opencut-integration-desktop-native.log. The correctly configured genuine-tool rerun is pending; the first failure is not counted passed.

Full Rust workspace, configured native desktop review, unchanged contract parity and assembled packaged smoke remain pending as of this ledger entry. No archive, final completion or merge readiness is claimed. Publication may follow all successful local prerequisites, with remote exact-head platform checks explicitly pending. Preserve PR ready state and use a non-force update. Required remote failures remain blockers; do not investigate or change restricted Windows diagnostics/deletion behavior or weaken any gate.


## Resumed checkpoint and disk exhaustion recovery

The assembled unchanged packaged smoke completed exit0,40/40, against the real release headless binary; /tmp/opencut-integration-packaged-smoke.log. Correctly configured genuine FFmpeg/FFprobe desktop review completed exit0,1executed/0failed/0ignored/59filtered in83.79seconds; /tmp/opencut-integration-desktop-native-configured.log. Final stable full lint passed; /tmp/opencut-integration-lint-final-stable.log.

The first checkpoint left full workspace and contract suites live. They subsequently exhausted this8.8GiB host: workspace failed publishing animation_edit_semantics fixture data with explicit ENOSPC after passing preceding suites; contracts failed linking procedural_grids/rich_text_documents with Bus errors at zero free space. Both returned101; full logs /tmp/opencut-integration-workspace-2.log and /tmp/opencut-integration-contracts.log are retained as failures, not passes. No implementation/assertion/timeout/ignore changed to avoid them.

After all three suites exited and no compiler remained active, removed only completed synthetic normalization unit-test directories in the task-created master-normalization-lifetime-/normalization-native-backend- namespaces, then ran exact cargo clean --profile dev -p opencut-editor-core -p opencut-headless -p opencut-desktop with CARGO_TARGET_DIR=/tmp/opencut-target. The supported Cargo cleanup removed1203 generated files/4.4GiB, preserving sources, third-party dependencies and release products; /tmp/opencut-integration-generated-clean.log. Free space restored4.6GiB. The next full unchanged workspace retry is /tmp/opencut-integration-workspace-3.log, session2627. Contract parity retry will wait for terminal workspace evidence to avoid concurrent duplicate build graphs/storage pressure. Prior native/TS/source/packaged evidence remains tied to unchanged source/toolchain/test settings; cleanup changed only disposable outputs.

Parent explicitly authorized local verification -> conform/synchronize/archive -> final protected gate -> nonforce publication -> terminal exact-head remote checks. Remote verification remains an explicit postpublication obligation, never inferred from local success.


## Terminal workspace retry

Full unchanged cargo test --workspace TERMINAL PASS, exit0:76 outer suites,1682passed/0failed/14existingignored; /tmp/opencut-integration-workspace-3.log and /tmp/opencut-integration-workspace-summary.json. The last summary per anchored Running/Doc-tests segment excludes nested child-helper summaries. The14ignore inventory is /tmp/opencut-integration-workspace-ignored.txt; no ignore attribute or assertion was changed. Desktop outer58/0/2 and core outer786/0/10 are included, not added again; the separately configured real desktop review1/0/0 remains separate evidence.

The late live narration guide now distinguishes metadata introduction47 from current48 and capture sources46/47/48 without changing package46. Relevant unchanged documentation/narration/channel regression suites passed40/40; /tmp/opencut-integration-live-guide-tests.log. No executable Rust or TypeScript behavior or test assertion changed after the passing stable-source checks.

After workspace exit0, supported dev-profile cleanup for only the three task workspace crates removed520 generated files/3.2GiB; /tmp/opencut-integration-precontracts-clean.log. Sources, dependencies, release executable and passing logs preserved. Rebuilt the actual debug headless successfully in22.02seconds; /tmp/opencut-integration-headless-precontracts-build.log. Sequential unchanged contract parity retry is /tmp/opencut-integration-contracts-2.log. No concurrent duplicate suite is running. These cache operations do not alter test settings or implementation; earlier failures remain retained.


## Terminal contract parity

Unchanged sequential bun run contracts:check TERMINAL PASS exit0:533native tests across40outer suites, zero failures/ignored, plus706TypeScript tests across46files, zero failures/skips; /tmp/opencut-integration-contracts-2.log and /tmp/opencut-integration-contracts-native-summary.json. This includes actual headless, narration, CP1, historical contract, desktop compositing18/18 and live MCP schema consumers. All required local implementation suites are now terminal passing; earlier failed attempts remain retained above.


## Local openspec-verify-change conformance review

Reviewed the approved proposal, design, both complete delta files, tasks, owning implementation and actual terminal logs. The postapproval source-inventory/delta synchronization is explicitly traced to the originally approved ordinary-capture-in-schema48 requirement, not a new package format. Proposal/design status prose now records the performed approval instead of leaving a contradictory pending-approval statement. All three verification dimensions were reviewed; none was skipped.

| Dimension | Local result |
| --- | --- |
| Completeness |13/13implementation tasks complete;3/3integration requirements implemented |
| Correctness |10/10delta scenarios mapped to automated evidence below |
| Coherence |Schema48 artwork, strict main47 narration, package46, ownership and exact historical captures/inverses agree with the approved design |

| Delta scenario | Implementation and automated evidence |
| --- | --- |
| Migrate retained legacy history | model.rs/migrations.rs; artwork_preparation.rs schema46_current_and_history_migrate_idempotently_and_future_or_premature_are_preserved; narration_blocks.rs schema46_current_history_adopt_empty_manifests_and_invalid_generations_never_publish; main47_narration_and_retained_history_survive_artwork48_without_identity_loss; full native historical-generation coverage |
| Reject future or premature derivation | Same current/retained migration cases, explicit schema49/premature47/null/missing controls, complete unchanged inventories; core model/raw guards and canonical provenance validation |
| Preserve both features through transactions and history | New main47_narration_and_retained_history_survive_artwork48_without_identity_loss checks exact manifests/tracks/assets/source bytes, commit/reopen/undo/redo and stale REVISION_CONFLICT |
| Recover every exact predecessor without masking drift | artwork-integration.test.ts complete main47/artwork47 pins and negative controls; unchanged native speech-alignment marker chain and original artwork/narration raw/compact/expanded digest controls; actual live catalog contract parity |
| Preserve package compatibility without dropping artwork provenance | New package46_refuses_selected_derivative_without_mutation_and_ignores_unrelated_derivatives; CP1 source46/47/48 test; capture.rs refuses selected derivatives before publish and scopes dependencies to selected recipes |
| Capture a caption and scene recipe without changing its source | Existing core capture_is_current_revision_readonly_and_rejects_pending_recovery and native_nested_capture_closure_and_literal_slots_roundtrip_without_partial_mapping_effects; CP1 sources46/47/48 |
| Pending recovery or legacy capture source | Existing core pending-journal/link capture controls and CP1 unsupported45/future49/malformed metadata controls preserve authoritative bytes |
| Reject unsafe or excessive packages | Existing raw_duplicate_keys_are_rejected_in_every_native_map, raw_size_depth_and_required_nullable_boundaries, native package/library bounds and publication/fault suites |
| Literal slot names remain compatible | Existing native_nested_capture_closure_and_literal_slots_roundtrip_without_partial_mapping_effects plus shared bridge closed-record/special-key cases |
| Preserve CP1 across schema47 narration adoption | cp1_component46_survives_project47_and_readonly_capture_of_both_compatible_sources now explicitly covers46/47/48 while old package hashes/component46 and rejection of portable47 remain enforced |

Critical:0. Warning:0. Suggestion:0 within local implementation conformance. The final prearchive protected run passed538policy tests and80specitems, then returned1 solely for this active change; /tmp/opencut-integration-moon-prearchive-final.log. This expected rejection is not protected-gate success. All required local implementation checks are terminal passing, with prior failures preserved above. This authorizes synchronization/archive under the parent's explicit ordering; final protected success and exact-published-head CI remain separate mandatory lifecycle actions. Genuine CODEOWNER acceptance is still pending and not supplied by this task-agent conformance review. No GUI/libmpv/Windows readiness or disposal fix is claimed.


## Completed synchronization and archive

Synchronized only the reviewed artwork migration replacement, combined conformance addition and CP1 source-inventory refinement into project-persistence and channel-style-packs. Preserved main's entire Atomic schema47 narration manifest adoption requirement and every unrelated requirement/scenario; ordinary portable component46 remains unchanged. Source/prepared SHA256 checks guarded synchronization; the only subsequent formatting adjustment removed one extra EOF blank line. Archived only this approved change to openspec/changes/archive/2026-10-21-integrate-artwork-with-narration-schema with .openspec.yaml bytes preserved and13/13tasks complete. Original artwork and narration archives/captures/pins remain unchanged.

Postarchive unchanged protected Moon TERMINAL PASS exit0:538policy tests and79specitems, valid CI parity policy; /tmp/opencut-integration-moon-postarchive.log. Separate pinned strict all-spec validation TERMINAL PASS79/79; /tmp/opencut-integration-strict-postarchive.log. Final protected/strict logs after the EOF normalization are /tmp/opencut-integration-moon-publication-final.log and /tmp/opencut-integration-strict-publication-final.log. This report does not claim remote checks or CODEOWNER acceptance; both are separately observed after publication. The former active-only prearchive rejection and all earlier resource/setup failures remain retained, not relabeled as passes.

All authorized local verification and conformance are complete. Refresh refs and publish this integration as a merge descendant of both verified heads without force, keeping PR229 ready. Exact-head remote CI remains a mandatory postpublication obligation; no full-validation/merge-readiness claim, merge, deployment or issue closure is authorized. Restricted Windows disposal work remains untouched.
