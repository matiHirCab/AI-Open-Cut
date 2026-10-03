# Conformance verification ledger

Current state: requested behavior and the user-authorized normal merge of current main4c2897e0 are independently accepted. The final merged-tree required local checks passed; the normal merge commit, push and its exact-head CI are tracked externally. Earlier passing evidence is retained as historical context.

## Requirement and scenario traceability

| Requirement | Scenario | Automated evidence |
| --- | --- | --- |
| Canonical range review preset selection | Resolve named presets across aspects | `canonical_review_aspects_rounding_custom_and_defaults` consumes preview-review-v1.json for landscape, portrait, square, ties and minimum width. |
| Canonical range review preset selection | Preserve project and custom settings | Same fixture test checks omitted/project selection, exact odd custom dimensions, default/explicit fps and audio choices. |
| Canonical range review preset selection | Reject conflicting or unsupported settings | `canonical_review_rejects_bounds_intervals_and_malformed_selections`, `review_request_contract_defaults_and_strict_resolution`, `review_range_transport_rejects_before_io_and_preserves_revision_reopen`, MCP schema tests cover strict shape, bounds, interval and finite fps. |
| Audio enabled immutable review | Review with audio or explicit silence | Native `native_preview_range_and_export_frames_are_consistent_when_ffmpeg_is_available` now routes through omitted review options and verifies default audio PCM against export plus no stream for explicit false. |
| Audio enabled immutable review | Repeat and reopen immutable review | Shared real MCP/packaged `verifyPreviewReviewWorkflow` repeats default/project selection, preserves exact project.json/history.json/all retained draft JSON bytes across successful renders/reopen, and checks undo/redo. Native parity checks visual SSIM>=0.99, audio RMS<=0.0001 and timing<=one frame. |
| Audio enabled immutable review | Preserve early failures | New review participates in the existing fake-adapter all-facades failure helper for missing references, lexical/canonical path escapes and evaluation failures; headless/MCP tests cover revision conflicts before job admission/publication. |
| Compatible audiovisual review tool | Queue default and selected reviews | Fixture-driven schema/forwarding tests plus real MCP and packaged selection workflow and disposable137/138 union review workflow. |
| Compatible audiovisual review tool | Preserve legacy input | Legacy MCP omission remains false; headless numeric request declaration remains unchanged; `bounded_review_preserves_above_bound_legacy_range_routing` proves above-bound legacy dimensions execute with fake adapters while new review fails without I/O. |
| Compatible audiovisual review tool | Reject malformed selection | Rust canonical resolution fixture, headless typed request test and transport errors, MCP invalid/partial/conflicting/unknown selections. |
| Discoverable review preset support | Discover ready and unavailable review | Canonical rendering capability parity, existing readiness filtering, headless review transport test checks capability presence equals readiness for both lists, and real MCP review checks both lists when ready. |

No persisted changes: schema30, migrations, aliases and batch domain edits are unchanged. New range review is ephemeral and read-only. Existing immutable worker/job/cancellation behavior is reused; parameterized worker test verifies new review's MP4 temporary cleanup while retaining the existing descendant termination assertion.

## Completed checks

- `cargo fmt --check --all`: passed, `/tmp/issue71-fmt-final.log`.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed, `/tmp/issue71-clippy.log`.
- `cargo test --workspace`: passed, `/tmp/issue71-workspace-2.log`. Local desktop linker prerequisites use temporary `/tmp/issue71-native-libs` symlinks to installed versioned xcb/xkbcommon libraries via LIBRARY_PATH; no system changes.
- Bridge `bun run contracts:check`: passed all Rust suites and364 TypeScript tests in15 files, `/tmp/issue71-contracts-final.log`.
- Bridge typecheck/lint: passed, `/tmp/issue71-typecheck-3.log`, `/tmp/issue71-lint-final.log`.
- Bridge unit tests:436 passed, one existing skipped, `/tmp/issue71-unit-3.log`.
- Full MCP `bun run test:integration`:16 passed with the current retained-draft byte assertions, `/tmp/issue71-integration-final.log`.
- Required release native golden: passed (396.95 seconds), `/tmp/issue71-native-golden-release-2.log`; generated performance report validation passed, `/tmp/issue71-golden-report-validation.log`.
- Packaged `bun run test:smoke`:10 passed with the current retained-draft byte assertions, `/tmp/issue71-packaged-2.log`.
- Hermetic Python workers: passing unittest/pytest suites, `/tmp/issue71-python.log`.
- Focused native review audio/visual/export parity: passed, `/tmp/issue71-native-review.log`.
- Strict all-spec validation and prearchive protected task:38 specifications/changes passed; protected task rejected only this own active change as expected, `/tmp/issue71-policy-prearchive-5.log`.
- Disposable contract integration: passed, see contract-integration.md.

## Corrections and environment evidence

Initial test authoring had invalid Project equality/reopen usage and a missing fixture transform; fixed before passing runs. The original fixture failure is retained at `/tmp/issue71-core-focused-initial-fixture-failure.log`. MCP digest/capability expectations were synchronized to the reviewed manual catalog. Independent review found instruction-prefix and missing integration/persistence/cleanup coverage; all three findings were corrected and accepted in implementation-review.md.

Initial unit run failed the unchanged descendant PID-absence assertion because the container init did not promptly reap orphan processes. Tests now run under a task-local Python PR_SET_CHILD_SUBREAPER harness outside the repository; the original assertion remains intact. Initial expanded instruction prefix failed its contract and was shortened. Logs: `/tmp/issue71-unit-initial-failures.log`, `/tmp/issue71-unit-2.log`; final unit run passes.

Initial packaged run had three unrelated30-second timeouts during concurrent full workspace/release compilation; no test limits or assertions were changed. The final complete retry passed10/10. Initial log: `/tmp/issue71-packaged.log`.

Pinned Bun1.4.0, Moon2.3.3 and Rust1.97.0 were installed under task-local /tmp homes and verified against .prototools. Moon's cache locations use task-local MOON_HOME/PROTO_HOME/XDG_CACHE_HOME, and supported MOON_TOOLCHAIN_FORCE_GLOBALS=1 uses these verified exact versions; the ordinary protected task and its checks are unchanged. Earlier bootstrap/cache/offline failures are retained at `/tmp/issue71-policy-prearchive*.log`. Independent implementation review accepted this bootstrap as equivalent pinned execution, not a policy bypass.

Supplemental debug-native all-library sweep was interrupted after serial unrelated golden suites caused build contention; its initial native golden failure and full output remain at `/tmp/issue71-core-lib.log`. The required release golden run isolated that failure to the host font hash differing from the reviewed font. Checked-in `tests/fixtures/fonts/DejaVuSans.ttf` has the exact required SHA256 `ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280`; the release retry explicitly uses that file. No golden references/tolerances were changed. Initial failure: `/tmp/issue71-native-golden-release.log`; final retry passed at `/tmp/issue71-native-golden-release-2.log`. The captured `/tmp/issue71-render-baseline-linux.json` passed the external report validator.

## Pending gates

Specs synchronized and archived on2026-10-03; ordinary protected task passed37/37 specifications and CI policy. Verified commit/draft PR publication and terminal exact-head CI remain pending in the external delivery ledger. Publication remains draft only; no merge/deployment. Delegated reviews do not substitute for human @matiHirCab CODEOWNER acceptance.

Latest test-only readiness assertions explicitly verify the capability is absent from both lists when rendering is unavailable; targeted test, final fmt and strict Clippy pass (`/tmp/issue71-unavailable-capability.log`, `/tmp/issue71-fmt-final-2.log`, `/tmp/issue71-clippy-final.log`). All production inputs are unchanged from the full passing checks.

## OpenSpec verification assessment

Completeness:11/11 implementation tasks completed;4/4 requirements and10/10 scenarios covered. Correctness: canonical core resolution/defaults, additive transports and unchanged legacy behavior match the reviewed design. Coherence: existing renderer, worker/job envelopes, persisted schema30 and failure-order routing are reused. Independent final conformance review accepted with no remaining implementation findings. No implementation warnings or suggestions remain. Administrative synchronization/archive, postarchive gates and publication are explicitly tracked separately; they are not claimed complete by this assessment.

Postarchive protected/strict gate passed: `/tmp/issue71-policy-postarchive.log`. Publication re-fetch found PR137 merged into currentmain480bd8d7; integration reconciliation is required before publication.

## Current-main verification

Final publication base is480bd8d7f326eb4cf204dd09f8b9d28f3f61e802 after PR137 merged; feature rebased with only deliberate catalog digest conflict. Independent reconciliation review accepted. Production Rust/headless files are byte-identical to original verified feature commit0db0c728, so workspace, fmt/strict Clippy, native review/export parity and required release golden evidence remain applicable.

Affected current-base checks rerun: typecheck and lint pass (`/tmp/issue71-reconcile-typecheck.log`, `/tmp/issue71-reconcile-lint.log`); unit472 pass/one existing skip (`/tmp/issue71-reconcile-unit.log`); hermetic Python pass (`/tmp/issue71-reconcile-python.log`); full MCP integration16 pass (`/tmp/issue71-reconcile-integration.log`); packaged10 pass (`/tmp/issue71-reconcile-packaged.log`); protected strict specification/policy task passes37/37 (`/tmp/issue71-policy-reconciled.log`).

Initial current-base contract run omitted the task-local subreaper and failed the unchanged descendant PID assertion for both parameterized cases; original log retained at `/tmp/issue71-reconcile-contracts.log`. Complete retry uses the same previously reviewed subreaper without repository assertion changes. Complete retry passed every governed Rust fixture suite and365 TypeScript tests in15 files: `/tmp/issue71-reconcile-contracts-2.log`.

## Exact-head CI regression correction

Initial published commit3c5e02fe passed all three platform correctness jobs, specification/contract parity and packaged integration/smoke, but its native render job failed in `native_worker_reuses_rasters_across_requests_and_restarts_cold`. The review request example had been inserted before legacy draft/export examples, shifting the feature-enabled test's positional export lookup. This was a genuine fixture regression: the feature-enabled native worker suite had not been executed locally before initial publication. Full CI failure evidence is retained externally in `ci-render-failure.log`.

Correction appends both new operation and request after the unchanged four legacy entries. No production behavior, render tolerance or existing assertion is weakened. Added `additive_review_fixture_preserves_legacy_request_positions` catches regression in the normal suite. Exact feature-enabled native worker suite passed4/4; fmt and strict workspace Clippy passed. Independent Sol medium fix review accepted. Initial correction contract check caught an operation-list/request-list order mismatch; both arrays are now aligned and the initial correction failure log is retained. Final contract/native cache segment results and replacement exact-head CI are recorded externally when terminal.

Correction final local checks passed: all governed Rust suites plus365 TypeScript contract tests (`/tmp/issue71-ci-fix-contracts-2.log`); exact aligned feature-enabled native worker4/4 (`/tmp/issue71-ci-fix-native-worker-aligned.log`); native core raster-cache13/13 (`/tmp/issue71-ci-fix-raster-core.log`); native bridge cache worker1/1 (`/tmp/issue71-ci-fix-raster-bridge.log`); normal headless complete suite (`/tmp/issue71-ci-fix-headless-2.log`); feature-off binary restored (`/tmp/issue71-ci-fix-normal-build-final.log`). No executable bridge or renderer implementation changed during correction, so original complete current-base bridge/render conformance evidence remains applicable. Replacement exact-head CI is tracked externally.

## User-authorized PR conflict resolution

The user requested fixing conflicting OpenCut PRs after PR138 merged. PR139 normally merges main4c2897e0499281c75bf23b6abac99e2f4b15d2dc (actualmergedPR138head360f8187193ddf54b38224c16e9ce2e72fe045db), preserving all speech/artifact/current-main semantics and issue71 request selection/defaults. No rebase, force push, merge of the PR or deployment. Four shared-file textual conflicts resolved manually, both owner/test/capability sections retained and expanded catalog digest77e1e52c independently confirmed. All existing main tool definitions are unchanged; newreviewtool uses the updated shared artifact-aware job definition.

The reviewer accepted strengthened real MCP/packaged coverage: default completed review metadata/resource links and explicit artifact reads match exact published MP4 bytes. Current merged production Rust/headless, render-worker fixture, native golden references/font and pinned tools are identical to e098b1be; prior passing native audiovisual/cache/golden evidence remains applicable. The complete required fmt/strictClippy/workspace and bridge contracts/type/lint/unit/integration/packaged/Python/protected suites are rerun on the final merged tree regardless. Their results are recorded before the merge commit; new exact-head CI remains a separate external gate.

Merged-tree completed checks: `cargo fmt --check --all`, strict workspace Clippy, full `cargo test --workspace` passed (`/tmp/issue71-main74-fmt.log`, `-clippy.log`, `-workspace.log`). Bridge type/lint passed on the final resource-byte assertions (`/tmp/issue71-main74-typecheck-final2.log`, `-lint-final.log`); unit501 pass/one existing skip (`-unit.log`); contracts394 pass plus all governed Rust fixtures (`-contracts.log`); Python suites pass (`-python.log`); ordinary protected strict spec/policy gate passes (`-policy.log`). Affected native bridge cache test passed1/1 and feature-off binary was restored (`-native-bridge.log`, `-normal-build.log`). First full integration16 passed (`-integration.log`); a final full feature-off integration rerun avoids any ambiguity from the brief overlapping feature-mode build. Packaged smoke10 passed (`/tmp/issue71-main74-packaged.log`); final feature-off full integration16 passed171.44 seconds (`/tmp/issue71-main74-integration-normal.log`). All required local merged-tree gates are complete.
