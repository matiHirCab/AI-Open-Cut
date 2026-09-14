# Verification: draft font matching

## Preceding-bindings extension: implementation verified

The approved extension addresses two confirmed failures after a preceding exact action supplies a font: equivalent inherited replacements incorrectly reject, and explicit-resolution versus inheritance alternatives incorrectly succeed. Both replacement orders were reproduced in `%TEMP%/opencut-reversion-review-reproduction-final.log`. Weighted global alternative analysis and chronological preparation now correct both outcomes. All required implementation checks passed with Rust 1.97.0. Atomic rejection remains in force. This font change is now synchronized and archived; the clean-PR post-archive gate passes. Unrelated local work remains excluded from the PR.

Both permanent public-API regressions failed before implementation (`opencut-prefix-red.log`). Focused tests passed (`opencut-prefix-drafts-final.log`: 20 tests; `opencut-prefix-matching.log`: three matching tests). The final longer-chain assertions passed separately (`opencut-prefix-chains.log`) and in the complete workspace run (`opencut-prefix-workspace.log`).

| Prefix scenario | Implementation | Automated evidence |
| --- | --- | --- |
| Equivalent inherited replacements | `DraftFontMatches::for_step` compares complete bindings against the candidate after normal operation application | `prepared_prefix_allows_equivalent_inherited_replacements`: exact and intent-retained prefixes, both orders, changed defaults/source removal, chains, repeated removal/reordering, preview state/reopen/commit |
| Explicit resolution versus inheritance | globally optimal unmatched/candidate outcomes use actual inherited bindings; resolution markers stay distinct | `prepared_prefix_rejects_resolution_versus_inheritance`: both orders, identical current hashes, authoritative/font byte snapshots |
| Freshly resolved prefix and later failure | common store preparation loop stages bytes until all matching and representability checks succeed | `freshly_resolved_prefix_is_not_published_when_later_matching_is_ambiguous`: both orders, a newly resolved bold face, no font publication |
| Global alternatives, exact priority and chronological chains | integer weighted assignment, equality graph reachability, original-order canonical pairs; alternatives retained before canonicalization | exhaustive oracle covers 21,297 weighted graphs of dimensions 1–3, independent lexicographic optima and three chronological models; prior 512-graph/six-outcome oracle and 100-operation test remain |

No dependency, public field, persisted version or rendering change was introduced. The matching matrix is bounded by twice the existing operation limit. All five normative requirements and fourteen scenarios are covered by the combined traceability tables and current check evidence below.

## Preserved selector-reversion coverage

The earlier approved selector-reversion correction addresses base A / previous draft B / current default C silently selecting A on reversion. Its regressions remain in the final passing suite. Atomic rejection remains the approved policy for results draft 2 cannot represent.

| New scenario | Implementation | Automated evidence |
| --- | --- | --- |
| Family/path reversion and null resets, including styled-only differences | `retention` marks changed selectors; `apply_retained_fonts` captures/clears inherited bindings; `validate_resolved_component_fonts` compares complete bindings before publication | `component_selector_reversion_rejects_different_current_bindings_atomically`: eight cases, each rejected twice with byte-for-byte draft/project/history/font comparisons |
| Equivalent reversion, unchanged sibling, repeated update, preview/reopen/commit | equal complete bindings accepted; `draft_binding_step` uses the snapshot before artificial clearing | `component_selector_reversion_accepts_equal_complete_bindings_through_commit`: four selector cases; empty persisted step assertion |
| Reintroduced local IDs | missing prior local identity produces explicit resolution marker | `component_reintroduced_local_ids_require_current_resolution`: differing regular, differing bold, and equal bindings |
| Resolve versus inherit/retain ambiguity | marker set participates in full outcome equality; unmatched baseline resolves only normally unresolved local IDs | `component_matching_distinguishes_resolution_from_inheritance` in both old-operation orders; exhaustive 512-graph oracle expanded to six outcome models, including explicit resolution versus bindings and destination-specific unresolved baselines |

The new reversion, reintroduced-ID and ambiguity tests failed against the prior implementation (`opencut-reversion-red.log`, `opencut-reversion-reintroduced-red.log`, `opencut-reversion-ambiguity-red.log`). All 17 draft tests then passed (`opencut-reversion-drafts.log`). Final workspace checks also cover the subsequent empty-step assertion. Matching retains its polynomial algorithm and existing 100-operation bound; no rendering or public/persisted format changes were made.

## Approval and compatibility

The user's explicit implementation request approves the plan transcribed in this change. No public fields, version changes, migrations or Unicode rendering changes were introduced. Editor-core assets owns font retention/matching; store owns replay and atomic publication. Existing operation validation bounds both old and replacement operation lists before matching.

## Requirement-to-test traceability

| Requirement/scenario | Implementation | Automated evidence |
| --- | --- | --- |
| Complete ambiguity in both directions | assets/fonts/matching.rs weighted optimal alternatives; chronological comparison of complete outcomes | `one_old_font_action_cannot_choose_between_two_replacements`; `ambiguous_draft_matching_is_atomic_and_equivalent_matches_succeed`; prepared-prefix regressions |
| Exact priority and equivalent original-order assignments | lexicographic structural/total match objective; canonical pairing within the optimal equality graph | Existing reorder/equivalent regression; exhaustive `alternating_paths_agree_with_exhaustive_retention_outcomes` checks all 512 three-by-three graphs with six outcome models, including bold-only differences, explicit resolution and inherited outcomes; weighted oracle adds exact-edge priority |
| Bounded matching | store validates old and new operation counts; polynomial matching | `equivalent_actions_pair_in_original_order_at_edit_limit` exercises 100 operations |
| Partial component selector edit, preview/reopen/commit | replay captures actual local bindings; preparation applies retained maps to the targeted component | `component_selector_edit_preserves_untouched_local_fonts` covers create/update, source removal and full binding equality |
| Local identity, movement, style and scope | local-ID/selector comparison independent of non-font properties | `component_local_retention_survives_structure_and_non_font_edits`; `component_update_retention_is_scoped_across_identical_local_ids`; existing `component_draft_bindings_use_scoped_local_identity` |
| Distinguish inherited children from selector-step contributors | shared store replay captures old actual bindings; chronological preparation uses actual scoped prefix bindings to normalize equivalent inheritance | `component_matching_compares_actual_local_bindings_not_shared_selector_steps`; `component_actions_without_new_bindings_have_equivalent_empty_retention`; `component_replacement_retains_fonts_inherited_from_removed_draft_actions`; prepared-prefix regressions |
| Conflicting/equivalent selector outcomes | checked selector-map construction before publication | `component_shared_selector_must_be_representable_in_draft_v2` verifies byte snapshots, owned font bytes, version 2 and commit |
| Inherited binding conflict | scoped application rejects an unrepresentable override before publication | `inherited_component_binding_conflicts_fail_before_font_publication` stages a changed font and compares all authoritative and owned font bytes |
| Existing font lifecycle, integrity, revision and rendering parity | unchanged core validators and common materialization/renderer | Workspace font_resolution, shaping, store draft, component and headless suites; contracts/integration/smoke gates |

## Regression history

Before implementation the three new primary tests failed while all four existing tests passed (`%TEMP%/opencut-matching-red.log`). An independent ordering assertion then exposed an incidental augmenting-path ordering mismatch (`opencut-matching-order-red.log`), and a component replay fixture exposed false ambiguity from interpreting a selector step as every child's binding (`opencut-matching-local-red.log`). Removal of an earlier action also demonstrated the need to retain the later action's actual inherited binding (`opencut-matching-inherited-red.log`). All were corrected and retained as permanent regression coverage. Temporary review reproductions are represented by permanent repository tests.

## Check evidence

Full output is retained outside the repository under `%TEMP%/opencut-prefix-*.log`. Rust commands selected `+1.97.0`; Bun commands invoking Cargo used `RUSTUP_TOOLCHAIN=1.97.0`. No default toolchain was changed.

| Command | Result | Log |
| --- | --- | --- |
| `cargo +1.97.0 fmt --all -- --check` | Pass | `opencut-prefix-fmt.log` |
| `cargo +1.97.0 clippy --workspace --all-targets -- -D warnings` | Pass; upstream proc-macro-error2 future-compatibility notice only | `opencut-prefix-clippy.log` |
| Focused draft tests / matching tests / final chain tests | 20 / 3 / 2 passed | `opencut-prefix-drafts-final.log`, `opencut-prefix-matching.log`, `opencut-prefix-chains.log` |
| `cargo +1.97.0 test --workspace` | Pass: 291 core unit tests, all integration suites including 20 draft and 15 font lifecycle tests, 27 headless protocol tests and doc tests | `opencut-prefix-workspace.log` |
| `bun run typecheck` / `bun run lint` | Pass | `opencut-prefix-typecheck.log`, `opencut-prefix-lint.log` |
| `bun run test:unit` | 392 passed in 21 files on rerun; initial headless timeout retained below | `opencut-prefix-unit-retry.log`, `opencut-prefix-unit.log` |
| Hermetic Python test runner | 10 unittest and 5 pytest passed | `opencut-prefix-python.log` |
| `bun run contracts:check` | Governed Rust suites and 328 TypeScript parity tests passed | `opencut-prefix-contracts.log` |
| `bun run test:integration` | 11 passed | `opencut-prefix-integration.log` |
| `bun run test:smoke` | 6 passed | `opencut-prefix-smoke.log` |
| Pinned OpenSpec `validate --all --strict --no-interactive` | 27 items passed, also checked by Moon | `opencut-prefix-spec.log`, `opencut-prefix-moon.log` |
| `git diff --check` | Pass; existing CRLF notices only | `opencut-prefix-diff-check.log` |

The first TypeScript unit run failed `accepts protocol events split across stdout chunks` with `HEADLESS_TIMEOUT` during concurrent startup. The unchanged full suite passed on rerun; no timeout or test was weakened. The native workspace suite used FFmpeg/FFprobe 7.1.1, the repository DejaVu fixture, `OPENCUT_GOLDEN_REQUIRED=1`, and `RUST_TEST_THREADS=2`. Seven existing core tests are intentionally ignored subprocess, explicit recapture, or external-report helpers; applicable parent tests passed. No new ignore, warning suppression, dependency change or golden recapture was introduced.

## Conformance assessment

| Dimension | Assessment |
| --- | --- |
| Completeness | 15/15 tasks performed; all five requirements and fourteen scenarios covered. Living specifications are synchronized, the change is archived, and the clean-PR post-archive gate passes. |
| Correctness | Both reproduced prefix failures corrected; exhaustive weighted alternatives and chronological outcomes agree with independent enumeration. All prior lifecycle, selector-reversion, collision, revision and integrity coverage passes. No remaining implementation/specification mismatch identified. |
| Coherence | Matching remains internal to editor-core assets. Store uses its shared preparation loop and actual prefix state. Canonicalization never narrows the global alternatives checked later. Bounds and public/persisted versions remain unchanged. |

The proof obligation for chronological comparison is explicit: after every previously accepted step, all globally optimal assignments have the same normalized font outcome and therefore the same prepared font state. Every allowed current edge occurs in a global optimum. Comparing all of those edges against that common state preserves all alternatives without enumerating assignments. The weighted exhaustive test checks this invariant against full chronological traces, separately from the public-API regressions.

## Historical shared-checkout gate results

The final repeat against the completed artifacts had the same result (`opencut-prefix-moon-final.log`); OpenSpec reports 14/15 tasks complete (`opencut-prefix-apply-final.json`).

After all implementation checks passed, Moon validated all 27 OpenSpec items but exited 1 because this change and unrelated `reduce-agent-context-overhead` remain active (`opencut-prefix-moon.log`). The repository's `docs/spec-driven-development.md` Verification order states: "Any other failure blocks archival; never modify or archive unrelated work to clear it." Accordingly, specifications were not synchronized, the change was not archived, and a post-archive Moon run was not possible. Task 3.4 remains open. The verification report does not claim overall completion or merge readiness. Unrelated work was left untouched.
## Finalization on the PR tree — 2026-09-14

At the user's request, finalization used a clean detached worktree of PR commit `f8a052c39c5b7ff2eb95195d419a237bf142a68e`. The unrelated, uncommitted `reduce-agent-context-overhead` change and its files were neither moved nor modified. No implementation, contract, fixture, toolchain or protected policy input changed; prior passing implementation evidence remains applicable. Only living specifications and this change's archival/task/evidence artifacts changed.

The clean pre-archive Moon run passed all 26 validations and rejected only this active change (`%TEMP%/opencut-archive-preflight.log`), satisfying the pre-archive condition. All artifacts were complete and all implementation tasks verified. Five added requirements were synchronized to `openspec/specs/font-resolution/spec.md`, preserving the existing scenarios, and the change was archived here with `.openspec.yaml` intact.

The required post-archive `moon run root:openspec-validate` passed all 25 specification validations and the unchanged CI parity policy (`%TEMP%/opencut-archive-postflight.log`). Task 3.4 is complete. The shared original checkout may still reject the unrelated active change; that result is separate from the verified PR tree. No merge or claim about GitHub-hosted CI is implied by these local checks.

Final repeat checks also passed: strict all-spec validation in %TEMP%/opencut-archive-strict-final.log and protected Moon in %TEMP%/opencut-archive-moon-final.log (25 specifications, no active changes in the PR tree).
