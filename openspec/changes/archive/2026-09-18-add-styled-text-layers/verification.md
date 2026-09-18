# Verification: add-styled-text-layers

## Final archival result — 2026-09-18

All tasks are complete following the owner's explicit conversational visual/CODEOWNER approval and authorized coordinated archival of both changes. Accepted delta requirements are synchronized into the five affected living specs. Strict all-spec validation passed (26 living specs, exit 0), and the unchanged protected Moon gate passed (exit 0). Logs: `%TEMP%/opencut-archive-specs.log` and `%TEMP%/opencut-archive-moon.log`. There are no active change directories. No validation policy was weakened. Earlier incomplete-task/blocker statements below are historical and superseded by this result. GitHub branch-protection review requirements and remote CI remain separate from this recorded conversational approval/local completion; no GitHub approval event is fabricated.

## Review approval and coordinated archival — 2026-09-18

The repository owner replied “Approve” in this task to the explicit request for independent visual and designated CODEOWNER review sign-offs for PR #120 after reviewing the completed correction and full verification results. This records conversational owner approval, not a fabricated GitHub review event. Tasks 3.3 and 4.2 are satisfied by that approval and the existing automated evidence. Conformance verification finds no remaining code/spec/test mismatch.

The user selected both active changes for archival and approved proceeding after resolving their verification blockers. The last pre-archive protected check rejected only these two selected changes; no unrelated change is being archived. They are processed as one coordinated batch, followed by mandatory unchanged protected-gate and strict all-spec validation. Existing historical blocker reports below are superseded by this approval and the final archival result.

## Native compatibility resolution — 2026-09-18

This assessment supersedes the native mismatch below. Task 7.1 is complete: conditional `EvaluatedTextStyle` debug formatting omits absent paints, preserves all legacy fields/order, and retains explicit empty/nonempty paints. No reviewed semantic, image or audio reference was regenerated. A new native-independent rule-card test failed before the correction and passes afterward against both original/moved references; a separate test distinguishes absent, empty and colored stacks.

Verification uses Rust 1.97.0; full native rendering uses the checked-in DejaVuSans fixture, isolated FFmpeg/ffprobe 7.1.1 and `OPENCUT_GOLDEN_REQUIRED=1`. All logs are under `%TEMP%/opencut-finalize-`:

| Check | Result | Log suffix |
| --- | --- | --- |
| New legacy semantic regression before correction | 101, expected failure | `red.log` |
| Focused semantic tests after correction | 0, 5 tests | `semantic.log` |
| `cargo fmt --check --all` | 0 | `fmt.log` |
| Strict workspace Clippy, all targets | 0 | `clippy.log` |
| Full serial `cargo test --workspace` | 0 | `workspace.log` |
| Exact `renderer::golden::native_golden_render_conformance` | 0, full golden suite, 535.52 seconds | `native.log` |
| Agent-context focused tests | 0, 17 tests | `context-tests.log` |
| Strict all-spec validation | 0, 27 items | `specs.log` |
| Protected Moon validation | 1, only the two active changes rejected | `moon.log` |

The initial new test had an unnecessary mutable binding; this was removed before final strict Clippy/workspace verification. The full native run exercises unchanged native implementation after that test-only cleanup. Earlier bridge/contract/Python evidence remains applicable: this correction affects internal semantic debug output and test coverage, with no wire, provider, rendering-pixel, schema or dependency changes.

OpenSpec verification reassessment: 14/19 tasks complete; the identified code/reference mismatch is resolved, and required local implementation checks pass. Independent visual and designated CODEOWNER reviews remain unrecorded (`reviews: []`, `REVIEW_REQUIRED` on PR #120). These are separate evidence requirements; neither the user's request to finish nor passing native checks is represented as reviewer approval. Both active changes still fail the protected archive-only gate, so no synchronization or archival was performed. Remaining tasks are 3.3, 4.2, 5.3, 5.4 and 5.5.

## Unchecked-task reassessment — 2026-09-18

This section supersedes earlier statements that only review/archive gates remain or that all required native checks pass. The previously recorded focused raster and native rich-text results remain valid, but they did not exercise the entire native golden suite.

At PR head `e86212ca`, CI run `35342116136` reports a full native rendering failure in job `105590256852`: `renderer::golden::native_golden_render_conformance` fails at `renderer/golden/rule_card.rs:334`, comparing generated semantic plan bytes with stored references. Decoded assertion data differs by nine added `paint_layers: None` lines. This mismatch requires correction and a successful full native rerun before conformance completion; it is not evidence of a pixel difference. Full log: `%TEMP%/opencut-unchecked-ci-render.log`.

Fresh strict all-spec validation passes all 27 items, while the protected Moon task exits 1 because `add-styled-text-layers` and `reduce-agent-context-overhead` are both active (`%TEMP%/opencut-unchecked-specs.log`, `opencut-unchecked-moon.log`). GitHub reports no reviews and `REVIEW_REQUIRED`; independent visual review and designated CODEOWNER review are still unrecorded. The foundation CI job fails on the unsuccessful OpenSpec/render prerequisites. PR #120 is currently not draft; that state does not establish readiness.

All five previously unchecked tasks remain incomplete, with the new native mismatch explicitly added to tasks 3.3/5.4. No code was changed, no approval was inferred and no specs were synchronized or archived during this reassessment.

## Status

Implementation and automated verification follow the explicitly approved proposal. All required implementation suites pass on the pinned toolchain. This change is **not ready to archive or merge**: independent visual/CODEOWNER review remains unrecorded and the protected gate rejects another active change. The unrelated `reduce-agent-context-overhead` change and pre-existing working-tree edits are preserved.

| Verification dimension | Outcome |
| --- | --- |
| Completeness | 13/18 tasks complete; remaining tasks require review and protected archival gates |
| Correctness | All 8 delta requirements have implementation and automated scenario evidence below; independent visual reference review is pending |
| Coherence | Original ownership graph preserved; 20 architecture tests pass; contracts, documentation and design synchronized with implementation |

The `openspec-verify-change` workflow was run for this change using pinned OpenSpec status/apply context and the proposal, design, tasks and four delta specs. Its critical findings are the review and archival gates listed below. Synchronization and archival were intentionally not run because the repository explicitly forbids proceeding when another active change blocks the protected gate. Living specs have not been presented as updated or the issue as fully completed.

## Requirement and scenario traceability

| Requirement / scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| Grapheme indexed spans: Unicode, malformed ranges, shaping and false overrides | `model/styled_text.rs`, `validation/styled_text.rs`, `fonts/shaping.rs` | Official Unicode 16 GraphemeBreakTest vectors; canonical valid/invalid catalog; cross-run graphemes and 256-span boundaries; shaping ligature/bidi/false-override test |
| Ordered paint stacks: composition, inheritance/empty, inclusive numeric limits | `validation/styled_text.rs`, `render_artifact/text/layers.rs` | Catalog parity; finite/boundary tests; ordered-paint versus empty/legacy raster tests; atomic invalid mutation tests |
| Reversible edits: aliases, lifecycle/replacement, stale/missing targets | Existing timeline/draft operations with extended document/style models; evaluated component overrides retain spans | `edits_preserve_projection_aliases_history_and_atomic_failures`; `documents_survive_copy_split_move_trim_components_and_drafts`; MCP `rich-text-workflow.ts`; independent root-slot and nested/local/outer repeater span propagation tests |
| Schema 20: current/history/drafts, invalid sources, interruption recovery | `model.rs`, `migrations.rs`, `store.rs`, canonical draft validation | `schema_19_to_20_preserves_history_drafts_and_reopen`; malformed retained styled draft rejection; current/undo/redo forbidden-field rejection; `font_and_legacy_draft_activation_recovers_every_publication_phase` now covers source 19 as well as 18 |
| Preserved schema 19 font milestone: mixed snapshots, missing fonts, all fault phases | Existing migration/font preparation, extended current-version bounds | `font_resolution.rs`, retained draft/font migration and existing publication fault-injection suites; future-version fixture uses current + 1 |
| Shared bounded raster: geometry and work rejection | Shared text measurement/rasterization, finite Gaussian kernel and union coverage | Fractional stroke/signed shadow padded bounds; exact inclusive paint work limit; independent Gaussian impulse (41/25/15 alpha), zero and smallest-positive sigma; duplicate glyph shadow union equality; existing transformed text/raster limits |
| Render intent parity: frame/range/draft/export, legacy and failures | Existing managed fonts/render pipeline, extended evaluated span/layer data | `native_rich_text_render_conformance`: plain, run styles, spans/paints, scaled components, slots, rotated repeaters, lossless legacy comparisons, SSIM >= 0.99 and PCM RMS <= 0.0001; existing animated text and managed font/integrity/confinement suites |
| Governed additive contracts: runtime support, malformed/unsupported versions | Headless status/typed inputs, bridge Zod schemas, catalogs and ownership | Rust protocol tests; `styled-text.test.ts`; canonical MCP surface comparison; `contracts:check`; integration and packaged smoke |

All implementation changes belong to these requirements. Validation stays in the existing core validation owner; model helpers only map checked indices. Scoped component/repeater IDs remain in render plans while glyph artifact filenames use deterministic indices, avoiding invalid Windows filename separators. This is covered by the combined native conformance fixture. No architecture dependency edge was added. Architecture tests enforce the original ADR 0003 matrix. Headless and bridge only deserialize/translate contracts. Scenes using the new span/paint fields also select the existing detail-preserving range encoding preset to meet the approved visual parity threshold. No Python worker contract changed.

## Review still required

- **Critical — task 3.3:** native parity and analytical raster references are automated, but independent review of the visual fixtures has not been recorded. Review `crates/editor-core/src/renderer/golden/rich_text.rs` and the analytical geometry/compositing tests in `crates/editor-core/src/render_artifact/text/layers.rs`. The combined native fixture now includes scaled components, rich-text slot overrides and rotated repeater copies; independent visual review is still pending.
- **Critical — task 4.2:** designated contract CODEOWNER `@matiHirCab` must review the implementation, canonical catalogs, consumer changes and parity evidence. The earlier “Approve” approved the specification before implementation; it is not recorded as implementation review.
- **Critical — tasks 5.3–5.5:** the protected gate rejects both active changes. Repository policy forbids archival when another active change blocks the gate. Do not archive or modify `reduce-agent-context-overhead` as part of this issue.
- **Toolchain:** `.prototools` pins Rust 1.97.0, Bun 1.4.0 and Moon 2.3.3. The default Rust is 1.93.0, but Rust 1.97.0 with Clippy/rustfmt is installed and is selected explicitly for final verification. Bun is 1.4.0. `proto use` failed fetching its plugin from ghcr; direct pinned-tool invocations avoid changing repository configuration or the user default. Moon 2.3.3 and OpenSpec 1.5.0 run through isolated Bun caches.

## Verification commands and logs

Full uncommitted logs are in `C:/Users/matia/AppData/Local/Temp/` with prefix `opencut-styled-`. Rust-dependent commands select `RUSTUP_TOOLCHAIN=1.97.0`; bridge commands run from `apps/agent-bridge`.

| Command | Exit / result | Log suffix |
| --- | --- | --- |
| `cargo fmt --check --all` | 0 | `fmt-pinned.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0; dependency-only future-compatibility advisory for existing proc-macro-error2 2.0.1 | `clippy-pinned.log` |
| `cargo test --workspace -- --test-threads=1` | 0; complete workspace passes, including 300 core unit tests and 20 architecture checks | `workspace-pinned.log` |
| `cargo test -p opencut-editor-core native_rich_text_render_conformance -- --test-threads=1` | 0; required real-media fixture passes with two shadows, two strokes, fill, span replacement, slots and transformed component/repeater copies | `native-pinned.log` |
| `bun run typecheck` | 0 | `ts-final.log` |
| `bun run lint` | 0 | `lint-final.log` |
| `bun run test -- --maxWorkers=1` | 0; 394 tests in 22 files | `unit-final.log` |
| `bun run contracts:check` | 0; Rust governed suites plus 328 TypeScript tests | `contracts-pinned.log` |
| `bun run test:integration` | 0; 11 end-to-end MCP tests | `integration-pinned.log` |
| `bun run test:smoke` | 0; 6 packaged smoke tests against the release executable | `smoke-pinned.log` |
| `bun run scripts/run-python-tests.ts` | 0; 10 unittest + 5 pytest | `python.log` |
| `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` | 0; 27 items | `specs-final.log` |
| `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` | 1; archive-only policy rejects this change and `reduce-agent-context-overhead` after all 27 spec/change validations pass | `moon-final.log` |

Native verification sets `OPENCUT_GOLDEN_REQUIRED=1`, uses the checked-in DejaVuSans fixture and isolated FFmpeg/ffprobe 7.1.1 executables in `opencut-styled-ffmpeg711` under the temporary directory. Ordinary workspace tests intentionally leave existing subprocess helpers and explicit reference recapture helpers ignored; their parent tests run the applicable helpers. Native styled conformance is run explicitly, so its required media assertions do not silently skip.

Previously encountered failures are not hidden: initial parallel Rust runs failed existing sampler/publication timing tests; initial parallel bridge unit tests hit the existing TTS cancellation timeout (serial retry passed). The architecture review found private dependency violations, which were fixed by moving semantic validation into its existing owner (20 architecture tests then passed). Strict lint findings and canonical numeric fixture mismatches were corrected. The upgraded schema-20 repeater fixture initially retained a schema-18 assertion that plain text had no run document; it now asserts the exact unchanged plain run, and passes. The expanded native fixture exposed invalid Windows glyph artifact filenames and range/export SSIM 0.988; portable filenames and the existing detail-preserving range preset fixed both, and the complete fixture passed at the required >=0.99 threshold. A concurrent Windows test rebuild hit LNK1104 on a running test executable; final Rust checks are serialized. Initial OpenSpec cache/permission failures were repaired. Native media verification requires FFmpeg 7.1.1 because the host FFmpeg rejects the repository's existing `filter_complex_script` invocation; an isolated temporary runtime is used without changing application configuration.









## Approved regression corrections — 2026-09-18

Both reviewed regressions are corrected under the existing legacy fallback, explicit stack precedence, render parity and bounded-work requirements. No public API, schema or migration change was made for these corrections.

The shared `text::paint_legacy` helper retains the original shadow pass, default stroke joins, per-glyph outline/fill order, individual colors/faces and visual glyph order without clearing the destination. Mixed rendering coalesces contiguous legacy glyphs across faces/colors. Explicit groups compare effective stacks and faces, ignoring overridden colors; explicit empty stacks remain distinct. Both painting and work accounting call the same grouping function and retain logical segment ordering.

### Regression evidence

- Before production edits, all three new tests failed (4 existing tests passed): invisible trailing-space span changed legacy pixels, ignored color changed explicit pixels, and ignored color incorrectly exceeded the inclusive work budget. Full red-run evidence: `%TEMP%/opencut-styled-fix-red.log`.
- After the fix, all 7 raster tests passed (`green.log`): dimensions and full PAM bytes match for `AV ` with legacy outline/shadow, mixed run colors/faces and RTL; red 60px stroke/white fill is unchanged by a green V override; real stack replacement changes output; existing layer-order and empty-stack tests pass; 4096-square/16-layer acceptance and excessive-work rejection remain exact.
- `paint_regression_conformance` repeats both reported cases through native frame rendering and compares decoded RGB exactly. The complete native conformance test also retains frame/range/draft/export, transformed component/repeater and audio parity assertions.

### Refreshed implementation checks

All logs below are under `%TEMP%`, prefixed `opencut-styled-fix-`. Rust checks use 1.97.0. Native checks require FFmpeg/ffprobe 7.1.1, the checked-in DejaVuSans fixture and `OPENCUT_GOLDEN_REQUIRED=1`.

| Command | Exit / evidence | Log suffix |
| --- | --- | --- |
| `cargo fmt --check --all` | 0 | `fmt.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | `clippy.log` |
| `cargo test --workspace -- --test-threads=1` | 0; 303 core unit tests, 20 architecture checks and all workspace suites | `workspace.log` |
| `cargo test -p opencut-editor-core native_rich_text_render_conformance -- --test-threads=1` | 0; both native regressions and existing cross-intent checks | `native-final.log` |
| `bun run typecheck` / `bun run lint` | 0 / 0 | `ts.log`, `lint.log` |
| `bun run test -- --maxWorkers=1` | 0; 394 tests | `unit-retry.log` |
| `bun run contracts:check` | 0; Rust governed suites and 328 TypeScript tests | `contracts.log` |
| `bun run test:integration` | 0; 11 tests | `integration.log` |
| `bun run test:smoke` | 0; 6 tests | `smoke.log` |
| `bun run scripts/run-python-tests.ts` | 0; 10 unittest and 5 pytest | `python.log` |
| Strict all-spec validation | 0; 27 items | `specs-final.log` |
| `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` | 1; archive-only rejection names both active changes | `moon-final.log` |

The initial bridge run timed out in two existing provider-worker tests during concurrent compilation (392 passed); the complete serial-worker retry passed. Initial spec commands failed because a temporary CLI cache lacked its executable; a fresh isolated cache passed. The first native attempt lacked the temporary FFmpeg executable, restored from the existing archive. The second exposed a missing previews directory in the new test fixture; creating that directory fixed setup. Formatting, strict Clippy and native conformance were rerun afterward. Workspace evidence remains applicable to production code and all non-native suites; its only subsequent Rust edit was that native-only fixture directory creation, exercised by the mandatory native rerun. Full failure logs are retained as `unit.log`, `specs.log`, `moon.log`, `native.log` and `native-retry.log`.

### OpenSpec verification reassessment

The `openspec-verify-change` workflow was rerun using refreshed pinned status/apply context, all four delta specs, proposal, design, tasks and implementation/test evidence. Completeness is 13/18 tasks; all 8 requirements retain automated evidence. Correctness: both confirmed rendering mismatches are resolved with red/green regression evidence. Coherence: the shared painter stays in the existing core render-artifact owner; grouping, accounting and the approved design agree.

Five critical completion gates remain: task 3.3 independent visual review; task 4.2 designated CODEOWNER review; task 5.3 protected gate blocked by both active changes; task 5.4 full conformance sign-off pending those reviews; task 5.5 synchronization/archival and post-archive checks. No new code/spec divergence was found. This is not archive- or merge-ready. The unrelated `reduce-agent-context-overhead` change is preserved and must not be modified or archived to clear this blocker.
