# Motion audit repair verification

Base: main `4b0485d2172cbc7614d7674994d80dcb405fd6f9`, including merged #165 and #166. The repair inherits preview caching; it does not introduce that implementation. No public contracts, schemas, catalogs, dependencies, render thresholds, goldens or existing workflow assertions changed.

## Requirement/scenario traceability

| Requirement | Implementation | Independent conformance |
|---|---|---|
| Isolated inference worker recovery: cancellation/timeout, queued/immediate | `src/transcription.ts`: generation-owned buffer/pending map, retirement barrier, TERM/KILL cleanup, guarded late callbacks | `known-text-alignment.test.ts`: eight combinations of active alignment/transcription × cancellation/timeout × queued/uncontended immediate retry; dispatch observed before cancellation; typed failure and healthy output/FIFO assertions |
| Graceful direct disposal | provider close retains drain semantics and reaps child; documented in `docs/agent-bridge.md` | active finite inference completes, new work is unavailable; real production bridge SIGTERM during 60s fixture inference reaps bridge/worker within 3s instead of the 10s deadline |
| Required integrated native narration gate | configured workflow and exact policy command include existing oracle | omission and `|| true` negatives; actual independent native oracle passed without changing references/negative controls |
| Complete bounded MCP conformance validation | test-only SDK interpreter, all errors, exact-schema memoization in source/package clients | nested union/format/additional-property/array/scalar negatives compared with original AJV; unchanged complete source 33/package 30 workflows pass |

## Profiling evidence and correction

The largest catalog-expanded draft output is 762,279 serialized bytes. A standalone default SDK AJV compile was OS-killed (137), with no bridge, renderer or Vitest process. Disabling its compile optimizer compiled in 2,047.95ms with RSS 506,220,544 but did not fix the complete suite: subsequent client execution reached RSS 6.9GB with 323MB used JS heap before loss. That attempted mitigation is not shipped. The SDK-bundled interpreter compiled the same schema in 41.75ms with RSS 50,839,552 and rejected an empty result (RSS 51,101,696 after 44.49ms), without any product process. It avoids generated-code allocation, preserves published schemas and reports all validation errors; source/package suites now pass without skipped workflows or increased limits.

This establishes a client generated-validator allocation cause; it does not establish a production bridge/core/model leak. External clients choosing the default SDK AJV engine may still encounter the same runtime-specific upstream behavior. This bounded change corrects repository conformance clients, not third-party clients. The SDK documents a draft-07 dependency-reference interpreter gap; the current governed output catalog is 2020-12 and does not use that construct. No general equivalence for arbitrary future schemas is claimed.

## Completed checks

- Regression-before: two new recovery cases failed on unchanged main, including dying-worker reuse/EPIPE. Production correction followed the failing evidence.
- Final focused provider/validator/shutdown: 33 passes across 3 suites, 5.57s.
- Complete bridge unit: 754 passes, 11 existing optional/native skips across 8 files, 144.52s; no new skip. The independent native oracle below is separate evidence.
- Complete supported source command `bun run test:integration`:33/33,61.82s (includes actual debug headless build). Earlier corrected direct Vitest run also passed33/33.
- Complete supported `bun run test:smoke`:30/30,34.37s, after actual release/compiled runtime assembly and verification.
- Actual configured native integrated narration:1/1,121.80s; FFmpeg/FFprobe/font configured and `OPENCUT_GOLDEN_REQUIRED=1`.
- `cargo clippy --workspace --all-targets -- -D warnings`:pass, including desktop targets.
- `cargo fmt --check --all`:pass.
- TypeScript typecheck/lint:pass after correcting the shutdown test's generic-stream typing; lint fixes affected only repair files.
- Hermetic faster-whisper tests:12 passes on Python3.12.14 with pinned pytest8.4.1; no model inference or Python3.11 claim.
- CI policy:505 passes across4 suites; dedicated workflow suite482 passes including both new omission/masking negatives.
- Pinned strict all-spec validation:59 items pass.
- Pre-archive Moon protected task: all505 tests and strict59-item validation pass; final policy rejection names only this active change, as required by governance. No attestation is claimed before archival.
- Full `cargo test --workspace` completed through terminal doc tests:1,490 passes across64 unfiltered result blocks, no failures, nine intentional ignores, including43 desktop tests,701 core library tests and64 headless protocol tests. Nested filtered child-run results are excluded from that aggregate. Unconfigured native early-return bodies in this broad run are not counted as native rendering evidence.

## Host setup failures and limits

Original audit child-process lifetime failures were caused by unreaped zombies under PID1. Unchanged bridge suites run under a local `PR_SET_CHILD_SUBREAPER` harness; no assertions/timeouts change. During repair verification `/tmp` filled; disposable Rust outputs were moved to the larger workspace and old paths retained by symlinks. A workspace link attempt then lacked unversioned XCB/XKB development names; local symlinks to installed runtime libraries corrected linking without product edits. Both failed logs are retained and do not count as passes.

Moon initially failed on read-only home caches/offline tool setup. Its supported `MOON_TOOLCHAIN_FORCE_GLOBALS=true` mode uses preinstalled pinned Moon2.3.3/Bun1.4.0/Rust1.97.0 with writable task-local cache homes; no task or policy was skipped. A direct policy test run without those cache settings failed its real-Moon attack fixture; the correctly configured run passed505/505. An environment restart recovered with files and running checks intact.

Evidence is Linux-only. No GUI playback, real-model alignment accuracy, real TTS/audio listening, Windows/macOS execution or exact repair-head remote CI success is claimed. Broad unchanged optional visual-native matrices remain CI obligations. Local full logs stay outside the repository; no credentials/media content were published.

## Verification and delivery status

Implementation coverage/design are coherent for all four requirements. All 11 tasks, four requirements and eight scenarios are covered; no critical conformance/design mismatch remains. All affected TypeScript gates passed after the final timing-preservation refinement. The pre-archive protected rejection is exclusively the expected active-change inventory. The four accepted requirements were synchronized into the living specs and this change was archived. Final `moon run root:openspec-validate` passed: 505 policy tests, strict validation of all 58 living specs, and CI parity policy. The pre-archive count of 59 included the then-active change. Draft publication follows the local verification; remote repair-head CI remains unverified here. No merge or deployment is authorized.

The final conformance review removed an unnecessary await when no retirement is pending, preserving warm-worker dispatch and one-microtask graceful-close timing. The direct-close regression now asserts dispatch without a polling delay. All affected bridge gates are rerun on that final behavior; unchanged Rust inputs retain the passing full workspace/native evidence.
