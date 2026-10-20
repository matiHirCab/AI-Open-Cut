# Verification of issue220

Status: implementation and independent production conformance approved; whole-workspace and exact-head publication gates remain pending. Do not treat this report as merge readiness until the terminal evidence below is completed.

## Scope and approval

Only #220 is implemented. Baseline main756b7b39ce0e2b1d5fcc65291084838da5331517. Independent specification approval and narrow amendments are recorded in approval.md. Independent reviewer inspected production core/CLI and approved conformance after reproducing and fixing three filesystem/input defects and a real subprocess exit defect. Reviewer authored only the independent verification harness, not production implementation. CODEOWNER acceptance is separate and pending. No workflow, package command, timeout, warning suppression or historical raw capture/digest was weakened or repinned.

## Requirement and scenario traceability

| Requirement | Implementation | Automated evidence |
| --- | --- | --- |
| Bounded deterministic artwork transformation | assets/artwork.rs, model/artwork.rs |12 exact raster cases: alpha, flood reachability, white interior, holes/fine line, padding, radius0, partial alpha/source-over, tolerance endpoints, palette/tRNS/sub8/16-bit, gamma ignored, corrupt/APNG/limits, maximum supported dimensions |
| Explicit inspect apply and discard | src/artwork/cli.ts, artwork-cli.ts |18 CLI cases: exact named files/hashes, explicit approval, all four altered files, foreign/missing/linked/FIFO files, escaped paths, existing destination, parent substitution, unsupported backend, idle cancellation, late completion, partial staging cleanup |
| Atomic source-preserving derivative commit | store/artwork.rs, existing persistence |11 core integration cases and2 private WAL fault cases: exact source/output bytes, one revision/asset, duplicate/stale/tampered intent, source deletion guard, undo/redo/reopen, before-journal rollback and after-journal recovery; missing probe facts preserve old operation behavior |
| Source-preserving artwork derivative ownership | assets.rs canonical reference/integrity/GC owner |core artwork cases cover current, history, draft transitive source roots, dangling/mismatch/self/two-node cycles, source/destination links, eventual history-eviction collection;23 architecture tests preserve dependency matrix |
| Artwork metadata schema migration | model.rs, migrations.rs |schema46 current/history migration is deterministic/idempotent; future and premature derivation preserve authoritative bytes; historical narration13 cases retain existing intent;91 headless cases include strict additive operations |
| Genuine source/operator acceptance | scripts/verify-artwork-preparation.py, source smoke registration |Actual Bun CLI+debug headless+FFmpeg/FFprobe with hand-built original PNG and literal independent RGBA/mask/light/dark oracles, hashes, preview byte preservation, commit/conflict/undo/redo/reopen/discard and real open-stdin SIGTERM exit |

## Passing focused checks

Full logs are local, uncommitted and outside the repository under /tmp/opencut-tools:

- `cargo test --locked -p opencut-editor-core --lib assets::artwork::tests`:12 passed, artwork-raster-tests-png181.log.
- `cargo test --locked -p opencut-editor-core --test artwork_preparation`:11 passed, core-artwork-probe-guidance.log.
- `cargo test --locked -p opencut-editor-core --test artwork_preparation --test architecture`:10 then-current artwork+23 architecture passed, core-artwork-final.log; later11-case suite adds missing-probe evidence.
- Private artwork WAL fault cases2 passed, core-artwork-faults.log; historical narration13 passed, core-artwork-tests.log.
- `cargo test --locked -p opencut-headless`:91 passed, core-artwork-headless.log.
- `bunx vitest run --config vitest.unit.config.ts tests/contracts.test.ts tests/narration-profiles.test.ts tests/artwork.test.ts`:67 passed, artwork-contract-final.log. Own additions are inverted precisely before retained predecessor chains; historical byte pins remain unchanged.
- Complete TypeScript run passed1012 tests,15 existing optional cases skipped across80 passing/10 optional-skipped files, artwork-full-unit-subreaper.log. Final governance-only patch then passed all85 affected tests (18CLI+67contracts/artwork/narration) across4files, artwork-cli-canonical-focused.log; actual serialized envelopes match canonical examples. No old case removed. Latest release docs18 passed, artwork-release-doc-latest.log. Release markers18 passed, /tmp/opencut-artwork-release-markers.log.
- Actual checked-in native acceptance passed again on final entry lifecycle fix, artwork-checked-in-acceptance-final.log; retained synthetic project/evidence path is in that log. Actual open-stdin SIGTERM emitted JOB_CANCELLED and exited1 in0.0163seconds. Oversized32769-byte open stdin emitted INVALID_ARGUMENT, exited1 in0.1673seconds with no stdout/headless dispatch and unchanged project/history.
- Strict workspace Clippy passed with unchanged `cargo clippy --workspace --all-targets -- -D warnings`, workspace-clippy-final.log. Existing third-party proc-macro-error2 future-incompatibility notice is retained; no source warning suppression was introduced.
- TypeScript typecheck/lint passed on stable service/contract inputs (artwork-full-typecheck-stable.log and artwork-full-lint-stable.log); final entrypoint static reruns remain below. Locked hermetic Python worker suites passed. Python12 unittest+12 pytest, artwork-full-python.log; no model download.

## Environment setup and retained unsuccessful checks

This isolated environment initially lacked Bun/Rust/Moon. Installed exactly pinned Bun1.4.0, Rust1.97.0, Moon2.3.3 under /tmp. GitHub CLI REST/GraphQL returned Forbidden; enabled GitHub connector reads succeeded. Shell git fetch succeeded. No secrets/access expansion.

System apt install returned100 because global apt paths were unavailable; it was not an automatic review denial. Isolated signed public Debian indexes/deb extraction under /tmp provided documented desktop headers/libraries without global installation/config writes. Helper/evidence: desktop-env.sh and desktop-bootstrap-evidence.md. Generated cargo cache is an ignored repository target directory with an external scratch alias.

Retained failures:

- First complete required contracts command reached native speech_alignment and failed a genuinely introduced marker-layer omission (11passed/1failed in that suite). Root added exact47→46 inverse against independently captured artwork predecessor SHA before the unchanged historical chain. First focused integration applied it to an unowned blend catalog (12passed/1failed); corrected membership to the explicit artwork pin inventory. Final13passed, artwork-native-marker-inverse-final.log. Independent reviewer verified all10 inverse outputs equal main756b7b39 rawbytes, original chain unchanged and missing/duplicate/unrelated drift controls reject. Original failed pipeline remains in artwork-required-contracts.log; final complete rerun is mandatory below.
- Final governance review identified missing canonical CLI-envelope coverage despite behavioral tests. Approved clarification within1.2 adds fixture-owned existing request/response envelopes and real parser/output verification without changing behavior; exact patch was independently approved and85 affected parity tests/typecheck/targetedlint passed. Complete required pipeline rerun remains mandatory below.
- Initial strict Clippy rejected newly appended production functions after assets.rs test module. Moved the functions structurally before tests; unchanged full strict rerun passed. workspace-clippy.log retains original failure.
- Initial full unit run started before native build and under concurrent compilation:947 passed/62 failed/15 existing optional skipped. Failure details retained in artwork-full-unit.log: prebuild native spawn failures, one contracts5s timeout and six descendant cleanup cases. No timeout/assertion changes.
- Quiet six-case cleanup reproduction still failed with reported descendant PIDs all PPID1/STAT Z. artwork-native-cleanup-recheck.log and artwork-zombie-evidence.log establish the container PID1 does not reap adopted orphans. An isolated user-space PR_SET_CHILD_SUBREAPER supervisor (/tmp/opencut-tools/subreaper.py) runs the unchanged commands and reaps only adopted descendants; it does not modify test assertions. All23 targeted cleanup tests then passed (artwork-native-cleanup-subreaper.log). Supervisor cleanup begins only after the main test command is terminal, so it cannot satisfy live test assertions artificially.
- Raw source integration43 passed/1 descendant-reaping failure, artwork-source-integration.log. Unchanged supervised rerun passed44/44 in121.84seconds, artwork-source-integration-subreaper.log, including checked-in real CLI acceptance.
- Initial Moon failed on read-only default proto/wasmtime homes, then its network probe. Isolated PROTO_HOME/MOON_HOME/XDG_CACHE_HOME and MOON_TOOLCHAIN_FORCE_GLOBALS=true select already installed exact pins without modifying repository policy. Actual `moon run root:openspec-validate` then passed policy tests and all77 spec items, rejecting only this active change as expected before archival (moon-prearchive4.log). This is expected rejection, not protected gate success.
- Independent acceptance harness first misread the existing open_project response; corrected only the test harness, retaining artwork-independent-acceptance-harness-error.log. Product assertions had not failed.
- Root additionally reproduced an oversized open-stdin process hang after the byte-limit error; unconditional entrypoint-finally stdin destruction fixed it. Independent actual-process acceptance now covers both signal and oversize while the pipe stays open. Existing unit consumers do not load the source entrypoint; their1012-case evidence remains valid, supplemented by final actual-process acceptance and static checks.
- One execution-server transport disconnect recovered; real date/git/log commands succeeded at01:00:52UTC with all work retained.

## Pending final checks and publication

- Complete required contracts command, assembled runtime smoke, workspace tests, final formatting/typecheck/lint.
- Pinned strict all-spec validation and protected prearchive gate on stable inputs; openspec-verify-change completion report.
- Only after every required implementation check passes: synchronize/archive, rerun protected gate+strict specs, publish own branch/draft PR and inspect exact-head checks to terminal.
- Required CODEOWNER review; installed GUI and unsupported-platform file-review readiness are not claimed. Core is platform-neutral; CLI file review intentionally Linux-only.

## Integration constraints

#211 does not bump project schema or change migrations, according to its owner. Its package v1 componentSchemaVersion46 must remain a package format marker independent of project47. Consume/migrate supported old component payloads explicitly, preserving old packages and both six #211 plus two #220 operations. A package path lacking transitive derivation provenance must reject new derived-artwork packaging explicitly until approved extension. This branch does not modify or claim tested integration with #211. #208 local work, #215/PR228 and CI/FFmpeg workflow ownership remain untouched. No second issue, merge or deployment.
