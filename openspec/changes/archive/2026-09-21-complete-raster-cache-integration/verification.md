# Verification index: complete-raster-cache-integration

Status: complete. Implementation, automated conformance, final CODEOWNER review, living-spec synchronization, archival and the protected post-archive gate all pass. Evidence limitations remain recorded below.

## Scope and approval

User approved the proposal, design, delta specifications and tasks with �Yes� on 2026-09-21. This approval authorizes implementation; it is not recorded as review of the final contract diff. Canonical owner is @matiHirCab. New consumer paths are registered in `.github/CODEOWNERS` and `contracts/contract-ownership-v1.json`.

The original issue-36 renderer cache and its archived change remain intact. The architectural clarification is explicit: the prior bridge used one process per request, so renderer-local bytes could not survive between bridge calls. The approved follow-up introduces a render-only worker; it does not claim the original completion report proved cross-request reuse. Non-render and concurrent overflow requests retain one-shot execution.

## Requirement and scenario evidence

| Scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| W1/W2 | `HeadlessClient.call`, synchronous worker reservation, existing `callHeadless`; shared native `dispatch` | `render-worker.test.ts`: sequential PID/count reuse and one-shot overlap; `headless.test.ts`; native protocol suite; default MCP integration and packaged smoke (results below) |
| W3/W4 | `worker.rs` bounded input/output and closed Envelope; `RenderWorker.#data` incremental fatal UTF-8 parsing, request correlation and terminal tracking | `worker_contract_and_inclusive_framing`; canonical event/operation fixture test; exact 16 MiB event acceptance, oversized/invalid UTF-8/wrong-ID/duplicate/malformed/crash discard cases |
| W5 | request deadline starts before readiness; five-second startup deadline; no fallback/replay | startup exit/version rejection, request deadline during startup, readiness deadline, active timeout tests |
| W6/W7/W8 | worker retirement and process-tree stop before cleanup; bounded stop leaves unconfirmed slot retired; deferred cleanup after observed exit; close blocks reservations; draft cleanup included | cancellation with one-shot overflow and draft temporaries, timeout, crash replacement, active shutdown, typed-error reuse; scoped adapter write-failure regression |
| W9 | immutable `Renderer::with_request_id` adapter; services retained but project/draft snapshot read in dispatch per request | `scoped_requests_share_cache_without_changing_adapter_or_identity`; native worker rejects mutations/invalid identity; native worker observes edited revision, an updated draft under the same ID, and rejects stale revision |
| X1/X2 | feature-gated counters at the actual raster closure path, shared Arc cache | native headless worker renders text/shape/SVG through frame/range/export/draft with exactly three initial misses; revision update misses; fresh worker starts cold. `render-worker-native.test.ts` observes real headless stderr counters through actual HeadlessClient calls, normal error reuse and a second cold client |
| X3/C2 | production evaluation/materialization of valid variants, existing key definitions | `valid_dependency_variants_match_fresh_and_keep_unrelated_keys`: 21 variants covering document/runs/spans/layout/paint ordering/font hashes/vector geometry/fill/stroke/SVG viewport/order/settings/identity/revision; changed keys miss, match fresh output and subsequently hit while base entries remain reusable. `composed_parent_sampling_misses_but_translation_reuses`: parent scale vs translation |
| X4/C1/C3 | local raster identity excludes composition-only timing/placement/opacity/occurrence identity | `occurrence_and_composition_changes_reuse_rasters`: independently asserted position, opacity and start time, plus component/repeater byte reuse; `warm_cache_drafts_revisions_history_and_reopen`: distinct same-revision drafts, atomic failures, undo/redo/reopen |
| X5/V1 | canonical preflight remains before lookup/materialization | `warm_preflight_matches_cold_across_routes_before_any_lookup`: cold/warm code/retryability/stage, hit AND miss counts, prohibited adapter events, unchanged project and existing output for frame/range/export. Invalid finite values, unsupported profile, missing binding/reference, missing/tampered fonts, lexical/canonical escapes, glyph budget and readiness. Unsupported SVG is rejected at the editing boundary before it can become a valid draft/project snapshot |
| X6/V2 | existing preflight/collision/publication ordering and scoped workspace RAII | invalid tests target an already-existing export and retain canonical errors; warm-cache write-failure and scoped-identity tests assert established FFmpeg error and workspace cleanup |
| B1/B2 | inclusive entry/key+payload byte budgets, LRU, Arc bytes, no lock across raster work, failed/poisoned cache bypass | `inclusive_budgets_lru_and_oversize_bypass`, `concurrent_misses_are_immutable_and_accounted_once`, `failed_rasters_retry_and_poison_bypasses` |
| L1/L2/L3 | same evaluated semantics and renderer routes, no persisted cache data | `native_raster_cache_all_routes_conformance`: independent red/blue interior pixels, cold/warm/fresh frame equality, fresh range/export decoded pixel and PCM equality, non-silent audio and SSIM/RMS comparisons, draft parity and unchanged project; existing pinned glyph plan test and workspace rendering/history regression suites |

## Environment

- Windows native runtime evidence only; POSIX process-group behavior is implemented but not executed on this host.
- Windows, repository HEAD `c298eedc3bd9ec1fa786585ac5fac5d1594cc7d0`, pre-existing working-tree issue-36 changes preserved.
- Bun 1.4.0. Rust checks explicitly set `RUSTUP_TOOLCHAIN=1.97.0`; the machine default stable toolchain is not used as pin evidence.
- FFmpeg/FFprobe 7.1.1 essentials build, restored from the existing local ZIP under `%TEMP%/opencut-35-ffmpeg-7`. The historical extracted binary path had disappeared; the first native invocation failed, and the subsequent actual-native runs passed.
- Reviewed font `crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf`, SHA-256 `ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280`.
- Native Rust runs set `OPENCUT_GOLDEN_REQUIRED=1`, both native executable paths and `OPENCUT_TEST_FONT_PATH`. Instrumented bridge run additionally sets `OPENCUT_RASTER_CACHE_TESTS_REQUIRED=1` against a feature build.
- Workspace suite runs serially (`RUST_TEST_THREADS=1`), following the original archive's observed parallel native-render contention. No claim of parallel reliability is made.
- Moon 2.3.3 and the pinned bunx OpenSpec cache were repaired. The protected gate now rejects only this active change, as expected before archival; this is not a final gate pass.

## Check ledger

Full logs are uncommitted under `%TEMP%` (`C:/Users/matia/AppData/Local/Temp`). An exit-0 check is reusable only while its relevant source inputs and environment remain unchanged.

| Check | Result | Log |
| --- | --- | --- |
| Rust formatting | PASS, exit 0 | `opencut-worker-fmt-final2.log` |
| Workspace strict Clippy, default | PASS, exit 0 | `opencut-worker-clippy-default2.log` |
| Workspace strict Clippy, test hooks | PASS, exit 0 | `opencut-worker-clippy-final2.log` |
| Feature headless tests including actual native worker | PASS, exit 0: 5 unit, 28 protocol, 3 worker | `opencut-worker-headless-final2.log` |
| Native core raster-cache tests | PASS, exit 0: 13 actual native/configured tests | `opencut-worker-core-final2.log` |
| Actual native bridge + worker protocol + legacy client | PASS, exit 0: 20 tests | `opencut-worker-bridge-final2.log` |
| TypeScript typecheck | PASS, exit 0 | `opencut-worker-typecheck8.log` |
| Full bridge lint | PASS, exit 0 | `opencut-worker-lint-final2.log` |
| Full bridge unit suite, serial files | PASS, exit 0: 412 passed, 1 explicitly skipped instrumented-only native test; separate required native run above passed | `opencut-worker-unit-serial.log` |
| Hermetic Python tests | PASS, exit 0: unittest 10 and pytest 5 | `opencut-worker-python.log` |
| Strict OpenSpec all-spec validation | PASS: 29 items with canonical pinned bunx restored and invoked by the final Moon run; earlier local fallback and tempdir failure retained below | `opencut-worker-moon-prearchive-final.log` (earlier fallback: `opencut-worker-openspec-pinned.log`) |
| Serial native Rust workspace | PASS, exit 0; seven intentional helper entries ignored, exercised by parent tests | `opencut-worker-workspace-serial.log` |
| Contract parity | PASS, exit 0: native suites and 346 TS tests | `opencut-worker-contracts-final.log` |
| Default binary MCP integration | PASS, exit 0: 11 tests after crash containment | `opencut-worker-integration-final.log` |
| Packaged smoke | PASS, exit 0: 6 tests after crash containment | `opencut-worker-smoke-final.log` |
| Moon protected pre-archive gate | EXPECTED BLOCK, exit 1: only `complete-raster-cache-integration`; 234 policy tests and 29 spec items pass | `opencut-worker-moon-prearchive-final.log` |

The long workspace command began before the final headless containment change and the additional canonical-escape assertion. Its unchanged core/desktop coverage is reused; the modified core test was rerun in the 13-test native suite and final default/instrumented headless suites cover the changed transport. No historical passing check is substituted for those affected checks.

Both Clippy modes retain Cargo's upstream future-incompatibility advisory for `proc-macro-error2`; no warning suppression was added. No golden references or assertions were weakened.

## Failures and superseding evidence

Earlier local logs are retained: missing native edit transform, unsafe fixture font path, then absent FFmpeg path (`native.log`, `native2.log`, `native3.log`) were corrected and superseded by `headless-final.log`. Bridge fixture extension resolution and project-state schema mistakes were corrected before `bridge-final.log`. Type inference/style and test Vec/Box errors were corrected before final typecheck/Clippy. An attempted headless `--lib` check was invalid because the package has a binary target; the full package suite supersedes it. Sandboxed lifecycle runs could not perform Windows process-tree termination; the same tests subsequently passed with approved elevated execution. Bunx tempdir EPERM and the missing Moon launcher are environmental failures retained explicitly. The locally verified pinned OpenSpec package is a diagnostic fallback, not a replacement for the protected Moon gate.

## Pre-approval completion gates (historical)

Obtain designated contract-owner review of the concrete diff, then synchronize/archive only this follow-up and pass the unchanged protected gate. Required implementation checks passed; the final conformance review and the same-draft-update coverage check are recorded below. Artifact approval is not silently relabeled as final code review. The original archive and golden references remain unchanged.

## Final lifecycle correction

Inspection found that taskkill alone cannot contain descendants after their parent has already crashed. Worker startup now installs a Windows kill-on-close Job Object with a non-inheritable process-lifetime handle. This implements the already-approved W6-W8 failure behavior; no domain/private-owner dependency edge is added. Headless adds a direct target-Windows dependency on the existing locked `windows-sys 0.61.2` package; no new package/version is introduced. POSIX retirement still kills the process group after leader exit. See the implementation decision and Microsoft Job Objects link in the renderer fixture guide.

`crashed_worker_terminates_renderer_descendants` starts a real blocking renderer subprocess, pins its process handle, forcibly kills the worker, and requires the child to exit; exit 0 in `opencut-worker-crash-tree.log`. The bridge cancellation test now keeps one-shot overflow active during cancellation, asserts a spawned descendant is gone, and asserts a published file survives. These are stronger lifetime assertions, not timeout relaxation.

The post-change concurrent unit run (`opencut-worker-unit-final.log`) failed one existing TTS cancellation test at its unchanged 5-second timeout; 411 tests passed including the worker lifecycle cases. The full serial rerun passed 412 tests (one separately executed instrumented-only test skipped); the parallel failure remains recorded. No timeout or assertion was weakened.

## Final OpenSpec verification (2026-09-21)

Applied the repository `openspec-verify-change` workflow using the pinned 1.5.0 CLI status and apply context. Reconciled the proposal, both delta specifications, updated design, 31 tasks, code and all logged checks. Planning completeness (`isComplete: true`) is not treated as implementation completion.

| Dimension | Result |
| --- | --- |
| Completeness | 31/31 tasks completed, including final owner review, synchronization/archival and passing post-archive validation. |
| Correctness | All six changed requirements, 19 delta scenarios (including four unchanged legacy protocol scenarios), and the ten retained cache scenarios have implementation/automated evidence in the index. No remaining implementation mismatch identified. |
| Coherence | One persistent worker with one-shot overflow, shared canonical dispatch, immutable identity, bounded disposable core cache, native process containment, and additive fixture-governed protocol match the approved scope. No editor-core private-owner edge or persisted-schema change. |

The conformance pass added the final same-ID draft-update native assertion: changed pixels equal a fresh worker, the repeated draft hits, and authoritative project bytes remain unchanged. All three instrumented worker tests passed in `opencut-worker-draft-final.log`; final feature Clippy and formatting passed in `opencut-worker-clippy-final3.log` and `opencut-worker-fmt-final3.log`. This changes only a feature-gated test body. The default headless binary was rebuilt successfully afterwards (`opencut-worker-build-default-final.log`).

**Pre-approval completeness item (resolved by approval below):** task 2.2 requires @matiHirCab to review the concrete `contracts/render-worker-v1.json`, ownership entries and governed consumers. ADR 0002 states: �Contract changes require synchronized native declarations, fixtures/catalogs, parity tests, and owner review in one pull request.� Local artifact approval is not asserted to satisfy review of the final implementation. Obtain that review before task 8.2; then rerun the unchanged protected gate for task 8.3. No PR was created or required merely to present this local reviewable diff.

**Verification limits:** runtime evidence is Windows-only. Parallel TTS cancellation timed out once under concurrent native work; the unchanged full suite passed with serial files. Seven ignored Rust entries are intentional helper subprocess entry points, exercised through parent tests, not skipped feature verification. The opt-in TS native test is explicitly skipped in ordinary unit runs and passed separately against the instrumented native binary. No public cache statistics, golden edits, weakened assertions, commits, pushes, or pull requests were introduced.

Assessment: implementation checks, conformance, designated-owner review and final archival/protected-gate checks are complete.

Final protected pre-archive gate rerun: `opencut-worker-moon-prearchive-final.log`, gate exit 1 solely for this active change; strict spec validation passed 29/29. CODEOWNER approval of the final contract and consumers was requested in the conversation and subsequently received below. No synchronization or archival is authorized by elapsed time or by a planning-artifact completeness flag.

## Final owner approval and archival (2026-09-21)

The user explicitly answered "Yes" to the final CODEOWNER review request for the concrete worker contract, ownership entries and governed consumers, and authorized synchronization and archival. Task 2.2 is complete; this final review is distinct from planning-artifact approval. Implementation evidence remains valid because the finishing edits affect only specifications, approval records and lifecycle task status. Post-archive results are recorded below.

Living specifications synchronized: `agent-bridge` replaces the typed-headless requirement and adds bounded worker transport and isolated request lifetime; `raster-caching` adds cross-request reuse, production-path invalidation evidence and warm preflight conformance. Each synchronized requirement was checked against its delta; unrelated requirements and the original issue-36 archive were preserved. The complete directory, including `.openspec.yaml`, was archived as `2026-09-21-complete-raster-cache-integration`.

Final checks (exit 0):
- `moon run root:openspec-validate`: protected archive-only gate and CI parity policy pass; strict validation reports 28/28 living specifications. Log: `%TEMP%/opencut-worker-moon-postarchive.log`.
- `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`: 28 passed, zero failed. Log: `%TEMP%/opencut-worker-openspec-postarchive.log`.

Task 8.3 was marked complete only after these checks passed. Both checks are rerun after this evidence/task update using the same log paths to cover the final artifact contents. No implementation source changed during this final approval/synchronization/archive step. No commit, push or PR was created.
