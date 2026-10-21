# Integration verification

Approval and immutable baselines are recorded in approval.md. Integration is isolated from the clean published PR229 tree at d70120792511f50f99c363206620211787999638. Source parent is 986728a2b8f3d27a790713baf245db347d94feb0; merged main is 8cb21a6bab2f5a0134ab6af3f91a018f0ae9bde8. No merge or deployment is authorized.

## Requirement and scenario traceability

| Approved scenario | Implementation and automated coverage |
| --- | --- |
| Classify a validated native rejection for both callers | src/headless.ts composes confirmedCoreError(reportedCoreError(event.error)) after event validation; tests/durable-main47-witness.test.ts recognizes the same object through both witnesses and checks preserved fields. Existing durable-production-core-outcome and narration settlement suites retain their own rules. |
| Keep unconfirmed effects uncertain | New witness test rejects local lookalikes and malformed responses; unchanged durable-production-settlement/service/store and narration-blocks workflow/CLI tests exercise missing acknowledgements, process loss, abort, retention and quarantine. No production service, replay, eligibility or persistent renderer change relative to original228. |
| Verify independent predecessor recovery | tests/fixtures/durable-main47-pins.json freezes independently fetched main47 and original228 catalog SHA256 before consumer edits. tests/durable-main47.test.ts proves all three full pretty/compact predecessors, idempotence, unrelated drift and exact large-number lexeme retention, plus missing/duplicate/narrowed negatives. Explicit main47, original228, pure narration/package, planning and durable adapters retain all historical captures and pins. Existing governance coexistence tests prove both inverse orders and malformed ownership rejection. |
| Run both workflows on current main contracts | Required source/assembled smoke invokes unchanged durable-production-workflow.ts with real frame/range/review/export, retained outcomes and crash recovery. Main narration/package workflow tests and native Rust contracts remain required. Native hero runs preserve the historical78-tool oracle, filtering exactly fixture-owned additions. Results pending below. |

## Semantic conflict review

- headless.ts preserves main's validated narration witness and durable's confirmed native receipt on one BridgeError. Both canonical classifiers remain unchanged. All native/core source files are byte-identical to main.
- contracts.test.ts retains both direct governance callbacks, main47 schema/operations, and actual live MCP schema registration checks.
- narration-profiles-projection.ts composes explicit pure main-peer inverses before unchanged source-planning, durable and NP1 inverses. It preserves main marker47-to46 and NP1 marker46-to45 recovery without recapturing historical bytes.
- short-candidates-contract.ts preserves live exact ownership assertions and removes only reviewed main peers and durable additions before the frozen original source-planning proof. Its owned projection remains a pure no-op on absent additions.
- Main historical proof consumers normalize the exact reviewed durable additions and two source-planning consumer paths through a dedicated adapter. The main-only and durable-only independently frozen whole catalog hashes remain authoritative. Newly registered integration consumer metadata is rolled back only by an exact guarded original228 view, never by broad reserialization.

## Recorded attempts

- Frozen bridge dependency install passed. An initial repo-root install failed because that directory has no package.json; corrected bridge invocation passed.
- Initial typecheck passed. Later helper narrowing produced TS2352; corrected checked-record assignment passed typecheck again. Lint passed before the final small helper formatting pass.
- Initial focused run:84 pass,3 fail. Two adapter defects (no-op identity and reviewed source-planning metadata variants) were corrected. The unchanged process lifetime test encountered PID1 zombie retention; rerun using the existing task-owned subreaper passed. Focused rerun:87/87 passed in5 files.
- First full unit run:1084 pass,57 fail,15 optional skips. Both failing files required a native headless binary absent from the isolated worktree (ENOENT). No test assertion or timeout was changed. Build and rerun pending.
- cargo fmt --all -- --check passed. Strict Clippy, workspace tests and remaining required gates pending.

Full local logs are retained uncommitted under /tmp/opencut-pr228-*.log, including every failed attempt. A pass is recorded only after terminal completion. Exact-head CI follows publication and remains separate from local verification and CODEOWNER acceptance.

- Strict workspace Clippy passed (unmodified strict flags). First workspace test compilation failed with ENOSPC/linker bus errors before terminal test success; full log retained. Removed only completed failed-run fixture directories and disposable Cargo development cache, preserving source trees, logs and separately copied PR229 verified binaries. Retrying unchanged workspace suite with CARGO_BUILD_JOBS=1; result pending.

## Terminal native workspace evidence

`CARGO_BUILD_JOBS=1 cargo test --workspace` completed with exit0:1661 passed,0 failed,14 existing ignored across84 reported harness summaries (including zero-test harnesses). The conditional `cargo build -p opencut-headless` then completed with exit0. Logs: /tmp/opencut-pr228-workspace2.log and /tmp/opencut-pr228-build.log. Prior ENOSPC attempt remains in /tmp/opencut-pr228-workspace.log.

After all target-using processes exited, relocated only the disposable Cargo target from the constrained /tmp volume to /workspace/opencut-task-target. /tmp/opencut-target remains a symlink, preserving configured references. SHA256 verification of the built headless executable before/after relocation passed. /tmp recovered5.2GiB; /workspace retains25GiB free. PR229 source tree, separately preserved verified binaries/manifest and all logs remain intact. Required explicit desktop-native and subsequent suites are running sequentially; no additional terminal pass is inferred here.

- Explicit configured desktop review native acceptance passed:1 passed,0 failed,0 ignored (59 filtered), using actual /usr/bin/ffmpeg and /usr/bin/ffprobe. Final TypeScript typecheck and lint passed (343 files checked). Logs: /tmp/opencut-pr228-desktop-native.log, /tmp/opencut-pr228-typecheck-final.log, /tmp/opencut-pr228-lint-final.log. Full unit rerun against the newly built native binary is active; earlier ENOENT failures are not counted as resolved until terminal rerun success.

- Terminal full unit rerun passed:1141 passed,0 failed,15 optional skipped (90 passed files,10 optional skipped files). This actual success against the built native binary resolves the first run's57 ENOENT cases; the failed log remains retained. Includes the new13 full-predecessor/negative tests and combined-witness regression, all unchanged narration/durable settlement, and strict historical/governance controls. /tmp/opencut-pr228-unit2.log. Required unmodified contracts:check is now running sequentially.

- Required `bun run contracts:check` completed with exit0: native 529 passed,0 failed,0 ignored; TypeScript706 passed,0 failed/ignored in46 files. /tmp/opencut-pr228-contracts.log. Source integration now active in the same sequential session.

- Required source MCP integration passed44/44,0 skipped; required assembled packaged smoke passed41/41,0 skipped. The latter builds a genuine release headless binary, compiles the Bun bridge, assembles/verifies the runtime package and executes through the packaged paths. Both include unchanged durable-production-workflow four-producer and crash acceptance plus main narration/package workflows. Logs: /tmp/opencut-pr228-integration.log and /tmp/opencut-pr228-smoke.log. Every step in /tmp/opencut-pr228-check-results.log completed with exit0. Required actual native hero debug run now active with OPENCUT_GOLDEN_REQUIRED=1 and real configured FFmpeg/FFprobe; no native hero pass inferred from optional default skips.

- Required actual native hero debug run passed2/2 with0 skipped in140.55s, using OPENCUT_GOLDEN_REQUIRED=1, /tmp/opencut-target/debug/opencut-headless and actual configured FFmpeg/FFprobe. Assertions, timeouts, legacy78-tool oracle and native output checks were unchanged. /tmp/opencut-pr228-native-hero-debug.log. No optimized substitute was needed for this current integration; assembled release acceptance is separately green. This local result does not establish Windows readiness or exact-head remote CI success.

## Remaining implementation gates and conformance

Hermetic Kokoro unittest12/12 and Faster Whisper pytest12/12 passed. Root policy627/627 passed. Strict all-spec/change validation80/80 passed. Prearchive protected Moon check exited1 solely because integrate-durable-requests-with-main47 remained active; its underlying strict validation80/80 passed and no other rejection was reported. This expected rejection is not counted as protected gate success. Logs: /tmp/opencut-pr228-python-tts.log, python-whisper.log, policy.log, strict.log and prearchive.log.

All8 approved implementation tasks are complete. Scenario mapping above covers both requirements and all4 scenarios, with real source/assembled/crash/native acceptance and independent predecessor negatives. No implementation/design mismatch, missing scenario or incomplete required local implementation gate remains. Verification found0 critical issues and0 warnings. The approved proposal/design retain their original preapproval wording and SHA256 as immutable review snapshots; approval.md records the subsequent explicit approval and this report records implementation outcomes. The final protected gate and exact-head CI remain separate post-archive/publication obligations.

Historical raw captures/pins originally present on228 were not edited. The narration/channel predecessor captures newly inherited from main are byte-identical to main. All core/headless files are byte-identical to main; original durable production service/store/prepare/classifier and persistent renderer are unchanged relative to source228. No Windows disposal diagnostics/behavior, workflows, assertions, timeout or unrelated branch was changed. No libmpv display/playback readiness or Windows readiness is claimed by these CLI results.

## Archive and final protected evidence

Synced exactly the2 approved requirements/4 scenarios into the living durable-production-requests spec and archived only this completed change on2026-10-21. OpenSpec apply reported all_done with8/8 complete tasks. Post-archive protected `moon run root:openspec-validate` passed with its unchanged policy attestation and79/79 living specs; separate pinned strict all-spec validation passed79/79. Logs: /tmp/opencut-pr228-final-protected.log and /tmp/opencut-pr228-final-strict.log.

Fresh Git fetch before publication confirms main remains8cb21a6bab2f5a0134ab6af3f91a018f0ae9bde8 and existing feature branch remains986728a2b8f3d27a790713baf245db347d94feb0. Connector metadata confirms the same head repository/branch/author and draft=true, open/unmerged. The integration will be published as a merge descendant of both refreshed refs, without force. Removed the untracked repository target symlink before staging; external disposable build cache remains available. No test output, cache or binary is committed.

Local implementation verification and protected gates are complete. Remote exact-head platform checks necessarily follow publication and remain pending until terminal; CODEOWNER acceptance is separate. No merge, deployment, issue closure, Windows readiness or global full-validation claim is made. Previous source-branch Windows FFmpeg-install failures are historical failing evidence, not successful checks for this integration. Every failed local attempt remains recorded above and in uncommitted logs.
