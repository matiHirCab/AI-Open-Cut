# Issue #72 conformance review and evidence

Status: implementation, all seven approved scenarios and repository-required local verification pass. Final conformance review finds no unresolved implementation/design/spec/coverage mismatch; remote draft/CI publication remains blocked. User approved proposal/design/spec/tasks on2026-10-09 ("yes"). This isolated branch starts from main c0031b6011a9000ba698bb105ecd0a122d29a625. Issues71 and15 were closed and72 open when dependency readiness was inspected. No recoverable implementation existed. Issue70 is separately verified/pushed; neither branch is merged or deployed.

## Completeness and correctness

| Approved scenario | Owner and automated evidence |
| --- | --- |
| P1 exact reuse | `render_artifact/preview_cache` streaming exact-float identity; renderer clone/correlation tests; native frame/range execution-count test; headless persistent-worker test; actual source and Bun-compiled MCP tests capture existing-feature stderr plus actual ordinary `SystemProcessExecutor::execute` counts. |
| P2 every identity dimension | Renderer independently varies projectID/revision/time, range start/end/W/H/FPS/audio, same-revision candidate content and adapter/font-root scopes; each range variant agrees with a fresh renderer. Native frozen AV test changes selected source WAV bytes at unchanged project/revision/path and compares fresh cold; native default-font test changes valid sfnt bytes at unchanged path, then restores the original reusable key. Process-owner tests vary native bytes/backend paths and reject unresolved/nonfile/script/forwarding bindings. Worker tests isolate same-revision draft color and restart cold. |
| P3 bounds | Cache tests inclusive entry/byte limits, LRU access/replacement and oversized/empty admission; filesystem reader rejects oversize before allocation, directories, symlinks and empty data. Actual native PNG with a deliberately tiny test cache renders twice uncached with equal independently checked pixels. Concurrent clones retain immutable bytes within both bounds. |
| P4 failures | Poison/concurrent retention tests; failed partial ordinary execution and rename/publication failures do not insert; optional unsupported/failed bounded reads render successfully uncached; retries succeed. Existing native worker terminal-error recovery remains unchanged. |
| P5 preflight first | Warm/cold invalid frame/range and glyph-limit typed code/retryability/stage equality; warm unavailable adapter rejects before publication; native source deletion rejects before lookup with unchanged counters and preview directory. MCP stale revision and late unresolved alias rollback preserve full authoritative state/owned files; subsequent valid previews still hit. Existing full resource-integrity/unsafe-path/normalization/readiness suites remain required. |
| P6 fresh publication | Cached bytes survive old output modification/deletion; clones/correlation IDs return fresh paths and current warnings/layouts. Partial hit-write cleanup, failed atomic rename and artifact metadata failure preserve established errors/retention. Native ranges retain encoded bytes and monotonic completion. |
| P7 media/lifecycle | Direct native frames/ranges use unchanged hash-verified frozen image/audio references (SSIM>=.99, aligned PCM RMS<=.0001, one-frame timing); source/compiled MCP use independent literal solid color plus frozen audio/timing, actual worker reuse, revision/undo/redo/reopen and atomic failure snapshots. Separate stereo normalization fixture independently meters complete-root precodec loudness/peak and verifies cold/warm full ranges/crops. Exports execute uncached. Original workspace/native/contract/lifecycle regressions are required without changed oracles or thresholds. |

## Coherence and compatibility

Private siblings remain inside artifact/process owners under ADR0003; renderer consumes owner-produced bindings and existing publication/preflight behavior. No top-level owner or new dependency edge is introduced. Artifact I/O's optional bounded read defaults to unsupported; custom adapters preserve uncached behavior. Renderer clones share cache; adapter/font-root reconfiguration resets it. Normal public method signatures, schema44, protocol1, contracts/catalogs/provider/speech-worker files remain unchanged. Test-only counters use the existing `raster-cache-test-hooks` feature and stderr, never normal protocol results.

The32-entry/64MiB bound covers accounted retained keys/payloads. It is not a bound on allocator bookkeeping, independent raster cache, materialized workspaces or concurrent in-flight bytes. Native backend fingerprinting admits direct FFmpeg-family executable content/canonical paths; unknown/ambiguous/script/forwarding bindings bypass. Legacy unresolved implicit font fallback bypasses; verified shaped faces and configured file bytes contribute identity. Hits still repeat materialization/raster and normalization preparation as applicable; measured savings concern ordinary final native execution only.

## Passing evidence recorded so far

- Final focused cache suite12 tests: `/tmp/opencut-issue72-cache-final.log` passes, including changed valid font bytes, independent AV/timing/normalization and actual oversized bypass.
- Final strict workspace Clippy/format: `/tmp/opencut-issue72-clippy-final-v3.log`, `/tmp/opencut-issue72-fmt-final-v3.log` pass.
- Full native core unit suite699pass/9 intentional helper-maintenance ignores: `/tmp/opencut-issue72-workspace-final-v3.log`; all remaining workspace targets pass too:72 result suites,1491 passes,0 failures,9 intentional helper/maintenance ignores.
- Original normalization regression: `/tmp/opencut-issue72-original-normalization.log` (unchanged16-case independent suite passes).
- New full-root/cropped warm normalization: `/tmp/opencut-issue72-normalization-v12.log` passes.
- Initial native frozen AV parity: `/tmp/opencut-issue72-preview-golden-v3.log` passes.
- Instrumented persistent Rust worker13 tests: `/tmp/opencut-issue72-hooks-worker-final.log` passes on the final implementation.
- Actual source/compiled MCP plus original raster worker3 native tests: `/tmp/opencut-issue72-bridge-hooks-final.log` passes on the final implementation.
- Formatting, TS typecheck and lint202 files: `/tmp/opencut-issue72-fmt-final.log`, `/tmp/opencut-issue72-typecheck-final.log`, `/tmp/opencut-issue72-lint-final.log` pass (final Rust source edits require refreshed formatting).
- Hermetic Python12 unittest/12 pytest: `/tmp/opencut-issue72-python-final.log` passes.
- Protected prearchive gate: `/tmp/opencut-issue72-protected-prearchive-v3.log` passes policy tests and all56 strict spec items, rejecting only the expected active72 inventory. This is not merge readiness; postarchive must pass.

## Failed runs and corrections retained

All logs remain uncommitted under `/tmp`. Initial focused v1 exposed missing warm generic readiness; eligible requests now check before lookup. Early fake adapter omitted range preparation; it now exercises generated first/last frames. Glyph-limit fixture initially duplicated stackOrder0; corrected to the actual second-item position1. Normalization recorder initially mixed a historical mono/animated fixture with its extra precodec analysis; FFmpeg asserted in that test-only analysis. Matching its existing current-schema/default-bus stereo3000ms fixture passes unchanged independent loudness/peak/root-crop assertions; separate frozen AV coverage is retained. One accidental native run used incorrectly named environment variables and skipped native work; it is not acceptance evidence. Strict Clippy requested routine conditional/return cleanup, without allowances.

Initial broad bridge native execution exposed opaque forwarding executables being admitted as backend identity; they now bypass optional caching rather than claiming the delegated implementation was verified. Their existing intermediate-capture oracles are unchanged; final original native MCP coverage passes all8 cases. A concurrent ordinary-unit run exceeded original short contract/recipe/lifetime timeouts during heavy Rust compilation; it remains failed evidence; final unit743pass and source32pass retries keep unchanged timeouts. No tests, golden inputs, thresholds or required selections were weakened. A trial extra native selection in the exact closed CI command block was rejected by its policy (`/tmp/opencut-issue72-protected-prearchive-v2.log`,107 fixture-policy failures); that addition was fully reverted, leaving the workflow and validator unchanged. The final protected prearchive rerun rejects only active72 inventory. The new source/compiled cache tests are selected by the existing native worker command. Direct cache native evidence is additionally run locally, with explicit backend/font settings.

The first native workspace command omitted required rules-screen resolution/scope and failed closed; its superseded run and v2 duplicate were stopped, and final v3 supplies documented PR/960x540 settings while the separate required release matrix retains all three resolutions. A superseded pre-backend-correction release build was interrupted; the final report-bearing release command remains required. An accidentally opt-in native contract run duplicated the same workspace targets; it was interrupted, and the exact CI-default contract command remains required. Timing-sensitive ordinary unit/source MCP runs under simultaneous heavy compilation remain failed logs (`unit-final`,v2,v3; `integration-final`, including worker exit after timeouts); final unit743/source32 retries pass with unchanged limits. Seven of eight original native MCP cases pass in `/tmp/opencut-issue72-bridge-native-final.log`; blend-mode exceeded its unchanged60s limit and passes the final quiet retry without altering the limit. Native forwarding capture oracles now pass without alteration for the other cases.

## Final local acceptance

Every entry below completed with exit0 on the final implementation; no implementation files changed after these runs:

| Check | Evidence |
| --- | --- |
| Formatting and workspace strict Clippy | `/tmp/opencut-issue72-fmt-final-v3.log`, `/tmp/opencut-issue72-clippy-final-v3.log` |
| Full native-enabled workspace72 suites/1491pass/0fail/9 deliberate ignores | `/tmp/opencut-issue72-workspace-final-v3.log`; includes unchanged architecture inventory, desktop40, core699, all native integrations and headless lifecycle/alignment/marker behavior |
| Focused native cache12, actual instrumented worker13 | `/tmp/opencut-issue72-cache-final.log`, `/tmp/opencut-issue72-hooks-worker-final.log` |
| Actual source/compiled cache plus original raster worker3 | `/tmp/opencut-issue72-bridge-hooks-final.log` |
| Original native MCP8 | Seven pass in `/tmp/opencut-issue72-bridge-native-final.log`; the sole blend timeout passes its unchanged60s limit in `/tmp/opencut-issue72-blend-retry-final.log` with normal release sidecar |
| Normal debug/release restoration and full normal release headless83/0 ignored | `/tmp/opencut-issue72-default-debug-restore.log`, `/tmp/opencut-issue72-default-release-restore.log`, `/tmp/opencut-issue72-headless-release-final.log` |
| Original frozen release golden/performance capture and independent report validation | `/tmp/opencut-issue72-golden-report-final.log`, `/tmp/opencut-issue72-golden-performance-final.json`, `/tmp/opencut-issue72-performance-final.log` |
| Unchanged original PR rules matrix960x540/1280x720/1920x1080 | `/tmp/opencut-issue72-rules-960x540-final.log`, `/tmp/opencut-issue72-rules-1280x720-final.log`, `/tmp/opencut-issue72-rules-1920x1080-final.log` |
| TS typecheck/lint202 files/full unit743pass | `/tmp/opencut-issue72-typecheck-final.log`, `/tmp/opencut-issue72-lint-final.log`, `/tmp/opencut-issue72-unit-final-v4.log`;11 native opt-ins run separately as actual11 native MCP cases, rather than claiming ordinary skips as native proof |
| Source MCP32 and packaged29 | `/tmp/opencut-issue72-integration-diagnostic.log`, `/tmp/opencut-issue72-package-final.log`; actual normal release sidecar override uses the existing supported test option; unchanged limits/oracles |
| Cross-language contract command | `/tmp/opencut-issue72-contracts-final-v2.log`:32 Rust result suites458pass plus594 TS cases; its normal CI environment intentionally does not substitute for the explicitly configured native suites above |
| Hermetic Python12 unittest/12 pytest | `/tmp/opencut-issue72-python-final.log` |

The earlier source worker exits did not reproduce in the final32-case run. A temporary, uncommitted Node child-exit observer recorded only lifecycle exit information (no arguments/environment/credentials); it changed no assertions, timeouts, selection, application code or heap setting. The passing trace shows bridge exit0 and intentional worker SIGTERM after successful teardown. Earlier crashes remain unexplained transient runner failures, not hidden or asserted to have a proven OOM cause. No automatic acceptance is claimed for those earlier incomplete runs.

The final completeness/correctness/coherence review traces every normative scenario in the table above. Private ownership, conservative dependency identity, preflight-before-lookup, immutable/fresh publication, error cleanup, bounded retention, real final-execution savings and normal contract byte identity conform to the approved scope. There are no implementation-critical warnings. The external draft-PR/live-CI acceptance task remains explicitly incomplete; user approval already covers verification followed by synchronization/archive before publication. Archival therefore covers locally verified behavior, not a claim of remote CI or independent human review.

## Remaining external publication


Verified behavior is synchronized into `openspec/specs/preview-artifact-caching/spec.md` and archived here. Protected postarchive `/tmp/opencut-issue72-protected-postarchive.log` and pinned strict-all `/tmp/opencut-issue72-spec-postarchive.log` pass56/56. All503 protected policy tests pass. Commit/push and exact committed-head protected validation follow; draft creation and remote exact-head CI remain unconfirmed while the API is denied.

GitHub API remains denied: runtime revision7 still excludes api.github.com. Git pushes are available. Draft creation and exact-head live CI cannot be claimed until permitted API access is active. This report is agent conformance review, not independent human/CODEOWNER approval.
