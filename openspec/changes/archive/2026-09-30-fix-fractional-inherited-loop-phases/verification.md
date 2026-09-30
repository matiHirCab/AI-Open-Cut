# Fractional inherited loop verification

Status: complete. All required implementation, conformance and final specification gates passed. This report does not treat checked tasks or skipped native tests as evidence.

## Requirement and scenario evidence

| Requirement/scenarios | Inspected implementation and automated evidence |
| --- | --- |
| Deterministic item-local loop evaluation / exact forward seams and exhaustion; ping-pong turn and return | `render_plan::looped_time_expression` shares constant-work phase and endpoint rules with explicit precision. Existing `animation::curve_tests::loop_phases_are_item_local_and_exact_at_seams` covers integer repeat, ping-pong, spring and Bezier reflection; `native_loop_scalar_expression_matches_core_at_seams` independently exercises the backend against integer core samples. Required native loop run passed both tests. |
| Preserve fractional repeat phase; fractional turns and finite completion | `native_loop_fractional_expression_matches_independent_phases` uses handwritten expected values at 99.25, 99.5, 99.75, exact repeat seams, ping-pong turns, finite completion and nonzero first-key offsets. It evaluates real FFmpeg expressions and does not call production phase/sampling helpers to calculate expectations. Before correction it failed at 99.25 with actual 100 versus expected 75 (exit 101). After correction all cases pass (exit 0). |
| Preserve independent properties; existing integer and independent audio sampling | Integer/audio calls retain `IntegerMilliseconds`; derived visual calls explicitly select `FractionalMilliseconds`. Curve compilation, property clamping and persisted timestamp arithmetic are shared and unchanged. Existing core loop, semantic-plan, audio-clock and native independent PCM regressions are retained; the complete workspace and required native runs passed. |
| Independent fractional visual loop conformance / reproduce fractional parent seam across intents | `native_inherited_timing_fractional_loop_seams` uses a 0.995-rate component and parent position keys (0,0), (99,100), (100,0). Root 100 ms independently requires parent x=50, child left=60. Before correction the four-intent fixture fails with missing expected pixels (exit 101). The corrected focused run passes frame, range, materialized draft and export (exit 0). Each intent is checked against independent pixels and opacity, in addition to unchanged SSIM >= .99. |
| Compose nested fractional phases | The same native test maps `(1.99*t + trim 1 - hidden-rank delay 1)*.5 = .995*t`; descendant opacity uses group child delay 1, and copied parent clocks subtract controller rank delay 1 plus signed offsets +1/-1. Hidden siblings consume ranks. Independent triangle phase, positions and opacity are checked at root 100/200 ms for each intent; focused required run passes. |
| Deterministic additional-copy semantics / zero-offset ranked controller, copy offsets and exact intervals | Existing `staggered_controller_restores_short_source_with_fractional_signed_clocks` covers ordinary [0,100), rank-one controller stagger 200, zero offset copied [200,300), fractional rates, hidden ranks and signed offsets. `nested_repeater_controller_shifts_source_stages_once_and_preserves_outside_clock` covers nested source stage/controller separation. Ordinary timing and generated identities/order remain unchanged. |
| Other preserved repeater scenarios: supported sources, component descendants, rich-text bindings, ordering/identity | Existing repeater scene tests `repeater_adds_stable_translated_and_faded_shape_occurrences`, `group_repeater_includes_nested_component_and_local_repeater_occurrences`, `component_repeater_preserves_numeric_order_for_eleven_equal_z_siblings`, `rich_text_bindings_reach_local_and_outer_repeater_copies_without_scope_leakage` and `component_local_repeater_occurrences_keep_unique_scoped_identities` are retained. No repeater production code changed in this follow-up. |
| Preserve compatibility and accurate verification | Schema remains 26; no public fields, aliases, error contracts, migration or publication logic changed. Workspace and contract suites retain standalone/batch/draft rollback, history, reopening, current/retained migration faults and zero-generated-materialization bounds checks. Guides and ADR 0004 distinguish persisted integer timestamps from fractional derived clocks; living repeater scenario is synchronized only after verification. Both previous archives are protected by a 21-file SHA-256 inventory outside tracked files. |

## External logs

Full logs are in `C:/Users/matia/AppData/Local/Temp/` with prefix `issue42-fractional-`.

- `scalar-red.log`, `native-red.log`: genuine failures on the unchanged renderer, after correcting fixture schema/ordering construction. These are expected red evidence, not successful checks.
- `loop-green.log`: required native fractional and existing integer loop tests passed, 2 tests.
- `native-green.log`: required focused native four-intent fractional test passed, including both signed nested cases.
- `required-unavailable.log` and `required-font-unavailable.log`: expected exit 101 for configured missing FFmpeg and missing font respectively with required mode enabled; absent dependencies cannot silently pass.
- `fmt.log`, `clippy.log`: formatting and strict workspace Clippy passed.
- `typecheck.log`, `lint.log`: passed.
- `test-unit.log`: first run failed one TTS cancellation timeout, with 420 passing tests and one optional real-provider skip. `test-unit-rerun.log`: full rerun with known Python passed 421 tests and the same optional skip. The failure is preserved; no test timeout or implementation was changed to obtain the pass.
- `python.log`: all 10 hermetic worker tests passed.
- `strict-pre.log`: all 33 living/active items passed strict pinned OpenSpec validation.
- `moon-pre.log` and `moon-verified-pre.log`: expected exit 1 solely for this active change; the latter reran after all required implementation checks passed. `strict-verified.log` passes all 33 items. No other policy failure occurred.
- `contracts-check.log`: all Rust/TypeScript contract checks passed, including 354 TypeScript tests.
- `test-integration.log`: initial run failed because the default headless path did not follow the shared CARGO_TARGET_DIR. `test-integration-rerun.log`: all 13 MCP integration tests passed with OPENCUT_TEST_HEADLESS_PATH pointing to the actual build. No transport code was changed.
- `test-smoke.log`: initial packaged run timed out after 30 seconds in the combined group/component workflow; 7 of 8 tests passed. `test-smoke-rerun.log`: the full rerun passed all 8 packaged tests with unchanged timeout and assertions.
- `workspace.log`: complete sequential workspace suite passed (exit 0), including publication bounds, alias/draft rollback, retained history/migration, architecture and headless wire regressions.
- `native-all.log`: all 4 required native inherited fixtures passed (exit 0; no skipped/ignored tests), including all ten independent nested samples 0 through 900 ms and the independent PCM reference. Runtime 2163.94 seconds with FFmpeg/FFprobe 7.1.1 and pinned DejaVu Sans SHA-256 `7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954`; do not infer native success from workspace tests that may return early without configured tools.

## Evidence limits

New fractional native fixtures cover repeat geometry/opacity and representative nested delays; the scalar backend fixture covers fractional ping-pong/finite boundaries. Existing required native fixtures retain Bezier/spring channels, both loop modes, shifted final-grid media sampling, exact half-open absence, and independent unstaggered audio reference. No tolerance, canonical reference or public behavior outside the approved visual phase correction is changed. Windows evidence does not establish a Linux CI run. Optional real-provider inference is outside inherited timing conformance. All required implementation checks and protected final gates passed.

## OpenSpec verification assessment

| Dimension | Assessment |
| --- | --- |
| Completeness | 11/11 tasks complete, including specification synchronization, archival and passing final gates. |
| Correctness | All 3 changed requirements and 16 scenarios map to inspected code and passing automated evidence above. Genuine pre-fix failures establish that the new tests detect the corrected defect. |
| Coherence | Approved design followed: explicit private precision selection, shared curve and endpoint compiler, preserved integer/audio path, no schema/public/error/dependency changes, unchanged tolerances and immutable prior archives. |

No critical implementation issues, warnings or suggestions remain. No required implementation check was skipped. Optional provider/platform limitations are stated above. First-run failures are preserved and superseded only by actual full passing reruns, with no timeout/assertion weakening. Protected gate and final synchronization outcomes are recorded separately after closeout.

## Final closeout

The two complete modified requirement blocks and added fractional-conformance requirement were synchronized into animation-loops, repeaters and inherited-animation-timing living specifications, preserving existing scenarios and unrelated requirements. Only this follow-up was archived. The pinned CLI selected UTC date `2026-09-30`, resulting in `openspec/changes/archive/2026-09-30-fix-fractional-inherited-loop-phases/`. Archival warned about the sole unchecked lifecycle task because final post-archive gates could not yet have run; those gates have now passed and that task is complete.

`moon run root:openspec-validate` and pinned `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` both passed after synchronization and archival (exit 0; 32 living specifications). Full logs are `issue42-fractional-final-moon.log` and `issue42-fractional-final-strict.log` in the external log directory above. They are rerun after this final evidence/task update so the reported completed artifacts are also checked.

All 21 saved file hashes in both prior issue-42 archives remain unchanged. `git diff --check` passed. Branch remains `codex/issue-42-inherited-animation`; HEAD remains `f78911fe4083c6c202bba463795fdb046bad7aff`. No commits or pushes were made. No required check remains skipped or failing; prior timeout/setup failures and expected negative-dependency failures are explicitly recorded above.
