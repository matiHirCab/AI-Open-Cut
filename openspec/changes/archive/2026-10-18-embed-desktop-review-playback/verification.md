# Verification — implementation and final local gates passed

Issue #213 is isolated on `feat/issue-213-embedded-review`, now based on main `fa5b2a28c3e2ea3544e660cd077be1ed12ac91fe`. Local WIP was committed and rebased without conflicts after the user externally merged #225. No #208 files/device, speech/narration, shared wire catalogs, dependency manifests/lockfiles, security settings or deployments were changed. This report records successful required implementation checks and final isolated GUI verification before publication; exact published-head CI is tracked in the pull request.

## Approved scope and implementation review

Proposal, design, two requirements/five scenarios and tasks were approved under delegated maintainer authority before executable edits. The first backend is optional installed Linux libmpv 2; other platforms keep an explicit unsupported fallback. Rendering remains core-owned and playback accepts only captured range artifacts.

A separate read-only reviewer found and re-reviewed three corrected blockers: render-thread-unsafe synchronous client calls (replaced by command_async and typed property observations), atlas eviction omitting the active window (now propagated), and short audio failures escaping the one-second grace (Playing requires output readiness and short EOF without prior readiness fails). ABI signatures/event layout/pointer lifetime and render→client→library-reference destruction were checked against installed primary headers. A final follow-up approved shared controllable phase values for non-Linux builds and notification only when visible state/image changes. No unresolved implementation blockers were identified; the final isolated runtime refresh and all general implementation gates below now pass. Specification closure and exact published-head CI remain separate gates.

## Current required pipelines

Executable inputs are frozen in `/tmp/opencut-213-rebased-source.sha256`; logs are uncommitted under `/tmp`. No test executable may be retired while Cargo has a verification plan in flight.

| Check | Current evidence |
| --- | --- |
| Rust formatting | PASS `/tmp/opencut-213-rebased-format.log` |
| Strict workspace Clippy/all targets | PASS `/tmp/opencut-213-rebased-clippy.log` |
| Full workspace | PASS 1,588 cases across Cargo targets plus 9 subprocess replays (1,597 successful test executions), zero failures; `/tmp/opencut-213-rebased-workspace.log` |
| Fresh headless build | PASS `/tmp/opencut-213-rebased-headless-build.log` |
| TypeScript typecheck/lint | PASS `/tmp/opencut-213-rebased-typecheck.log`, `/tmp/opencut-213-rebased-lint.log` |
| TS units | PASS 959 tests, `/tmp/opencut-213-rebased-ts-unit.log` |
| Contracts | PASS Rust consumers and 645 TS tests, `/tmp/opencut-213-rebased-contracts.log` |
| Source integration / packaged smoke | PASS 43 / 40 tests, `/tmp/opencut-213-rebased-{integration,smoke}.log` |
| Explicit native playback/controller/desktop build | PASS five controller tests and one explicit native case (15.18s); `/tmp/opencut-213-rebased-{controller,native,desktop-build}.log` |
| Final desktop GUI | PASS 48 isolated clock/color samples, two wraps, maximum observed discrepancy 200ms, zero tolerance violations; Playing window close exit 0, no audio clients and authoritative hashes unchanged |
| Existing native desktop review | PASS one rebased exact native case in 86.29s, `/tmp/opencut-213-rebased-existing-native.log` |

Environment uses pinned Rust 1.97.0/Bun 1.4.0/Moon 2.3.3 and the unchanged reaper supervisor for container orphan cleanup. The temporary target symlink and locally materialized unchanged JS dependencies are tooling only, not deliverables. Hermetic Python inputs/interpreter/environment remain unchanged; #225's `/tmp/opencut-python-tests.log` records 12 unittest plus 12 pytest passes. Reuse is limited to those unchanged provider inputs, not affected Rust/native/GUI checks.

## Native and genuine GUI observations

The explicit final pre-rebase playback test passed one case in 11.40s (`/tmp/opencut-213-final-native.log`); source and compiled-binary checksums were recorded. It verifies original red/blue pixels, short-clip state boundaries, pause, exact seek, continued clock and decoded frame 500ms after loop within 200ms tolerance, three owned stop cycles, actual audio samples, a valid 400ms clip, failed audio on long/400ms clips without loop, absent files and corrupt MP4. Controller tests additionally cover admission capacity, seek clamp, captured identity, factory failure, worker panic and reap. The rebased explicit native case also passed in 15.18s, including real audio. Binary checksums are `/tmp/opencut-213-rebased-binaries.sha256`; final GUI assessment is described below.

Actual built GPUI ran on a task-local X11 display with Openbox and Vulkan software rendering. It rendered the original 320×180/10fps project through core at range 0–3000ms, captured revision 2, and then played the resulting MP4. Observations include paused red at 0, paused blue after seek to 2000ms, Playing blue at 1800ms from start, stable pause, and loop continuing at 600ms after crossing the end. An isolated realtime PulseAudio output monitor captured 7,532,916 bytes; the strongest one-second block had RMS 0.06246 and peak 880Hz. This is real backend output, not evidence of physical-speaker quality. Every authoritative project/history hash remained identical during playback/control-only operations.

Undo produced revision 3/duration 0 while the retained review still showed captured revision 2/0–3000ms, STALE, and its old blue frame. Redo/render produced a fresh revision-4 review and stopped the prior player. Removing only the latest disposable generated MP4 displayed the fixed missing-file message and required a new render. A separate process without libmpv in its loader path showed the compatible-backend error and retained external reveal. Closing an actual window while Playing at 300/1000ms with Loop on exited 0 and left no audio sink input.

Repeated actual GUI opens/closes returned to 21 threads after every acknowledged close. Five initial and ten further completed cycles were measured, followed by eight controlled full-scroll cycles. The last six controlled closed samples were about 314–319MiB RSS. Native codec/allocator caches retain warm memory: standalone software-libmpv probes reproduce the warm-up without GPUI. These observations do not claim zero retained process RSS or a hard native-runtime heap quota. GPUI does not expose a stable public atlas-allocation counter; correct active-window eviction was independently reviewed and exercised through these actual replacements/closes instead of asserting implementation text.

Full captures remain outside Git under `/tmp/opencut-213-gui-*.png`; three rebased performed-control screenshots are checked in under `docs/fixtures/embedded-review/`; the original fixture is reproducible via the checked-in example. Rebased GUI showed paused red, exact paused seek to blue at 2000ms, live red/blue and continuing loop. Paused process CPU used 0.40s over 10.00s (4% of one core). During concurrent required suites, live screenshots sometimes retained an old color while the clock advanced; 32 independent X11/color/OCR samples confirmed changing colors but intermittent presentation delay above 200ms. Direct FFmpeg probes of the generated MP4 independently locate the color transition at 1500ms. A lower-load repeat after bridge completion showed first blue at 1600ms and red at 100ms after loop; all sampled color-boundary differences were at most 100ms (32 samples in `/tmp/opencut-213-motion-lowload/evidence.json`). The separate reviewer found no new code blocker and agreed that this fits the 200ms verification tolerance; the final isolated refresh after workspace completion additionally passed 48 samples/two loop wraps with maximum observed color-boundary discrepancy 200ms and no violations (`/tmp/opencut-213-isolated-motion-summary.json`). Closing the actual window while Playing exited 0, left no audio client and preserved every authoritative project/history hash (`/tmp/opencut-213-isolated-authoritative-{before,after}.json`). This sampled original fixture is not a proof of latency under every load or source. Actual rebased GUI audio captured 1,935,252 bytes, strongest one-second RMS 0.062397 and peak 880Hz (`/tmp/opencut-213-rebased-gui-audio.json`). Heavy concurrent-load observations remain a scheduling/performance limitation; the native 200ms clock tolerance is not claimed as a GUI synchronization guarantee. Diagnostic logs are `/tmp/opencut-213-motion-probe/evidence.json` and `/tmp/opencut-213-generated-mp4-pixels.json`. A first OCR probe was stopped because default OpenMP threading stalled; the final completed diagnostic bounded OCR threading, with no product/test timeout changes.

## Preserved failures and incomplete evidence

- Initial compile found two missing PreviewReviewOptions defaults and an Arc move; a later example compile needed explicit BatchEditOperation typing. Fixed before passing desktop compilation/Clippy. Logs: `/tmp/opencut-213-focused.log`, `/tmp/opencut-213-clippy.log`.
- The missing-file test initially expected a worker error after admission changed to reject missing files earlier; its expectation was corrected. The audio-sample assertion originally followed an intentionally failed device that truncated the ALSA file; positive output is now checked before fault injection. These failed logs remain `/tmp/opencut-213-native-audio-fault.log` and `...-retry.log`.
- An unpaced ALSA file/null sink captured samples but froze the playback clock after loop; the strengthened progress test failed `0 -> 0`. Repeating against a realtime isolated output monitor passed. File-only capture is explicitly not accepted as timing evidence. Logs: `/tmp/opencut-213-native-loop-progress.log`, `/tmp/opencut-213-native-pulse.log`.
- Initial bare Xvfb mapped a black window; shared-memory capture failed. A complete virtual session with a window manager/Vulkan and direct X11 capture displayed the actual editor. No black screenshot counted as acceptance.
- A repeated-close harness stopped early because incomplete scrolling made its click hit Loop instead of Close. Full-scroll repetitions subsequently verified acknowledged closes; the partial attempt is not passing evidence.
- JS dependency-directory symlinks produced nine Biome type-inference false positives in a byte-identical unchanged fixture. Materializing the same dependency files locally passed all 287 linted files; no rule/source/timeout changed. Logs: `/tmp/opencut-213-final-lint.log`, `...-lint-local-deps.log`.
- An attempted cleanup retired five earlier test executables with different fresh counterparts while Cargo still planned to run one. Workspace then failed with missing `masked_hero_reveal_native` before execution, an agent tooling error rather than a source regression (`/tmp/opencut-213-final-workspace.log`). Missing outputs were regenerated. A subsequent attempt and canonical contracts were stopped solely to rebase onto newly merged main; their exit 241 logs are incomplete, not pass evidence. No further build-output deletion is permitted during the final pipelines.

## Scenario traceability

- Play/seek/pause/loop: bounded Player/controller and native observed-property worker; focused tests, actual native pixels/audio/clock and genuine GUI controls above.
- Captured identity/history: Player identity and unchanged Artifact metadata/stale logic; controller identity test, existing review/history tests, authoritative hashes and actual Undo/Redo/replacement.
- Truthful readiness/failure: fixed Fault messages, pre-admission metadata check, optional library/API validation, typed native EOF/command/property errors and audio readiness; injected factory/queue/panic tests, actual invalid MP4/short and long output faults plus GUI absent-file/backend cases. Unsupported-platform execution is not claimed; its fallback is compiled by required platform CI.
- Bounded lifecycle: channel capacity 8, one latest worker frame, max 1280×720 display, one displayed/pending image, generation check, owned disconnect/join/destruction and active-window eviction; automated controller/bounds/native stop tests plus independently reviewed GUI resource API and measured replacements/window close.
- Supported runtime/GUI: original fixture, realtime isolated output capture, independent numeric pixel/clock tests and actual screenshots; source capability, installed readiness and physical hardware limits stay separate.

## Remote baseline and delivery

#225 head `748267983ee65a8cd9080e57341044a6997b19f3` passed all four workflows, including correctness on Linux/Windows/macOS, native/render/contract/rules-screen parity, packaged smoke and protected foundation aggregate ([CI](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/39167706836)). It was merged externally as `fa5b2a28`; no agent merge/deployment occurred. Historical main Windows timeouts were not reproduced on that head and no timeout was weakened.

All rebased required implementation checks and the final isolated GUI refresh pass. Conformance is verified, only this change is synchronized/archived and final protected/strict validation passes. The change is ready to publish as a separate verified draft. Exact published-head CI remains mandatory afterward. #208 and unrelated owners remain untouched.

## OpenSpec conformance verification

| Dimension | Final prearchive assessment |
| --- | --- |
| Completeness | 10/10 tasks; two normative requirements and five scenarios implemented |
| Correctness | 2/2 requirements and 5/5 scenarios mapped above to controller/native tests and genuine GUI evidence |
| Coherence | Approved desktop ownership, optional installed backend, immutable captured identity, bounded native/presentation lifecycle and compatibility decisions followed |

The separate final reviewer independently confirmed two requirements/five scenarios covered and no material implementation blockers after the isolated GUI refresh. There are no unresolved critical findings, warnings or code/spec mismatches. The heavy-load timing, physical-audio, native allocator-cache and unsupported-platform evidence limits above remain explicit. No required affected implementation check was substituted by a readiness skip.

Pinned strict validation passed 76 items before archive. The prearchive Moon gate ran 521 policy tests (zero failures) and strict validation successfully, then exited 1 solely because `embed-desktop-review-playback` was active. The full log `/tmp/opencut-213-prearchive-protected.log` was inspected for additional failures; none were present. This expected rejection is not reported as gate success. Implementation conformance is verified and the change is ready for synchronization/archival. Only final postarchive protected/strict success authorizes publication.

## Final synchronization and protected gate

Only this change was archived to `2026-10-18-embed-desktop-review-playback`; its two added requirements/five scenarios were merged into the existing living `desktop-review-surfaces` specification without removing earlier requirements. All 10 implementation/verification tasks are complete. Postarchive `moon run root:openspec-validate` passed its 521 policy tests, strict validation of 75 living specifications and unchanged parity policy. Separate pinned `validate --all --strict --no-interactive` also passed all 75 items. Logs: `/tmp/opencut-213-final-protected.log`, `/tmp/opencut-213-final-strict.log`. No active change or other owner's archive was altered.

Final assessment: all required local implementation, runtime, conformance and postarchive gates pass. Publish only a draft; exact-head CI must finish successfully before claiming remote completion. No agent merge/deployment is authorized or performed.
