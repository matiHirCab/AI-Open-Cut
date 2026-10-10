# Default audio timeline placement: conformance and evidence

Base: main16b41e07476b18cb15f4631be6229b2a4fafcfb8, including the cleanup repair. Explicit reviewer approval is recorded in approval.md. This change repairs an independently reproduced P2 in the canonical core render planner; it does not change public/persisted contracts, catalogs, providers, workflow policy or cleanup ownership.

## Independent before/after

The exact-main historical witness uses a500ms440Hz/48kHz WAV at2000ms. On unchanged main64335ba1 (whose audio-owner blobs equal16b41), actual H264/AAC default export had RMS0.0881614 at0.1–0.3s and0 at2.1–2.3s. Nonidentity DSP already placed the signal correctly. The original exact-main exports, PCM, typed request/result ledger and hashes remain preserved outside this change; they were not regenerated.

The same independent public-headless harness now yields default RMS0 at0.1–0.3s and0.0882978515 at2.1–2.3s. GainDb=-1 yields0 and0.0787162719; its MP4 hash is unchanged from the historical correctly placed DSP witness. Repaired default MP4 SHA256:1e43d7c390ce2fc8e7c94b7b936ca0c975266186e6e7a35103bdd2e870f9fa51. Tested default release binary SHA256:36c50983563bd5a075ba762f5cfee925d2d3345c6cc97aa38fec956e79f71527. These are local artifacts, not claims about a future remote binary.

Independent harness log:/tmp/opencut-audio-fixed-independent.log. Evidence:/workspace/opencut-epic9-repair-build/tmp/opencut-audio-assessment-cl3ie_v3 (typed ledger, measurements, exports, decoded float PCM, ffprobe records). Historical main evidence:/workspace/opencut-epic9-repair-build/tmp/opencut-audio-assessment-c3id2hso.

## Acceptance-to-evidence matrix

| Approved acceptance | Implementation and demonstrated evidence |
| --- | --- |
| 1. Delayed ordinary audio across absent/identity/active DSP | render_plan.rs physically pads nonzero ordinary starts with existing adelay before sequential mixing. New ordinary_audio_timing.rs uses literal authored clocks and independently synthesized440/880Hz WAV witnesses. Actual H264/AAC plus decoded48k float PCM asserts leading silence and expected tone. Independent before/after above corroborates the test. |
| 2. Gaps, overlaps, consecutive boundaries, trim and local controls | Six explicitly timed clips at0/500/600/1000/1200/2000ms test gaps, two simultaneous frequencies, source-in500ms, local fade and volume keyframes. Expected frequencies and intervals come from authored literals, not renderer graphs. Five processing/routing modes are exercised. |
| 3. Preview crops and mapped clocks | Each of ten layout/mode combinations renders export plus audiovisual previews starting0ms,900ms in a gap and2050ms inside a clip. Independent content windows are asserted; aligned full-interval preview/export RMS remains<=0.0001. Existing inherited-timing native conformance and native lifecycle offset/scaled/nested/repeated controls passed, preserving mapped clocks without double delay. |
| 4. Roles, routes, neutral/master DSP and ducking | Default role route, explicit voiceover route, master gain and unreachable SFX gain are tested. Neutral identity/unreachable output RMS remains<=0.0001 against absent DSP; intended gain ratio is checked. Three extra absent/identity/active exports independently assert globally timed role ducking and narration onset. Existing native bus-ducking, nested/master DSP, compression, balance/EQ, normalization, analysis and narration suites passed. |
| 5. Immutability, typed failures, cancellation, retry and history | Mandatory platform source/default-package helper produces seven real exports each: absent, identity, gain, moved, undone, redone, reopened. Every render preserves exact state; fresh MCP reopen verifies persisted history. Existing native encoder-failure lifecycle checks preserve destination/state and clean workspace; native headless lifecycle passed. Unit/contracts/integration/smoke retain cancellation/process-tree/preview-debt/retry/stale/missing controls. Missing adelay now returns existing DEPENDENCY_UNAVAILABLE before publication with edits/history available; five platform fallback controls passed in each mode. |
| 6. Contracts, cleanup and historical witnesses | Only planner placement and base filter inventory change executable production behavior. Existing instance/retained branches and cleanup owners are untouched. Public/schema/catalog/provider and workflow diffs are empty. Verify-only native golden conformance passed with the exact reviewed font and unchanged historical references; no replacement or tolerance change was needed. |
| 7. Required checks and protected delivery | See exact local ledger below. Strict/prearchive policy checks precede conformance verification and synchronization/archive. Final protected/spec checks and all exact-published-head CI remain mandatory external post-archive delivery acceptance; this record does not prematurely assert remote completion. |

The new PCM helper is called by the existing mandatory native DSP gate, and the transport helper by both existing mandatory source/default-package platform cases. Ordinary opt-in skips cannot replace those genuine native runs.

## Local execution ledger

Commands run from repository root unless noted. Full logs:/tmp/opencut-audio-*.log; companion .exit files contain exact status. Local environment: pinned Rust1.97/Bun1.4/Moon2.3.3, FFmpeg/FFprobe7.1.5, temporary native link libraries/tool homes and dedicated workspace build/TMPDIR. No repository dependencies/configuration or test deadlines were changed. A temporary process subreaper supplies container orphan reaping without modifying assertions.

- cargo fmt --check --all:exit0 (fmt-complete).
- cargo clippy --workspace --all-targets -- -D warnings:exit0 (clippy-complete).
- cargo test --workspace:exit0, full workspace and doc tests (workspace). Production inputs remained stable throughout; the later additional test-only global-ducking cases also passed in the final mandatory native DSP run.
- Bridge bun run typecheck and bun run lint:exit0 (typecheck-complete/lint-complete).
- Bridge bun run test:unit --maxWorkers=2 under the temporary subreaper:exit0,869passed/15 configured skips (unit-isolated).
- Bridge bun run contracts:check --maxWorkers=2 under subreaper:exit0, Rust prerequisites plus607 TypeScript cases/38files (contracts-isolated).
- Bridge bun run test:integration under subreaper:exit0,36cases (integration-complete).
- Bridge bun run test:smoke under subreaper:exit0,33cases (smoke-complete).
- Python Kokoro unittest discovery with existing hermetic venv:exit0,12cases (python-unittest-final). Faster-whisper pytest:exit0,12cases (python-pytest-final). Platform setup:exit0,5cases; motion release-backend setup:exit0,7cases (python-platform/python-release-final).
- Existing exact native DSP gate:exit0 (native-dsp-complete), includes43 new actual media outputs plus existing DSP witnesses. Native inherited clocks:exit0 (inherited-clocks). Native bus ducking:exit0,2cases (ducking-complete); normalization:exit0 (normalization-complete); audio analysis:exit0 (analysis-complete); narration fixture:exit0 (narration-complete).
- Native animation lifecycle:exit0,3cases including encoder failure (lifecycle-complete). Native headless edit/undo/redo/reopen/draft lifecycle:exit0 (headless-lifecycle-complete).
- Verify-only release core-lib native_golden_render_conformance with reviewed tests/fixtures/fonts/DejaVuSans.ttf:exit0, all six specialized suites plus existing capture/reference comparison (golden-reviewed-font). No update/capture-to env flag was used.
- Genuine Linux source/default-package platform runner under subreaper:exit0,2/2 (platform-final). Evidence:/workspace/opencut-epic9-repair-build/audio-platform-evidence includes14 new actual MP4s, RMS/job ledgers, manifests and existing frame/media/cleanup/fallback controls. Source and package absent/identity RMS0 before onset and0.0883100583 at2.1–2.3s; active gain RMS0.0787337024. Move/undo/redo/reopen maintain expected1s/2s placements.
- Strict all-spec validation:exit0,64/64 (specs-complete-prearchive). Prearchive protected Moon:exit1 solely because this approved change is active (protected-complete-prearchive); all specs and other policy checks pass. This expected prearchive rejection is not a final protected success.

## Retained failed attempts and limits

Initial full unit/contract attempts competed with heavy native compilation, producing existing5s catalog and120s desktop deadlines and process-reaping failures. Original logs remain (unit, unit-final, unit-serial, unit-timeout-recheck, unit-quiet, contracts). Final isolated unchanged-deadline runs passed; no failing assertion was weakened. One unfinished overly broad release golden build was interrupted and replaced by the exact canonical library test; workspace tests independently cover integration targets. Initial golden verification used the system font and failed its hash prerequisite before comparison; the exact reviewed font hash ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280 passed. Initial Python command/directory/import attempts were corrected to the existing hermetic environment; correct commands passed. One independent workspace test process was temporarily paused to serialize CPU-sensitive unit/contract checks and resumed; all owned paused processes were resumed. Full logs retain these attempts.

This evidence covers actual synthetic H264/AAC exports and actual default packaged MCP on Linux, not only graph equivalence or green CI. The exact Windows Brock69.7s/18-phrase production project/assets were unavailable and are not claimed tested. Genuine Windows/macOS source/package and release runs remain exact-head CI acceptance. Full production-scale speech, arbitrary codecs/hardware and long-duration memory/performance are not inferred from the bounded synthetic fixtures. Existing golden tolerances and package policies remain unchanged.

## Conformance status

Completeness: all nine implementation tasks and required local checks are complete; post-archive delivery acceptance remains explicitly external. Correctness: every changed requirement and scenario maps to owning code plus independent native or retained lifecycle tests. Coherence: physical placement precedes ordinary sequential mixing, local controls remain before delay, global ducking after delay, and existing mapped clocks/contracts/cleanup remain canonical. No production repair or witness update beyond the approved scope is present.

Conformance review with openspec-verify-change found zero CRITICAL issues: all requirements/scenarios have implementation and test evidence, all implementation tasks are complete, and scope/design match. Warnings: remote Windows/macOS and terminal exact-head CI are pending mandatory post-archive acceptance; the exact Brock project is unavailable. The verified implementation is ready for synchronization/archival. Final protected results and CI are recorded against the published head outside this commit to avoid a self-referential evidence commit.
