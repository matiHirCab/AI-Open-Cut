# Implementation evidence — 2026-09-30

## Additional lifecycle review correction — 2026-09-30

Fresh independent probes subsequently reproduced stale-draft base eviction blocking project access, incorrect inherited timing for sampled transitions/visibility, and raw oversized sampled-encoder diagnostics in core/headless. Earlier passing tasks and suites did not cover those scenarios. The user approved the additional [lifecycle correction](../2026-09-30-fix-extended-animation-lifecycle-regressions/verification.md); that record distinguishes reproduced failures, corrections, fresh verification and platform limits. This note preserves historical evidence and narrows earlier conformance claims rather than treating them as proof of these newly tested cases.

## Superseding review correction — 2026-09-30

Independent review reproduced five defects despite the passing checks recorded below: effects-only scale overshoot escaped candidate certification; identity extensions shifted offset legacy paths; schema-26 migration accepted malformed legacy-only retained drafts; correlated crop bounds omitted the interpolation floor; and extended consumers lost adjacent large integer timestamps. The original conformance claims below are historical evidence and do not establish those guarantees. Approvals and original logs are preserved.

The user approved `fix-extended-animation-review-regressions` on 2026-09-30. Its [correction verification](../2026-09-30-fix-extended-animation-review-regressions/verification.md) records owned regressions, corrections, reruns and platform limits. The correction passed implementation and conformance verification and its accepted deltas are synchronized and archived. Final protected gates are recorded in that report; no remote CI success is claimed.

The user approved the proposal and bounded-certification amendment on 2026-09-30. Implementation and required local checks pass. OpenSpec conformance review is recorded below. The user approved concrete contract review as designated CODEOWNER on 2026-09-30. Verification is complete, deltas are synchronized, and the change is archived at openspec/changes/archive/2026-09-30-render-extended-animation-channels. The final protected Moon and strict all-spec gates both pass (exit 0; 33 specifications passed, zero failed). Implementation and the mandatory local OpenSpec lifecycle are complete. This report does not assert remote Actions success.

## Requirement and scenario traceability

Paths are repository-relative. `tests/extended_visual_animation.rs` is also included by the existing required `tests/animation_channels.rs` native target. Canonical consumers do not regenerate catalogs.

| Requirement and scenarios | Implementation | Automated coverage |
| --- | --- | --- |
| Closed typed channels: supported/deferred values, transform collision, audio gain and parents | model/animation_channels.rs, validation/animation_channels.rs, animation.rs | Existing animation_channels tests, canonical channel catalog, new channel fixture cases and native rotation oracle |
| Bounded certification: intermediate crop, correlated-safe channels, bounded deterministic work, legacy preservation | evaluated_scene/extended_certification.rs, extended_visual.rs, store.rs | Spring crop rejection/correlated linear acceptance; inclusive node quota/left-first unit; exhausted batch/draft unchanged-byte tests; legacy draft edits on extended bases; existing legacy suites |
| Bounded targets: every property/static fallback, invalid scope/kind/identity/coupled geometry | model/visual_effects.rs, validation/extended_visual.rs, evaluated_scene/extended_visual.rs | Canonical extended fixture cases, topology/target/bound integration test, scoped effect identities, empty-channel fallback, strict Rust/TypeScript fixtures |
| Compound sampling: curves, loops, endpoints, linear-premultiplied color, fixed topology, invalid intermediates | animation.rs, extended_visual.rs, extended_certification.rs | Independent canonical hold/linear/Bezier/spring/ping-pong samples, fractional clocks unit tests, dense spring bound oracle, gradient spring collision rejection, native all-property corpus |
| Transactional edits: aliases, rollback/conflicts, clear | timeline.rs, store.rs | Scoped root/component alias batch undo/redo/reopen; aliased draft revision and byte rollback; stack clear/static preservation; actual MCP extended workflow |
| Bounded effects: transactional edits and invalid stacks/references | model/visual_effects.rs, validation/extended_visual.rs | Canonical accepted/rejected crop/effect cases, nonfinite and UTF-8 unit cases, stack eligibility/count/identity and invalid-radius byte-preservation tests |
| Effect composition: order/anchors and bounded work | render_artifact/extended_visual.rs, extended_certification.rs | Independent Gaussian normalization/glow alpha; tint/vignette/zero identity/order units; cumulative exact budget/overflow; hidden excessive-work rejection; noncentral native anchor parity |
| Shared rendering: each intent/property, transformed geometry/audio, fail before output, legacy regressions | evaluated_scene.rs, render_artifact/extended_visual.rs, render_plan.rs, render_process.rs, renderer.rs | Independent isolated visible-output changes for each of the 12 properties; all 12 properties in root/nested frame/range/draft/export at 0/200/500/900 ms, ping-pong/fractional clocks; exact draft PNG and SSIM >= 0.99; independent crop/rotation pixel assertions; PCM RMS <= 0.0001/timing tolerance; output-preservation failure; legacy goldens/cache suites |
| Schema-27 migration: history/drafts, invalid retained state, deterministic recovery/reopen | migrations.rs, store.rs, model.rs | Current/nonempty undo/redo/draft migration; premature current/history/draft and future version unchanged-byte tests; source-26 publication fault injection; no-write valid reopen |
| Governed activation: synchronized consumers and supported/deferred discovery | Canonical catalogs/ownership, headless main, bridge schemas/declarations, docs | Rust/TS structural parity and deliberate expanded MCP digest, capability/schema reporting, actual MCP batch workflow. Human CODEOWNER review approved on 2026-09-30. |

Each scenario in the five delta specifications is covered by its corresponding row. New behavior stays in editor-core; shared text measurement serves both certification and rendering. Native decode/streaming stays behind renderer ports. Transports invoke existing typed operations without parallel domain rules. There are no baseline changes, new operations, provider protocol changes, or new dependency edges.

## Verification evidence

Full uncommitted logs are under `C:/Users/matia/AppData/Local/Temp/`. Stable-tree reruns supersede intermediate logs.

| Check | Result | Log filename |
| --- | --- | --- |
| cargo fmt --check --all | Exit 0, final implementation | opencut43-fmt-final.log |
| cargo clippy --workspace --all-targets -- -D warnings | Exit 0, final implementation | opencut43-clippy-final.log |
| cargo test --workspace | Exit 0, final stable tree including legacy large-integer compatibility | opencut43-workspace-stable.log |
| Independent canonical compound values | Exit 0, final linear-light/Bezier/spring/reflected-loop fixtures | opencut43-compound-fixtures-final.log |
| Required native animation target | Exit 0, 34 passed, including isolated visibility of all 12 properties and all-intent parity | opencut43-animation-native-final.log, opencut43-isolated-visible.log |
| Native all-property root/nested intent parity | Exit 0, exact draft PNG and required SSIM | opencut43-compound-all-intents-final.log |
| Root/component alias target lifecycle | Exit 0 | opencut43-scoped-alias.log |
| Legacy integer sampling above f64 exact range | Exit 0; exact adjacent keyframes, linear progress and hold selection | opencut43-large-integer-sampling.log |
| Rules-screen PR 1920x1080 | Exit 0, five semantic states / six render operations | opencut43-rules-1920x1080-final.log |
| Rules-screen PR 1280x720 | Exit 0, five states / 25 operations | opencut43-rules-1280x720-final.log |
| Rules-screen PR 960x540 | Exit 0, five states / 25 operations; reviewed references unchanged | opencut43-rules-960x540-final.log |
| Main golden conformance / report validator | Exit 0, reviewed references unchanged; report validated | opencut43-golden-report-final.log, opencut43-golden-report-validation-final.log |
| Legacy native core library suites | 377 passed, 9 ignored helpers; outer historical workspace later failed an obsolete target fixture, subsequently fixed | opencut43-workspace-final.log |
| Native headless lifecycle / transform2d / font resolution | Exit 0; 1 / 13 / 16 tests passed | opencut43-headless-native-final.log, opencut43-transform-native-final.log, opencut43-font-native-final.log |
| Native raster cache | Exit 0, 13 passed | opencut43-raster-cache-final.log |
| Instrumented worker cache | Exit 0, 3 passed | opencut43-worker-cache-final.log |
| Instrumented bridge cache / default binary restoration | Exit 0 | opencut43-bridge-cache-final.log, opencut43-default-restore.log |
| TS formatting/typecheck/lint/unit | Exit 0 for all four checks, 426 unit tests passed, one opt-in native test skipped (covered separately) | opencut43-ts-format-final.log, opencut43-typecheck-final.log, opencut43-lint-final.log, opencut43-unit-final.log |
| Contract parity | Exit 0, final Rust parity and 359 TypeScript tests | opencut43-contracts-final.log |
| MCP integration / packaged smoke | Exit 0, final 13 integration / 8 smoke tests passed | opencut43-integration-final.log, opencut43-smoke-final.log |
| Hermetic Python worker | Exit 0, 10 unittest + 5 pytest; provider inputs unchanged | opencut43-python-final.log |
| Strict all-spec validation | Exit 0, final 33 items passed | opencut43-spec-final.log |
| Protected postarchive Moon gate | Exit 0; policy valid, 33 specifications passed, zero failed | opencut43-protected-postarchive-final.log |
| Strict postarchive all-spec validation | Exit 0; 33 specifications passed, zero failed | opencut43-postarchive-spec-final.log |
| Protected prearchive Moon gate | Exit 1 naming only this active change; 33 specs and policy tests pass. Expected blockage, not final gate success | opencut43-protected-prearchive-final.log |

Native tests use reviewed DejaVu Sans and compatible FFmpeg/FFprobe 7.1.1. Ordinary hermetic suites skip opt-in native cases; dedicated required runs provide separate evidence. Windows success does not establish Linux Actions execution or POSIX containment. All three required PR shards pass; scheduled full weekly CI is separate from PR scope.

Intermediate failures (obsolete schema assertions, missing target kinds, catalog mismatches, fixture setup, incorrect native paths, Windows executable locks, and isolated visibility fixtures whose opaque tint/subpixel radii masked their changes) were corrected and rerun. They are not passing evidence. Native process suites and headless rebuilds are serialized. No required failure is waived. The final legacy precision guard only applies when an integer channel has keyframes above 9007199254740991; reviewed native render fixtures stay below that threshold, leaving their sampled path and relevant inputs unchanged. Its exact-keyframe/hold/progress regression is tested separately.

## Completed review and lifecycle

Contract review and implementation verification are approved/completed, and all five deltas are synchronized. The verified change is archived, protected Moon and strict all-spec validation both pass, and all 26 tasks are complete.

## Conformance review — openspec-verify-change

Reviewed all resolved proposal, five delta specifications, approved design/amendment, tasks, owning code and tests using the pinned OpenSpec status/apply context. No artifact or conformance dimension was skipped.

| Dimension | Assessment |
| --- | --- |
| Completeness | 26/26 tasks complete; implementation, checks, contract review, verification, synchronization, archival and both final gates are complete. |
| Correctness | All 10 normative requirements and 29 scenarios have implementation and automated coverage, mapped above. Canonical consumers and local native render gates pass. Human approval required by Governed activation is recorded in proposal.md. |
| Coherence | Approved scoped targets, core ownership, atomic schema-27 adoption, continuous sampling, bounded left-first certification, streaming preparation, legacy preservation and deferred subset are followed. Registry/CODEOWNERS synchronization was corrected during review. No new dependency edge or baseline update. |

RESOLVED: task 1.2 required the designated CODEOWNER to review the implemented public/persisted changes. `contracts/contract-ownership-v1.json:4` designates @matiHirCab; `AGENTS.md:50` and `docs/spec-driven-development.md:39` require this review. The user approved those concrete changes as designated CODEOWNER on 2026-09-30 (response: Approve implemented contracts), covering schema-27 crop/effect/target declarations, active catalog/capability, migration and synchronized headless/MCP catalogs/digest. Earlier approvals separately authorized the planning artifacts.

No unresolved implementation/spec/design mismatch or uncovered scenario was found. No critical issue remains. Required checks, concrete contract review, conformance verification and intelligent specification synchronization are complete. Archival and both final gates completed in the mandated order. No implementation or specification lifecycle task remains.
