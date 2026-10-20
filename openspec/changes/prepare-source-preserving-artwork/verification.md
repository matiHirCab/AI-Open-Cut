# Verification of issue220

Status: implementation and independent production conformance approved; whole-workspace and exact-head publication gates remain pending. Do not treat this report as merge readiness until the terminal evidence below is completed.

## Scope and approval

Only #220 is implemented. Baseline main756b7b39ce0e2b1d5fcc65291084838da5331517. Independent specification approval and narrow amendments are recorded in approval.md. Independent reviewer inspected production core/CLI and approved conformance after reproducing and fixing three filesystem/input defects and a real subprocess exit defect. Reviewer authored the independent verification harness and verification/contract-test documentation contributions, not core/CLI production implementation. CODEOWNER acceptance is separate and pending. No workflow, package command, timeout, warning suppression or historical raw capture/digest was weakened or repinned.

## Requirement and scenario traceability

| Requirement | Implementation | Automated evidence |
| --- | --- | --- |
| Bounded deterministic artwork transformation | assets/artwork.rs, model/artwork.rs |12 exact raster cases: alpha, flood reachability, white interior, holes/fine line, padding, radius0, partial alpha/source-over, tolerance endpoints, palette/tRNS/sub8/16-bit, gamma ignored, corrupt/APNG/limits, maximum supported dimensions |
| Explicit inspect apply and discard | src/artwork/cli.ts, artwork-cli.ts |18 CLI cases: exact named files/hashes, explicit approval, all four altered files, foreign/missing/linked/FIFO files, escaped paths, existing destination, parent substitution, unsupported backend, idle cancellation, late completion, partial staging cleanup |
| Atomic source-preserving derivative commit | store/artwork.rs, existing persistence |11 core integration cases and2 private WAL fault cases: exact source/output bytes, one revision/asset, duplicate/stale/tampered intent, source deletion guard, undo/redo/reopen, before-journal rollback and after-journal recovery; missing probe facts preserve old operation behavior |
| Source-preserving artwork derivative ownership | assets.rs canonical reference/integrity/GC owner |core artwork cases cover current, history, draft transitive source roots, dangling/mismatch/self/two-node cycles, source/destination links, eventual history-eviction collection;23 architecture tests preserve dependency matrix |
| Artwork metadata schema migration | model.rs, migrations.rs |schema46 current/history migration is deterministic/idempotent; future and premature derivation preserve authoritative bytes; historical narration13 cases retain existing intent;91 headless cases include strict additive operations |
| Source/operator acceptance, with distinct genuine and hermetic modes | scripts/verify-artwork-preparation.py, explicit hermetic source smoke registration |Default actual Bun CLI+debug headless+FFmpeg/FFprobe; separate hermetic source smoke uses the exact synthetic probe and independent decoder, both with hand-built original PNG and literal independent RGBA/mask/light/dark oracles, hashes, preview byte preservation, commit/conflict/undo/redo/reopen/discard and real open-stdin SIGTERM exit |

## Passing focused checks

Full logs are local, uncommitted and outside the repository under /tmp/opencut-tools:

- `cargo test --locked -p opencut-editor-core --lib assets::artwork::tests`:12 passed, artwork-raster-tests-png181.log.
- `cargo test --locked -p opencut-editor-core --test artwork_preparation`:11 passed, core-artwork-probe-guidance.log.
- `cargo test --locked -p opencut-editor-core --test artwork_preparation --test architecture`:10 then-current artwork+23 architecture passed, core-artwork-final.log; later11-case suite adds missing-probe evidence.
- Private artwork WAL fault cases2 passed, core-artwork-faults.log; historical narration13 passed, core-artwork-tests.log.
- `cargo test --locked -p opencut-headless`:91 passed, core-artwork-headless.log.
- `bunx vitest run --config vitest.unit.config.ts tests/contracts.test.ts tests/narration-profiles.test.ts tests/artwork.test.ts`:67 passed, artwork-contract-final.log. Own additions are inverted precisely before retained predecessor chains; historical byte pins remain unchanged.
- Final complete TypeScript run passed1013 tests,15 existing optional cases skipped across80 passing/10 optional-skipped files in175.65seconds, artwork-full-unit-final.log. The earlier1012-case complete run is retained in artwork-full-unit-subreaper.log. Final governance-only patch then passed all85 affected tests (18CLI+67contracts/artwork/narration) across4files, artwork-cli-canonical-focused.log; actual serialized envelopes match canonical examples. No old case removed. Latest release docs18 passed, artwork-release-doc-latest.log. Release markers18 passed, /tmp/opencut-artwork-release-markers.log.
- Actual checked-in native acceptance passed again on final entry lifecycle fix, artwork-checked-in-acceptance-final.log; retained synthetic project/evidence path is in that log. Actual open-stdin SIGTERM emitted JOB_CANCELLED and exited1 in0.0163seconds. Oversized32769-byte open stdin emitted INVALID_ARGUMENT, exited1 in0.1673seconds with no stdout/headless dispatch and unchanged project/history.
- Strict workspace Clippy passed with unchanged `cargo clippy --workspace --all-targets -- -D warnings`, workspace-clippy-final.log. Existing third-party proc-macro-error2 future-incompatibility notice is retained; no source warning suppression was introduced.
- TypeScript typecheck/lint passed on stable service/contract inputs (artwork-full-typecheck-stable.log and artwork-full-lint-stable.log); final entrypoint/source-smoke static reruns passed (see final ledger). Locked hermetic Python worker suites passed. Python12 unittest+12 pytest, artwork-full-python.log; no model download.

## Environment setup and retained unsuccessful checks

This isolated environment initially lacked Bun/Rust/Moon. Installed exactly pinned Bun1.4.0, Rust1.97.0, Moon2.3.3 under /tmp. GitHub CLI REST/GraphQL returned Forbidden; enabled GitHub connector reads succeeded. Shell git fetch succeeded. No secrets/access expansion.

System apt install returned100 because global apt paths were unavailable; it was not an automatic review denial. Isolated signed public Debian indexes/deb extraction under /tmp provided documented desktop headers/libraries without global installation/config writes. Helper/evidence: desktop-env.sh and desktop-bootstrap-evidence.md. Generated cargo cache is an ignored repository target directory with an external scratch alias.

Retained failures:

- First complete required contracts command reached native speech_alignment and failed a genuinely introduced marker-layer omission (11passed/1failed in that suite). Root added exact47→46 inverse against independently captured artwork predecessor SHA before the unchanged historical chain. First focused integration applied it to an unowned blend catalog (12passed/1failed); corrected membership to the explicit artwork pin inventory. Final13passed, artwork-native-marker-inverse-final.log. Independent reviewer verified all10 inverse outputs equal main756b7b39 rawbytes, original chain unchanged and missing/duplicate/unrelated drift controls reject. Original failed pipeline remains in artwork-required-contracts.log; final complete rerun passed (see final ledger).
- Final governance review identified missing canonical CLI-envelope coverage despite behavioral tests. Approved clarification within1.2 adds fixture-owned existing request/response envelopes and real parser/output verification without changing behavior; exact patch was independently approved and85 affected parity tests/typecheck/targetedlint passed. Complete required pipeline rerun passed (see final ledger).
- Initial strict Clippy rejected newly appended production functions after assets.rs test module. Moved the functions structurally before tests; unchanged full strict rerun passed. workspace-clippy.log retains original failure.
- Initial full unit run started before native build and under concurrent compilation:947 passed/62 failed/15 existing optional skipped. Failure details retained in artwork-full-unit.log: prebuild native spawn failures, one contracts5s timeout and six descendant cleanup cases. No timeout/assertion changes.
- Quiet six-case cleanup reproduction still failed with reported descendant PIDs all PPID1/STAT Z. artwork-native-cleanup-recheck.log and artwork-zombie-evidence.log establish the container PID1 does not reap adopted orphans. An isolated user-space PR_SET_CHILD_SUBREAPER supervisor (/tmp/opencut-tools/subreaper.py) runs the unchanged commands and reaps only adopted descendants; it does not modify test assertions. All23 targeted cleanup tests then passed (artwork-native-cleanup-subreaper.log). Supervisor cleanup begins only after the main test command is terminal, so it cannot satisfy live test assertions artificially.
- Raw source integration43 passed/1 descendant-reaping failure, artwork-source-integration.log. Unchanged supervised rerun passed44/44 in121.84seconds, artwork-source-integration-subreaper.log, including checked-in real CLI acceptance.
- Initial Moon failed on read-only default proto/wasmtime homes, then its network probe. Isolated PROTO_HOME/MOON_HOME/XDG_CACHE_HOME and MOON_TOOLCHAIN_FORCE_GLOBALS=true select already installed exact pins without modifying repository policy. Actual `moon run root:openspec-validate` then passed policy tests and all77 spec items, rejecting only this active change as expected before archival (moon-prearchive4.log). This is expected rejection, not protected gate success.
- Independent acceptance harness first misread the existing open_project response; corrected only the test harness, retaining artwork-independent-acceptance-harness-error.log. Product assertions had not failed.
- Root additionally reproduced an oversized open-stdin process hang after the byte-limit error; unconditional entrypoint-finally stdin destruction fixed it. Independent actual-process acceptance now covers both signal and oversize while the pipe stays open. Existing unit consumers do not load the source entrypoint; their1012-case evidence remains valid, supplemented by final actual-process acceptance and static checks.
- One execution-server transport disconnect recovered; real date/git/log commands succeeded at01:00:52UTC with all work retained.

## Pending final checks and publication

- Final workspace Rust tests/Clippy/raster rerun; current formatting/typecheck/lint, complete contracts, source integration and assembled runtime smoke passed.
- Pinned strict all-spec validation and protected prearchive gate on stable inputs; openspec-verify-change completion report.
- Only after every required implementation check passes: synchronize/archive, rerun protected gate+strict specs, publish own branch/draft PR and inspect exact-head checks to terminal.
- Required CODEOWNER review; installed GUI and unsupported-platform file-review readiness are not claimed. Core is platform-neutral; CLI file review intentionally Linux-only.

## Integration constraints

#211 does not bump project schema or change migrations, according to its owner. Its package v1 componentSchemaVersion46 must remain a package format marker independent of project47. Consume/migrate supported old component payloads explicitly, preserving old packages and both six #211 plus two #220 operations. A package path lacking transitive derivation provenance must reject new derived-artwork packaging explicitly until approved extension. This branch does not modify or claim tested integration with #211. #208 local work, #215/PR228 and CI/FFmpeg workflow ownership remain untouched. No second issue, merge or deployment.

## Source smoke verification amendment

PR229 source fake-provider job118401718508 failed the newly added artwork subprocess because that job intentionally has no FFmpeg executable; its43 prior source cases passed. This failure is retained and is not a passed source gate. Independent review approved an explicit test-only `--hermetic` source-smoke mode: a probe stub accepts only the exact generated PNG, an independent stdlib decoder verifies normalized RGBA8 PNG pixels with CRC/zlib/filter0..4, and every original state/hash/history/CLI-process assertion remains enabled. Default acceptance still requires genuine FFmpeg/FFprobe and is separate native evidence. No workflow, install step, timeout or production behavior changes.

Both updated commands passed on stable debug headless SHA256 d19feff5969544431ef02a9f17d73f22c6248cfff8b09d0eb1517231a5578963: `python3 scripts/verify-artwork-preparation.py --hermetic` (artwork-hermetic-acceptance.log, retained evidence /tmp/opencut-artwork-acceptance-70fh3nrx/evidence.json) and the unchanged default `python3 scripts/verify-artwork-preparation.py` (artwork-genuine-media-acceptance.log, /tmp/opencut-artwork-acceptance-llvboto5/evidence.json). Reports explicitly distinguish hermetic from genuine-media-tools; both preserve source/result hashes and every exact pixel/history/process oracle. The default run used actual FFmpeg/FFprobe. Independent review also approved the five mechanical chunks_exact/as_chunks Clippy repairs as behavior-preserving. Full source smoke and final static checks subsequently passed (see final ledger); new-head CI remains separate required evidence.

## Final check ledger (implementation gates in progress)

- Final formatting: `cargo fmt --all -- --check`, exit0, artwork-format-final.log.
- Final TypeScript static checks after hermetic source-smoke registration: `bun run typecheck` and `bun run lint`, both exit0; artwork-typecheck-ci-fixes-final.log and artwork-lint-ci-fixes-final.log (305files, no fixes).
- Final strict specification validation before archival: pinned `validate --all --strict --no-interactive`, exit0,77items; artwork-strict-prearchive-final.log.
- Final protected prearchive command: unchanged `moon run root:openspec-validate`, exit1 solely because prepare-source-preserving-artwork remains active; policy tests and77spec items passed. artwork-moon-prearchive-final.log. This is the expected prearchive rejection and never a passed protected gate.
- Initial published8f8d0045 CI detected five constant-size chunks_exact lint errors on all three correctness platforms. Local exact1.97 setup did not emit this lint; CI setup says1.97 installed but diagnostic docs link rust-clippy/rust-1.99.0. Actual CI cargo-clippy version was not separately logged; no PATH root cause is claimed. Only five new artwork loops were changed to stable as_chunks, preserving traversal/remainders/pixels. Independent review approved; no suppression, pin or workflow change.
- Initial published8f8d0045 fake-provider integration passed43 historical cases and failed the new artwork acceptance because that job intentionally lacks FFmpeg. The explicitly approved hermetic mode fixes that verification dependency; default acceptance still requires and passed genuine FFmpeg/FFprobe. Original failure job118401718508 is retained as evidence. Initial Contract parity and Windows renderer startup jobs passed; exact final-head outcomes will be reported on PR229 after all fixes and lifecycle changes are pushed.

## Explicit scenario index

All five normative requirements and eleven delta scenarios have automated coverage:

| Delta scenario | Concrete test evidence |
| --- | --- |
| Inspect exact alpha and edges | assets::artwork::tests twelve literal raster cases; independent genuine/hermetic acceptance result/mask/light/dark pixels |
| Reject unsupported or bounded input | malformed_oversized_animated_and_unsupported_sources_are_typed_failures; preparation_is_deterministic_from_original_and_maximum_bounds_are_admitted |
| Review and reject ambiguous cleanup | CLI exact bounded review/explicit approve/discard cases; raster enclosed white/hole/tolerance cases; actual native discard preserves project/source |
| Reject unavailable backend and cancellation while reading | CLI unsupported-backend/idle-stdin cancellation/oversize tests; actual open-pipe signal and oversize process acceptance |
| Reject altered or unsafe review files | CLI four altered-file cases, foreign/missing/symlink/FIFO/parent-substitution/staging cleanup tests |
| Apply reopen undo and redo | preview_is_nonmutating_and_commit_reopen_history_preserve_exact_source_and_pixels; both actual-process acceptance modes |
| Conflict duplicate cancellation and injected failure | conflicts_tampering_and_invalid_parameters_never_publish_assets; both private artwork WAL failure tests; CLI cancellation/late completion; actual repeated apply conflict |
| Retain source through derivative and history | derivative_source_delete_guard_and_history_keep_owned_bytes; retained_draft_keeps_derivative_and_transitive_source_logically_owned; unreachable_derivative_and_original_collect_only_after_history_eviction |
| Reject corrupted derivative provenance | malformed_provenance_and_retained_cycles_fail_closed_without_rewrite; two_asset_cycle_and_history_only_dangling_provenance_fail_closed; symlinked_source_and_destination_preserve_foreign_files_and_project_state |
| Migrate retained legacy history | schema46_current_and_history_migrate_idempotently_and_future_or_premature_are_preserved; historical narration and full native producer migration suites |
| Reject future or premature derivation | same schema46 migration test checks authoritative current/history bytes; malformed provenance/current history tests |

- Final unchanged `bun run contracts:check`: exit0, all native producer suites passed (500native tests across38cargo suite summaries, zero ignored, including desktop compositing18/18), TypeScript647/647 across42files. artwork-required-contracts-final.log. This supersedes the retained failed first run without removing its evidence.

- Final complete `bun run test:integration`: exit0,44/44 in104.07seconds, including explicit hermetic artwork source-CLI acceptance. artwork-source-integration-hermetic-final.log. All43 historical source cases remain unchanged and passed.

- Final unchanged `bun run test:smoke`: exit0, real release headless build1m35seconds, compiled bridge and verified assembled runtime,40/40 packaged tests in45.36seconds. artwork-packaged-smoke-final.log.

- Initial-head native renderer Windows job118401719665 failed the packaged preview-disposal race at tests/preview-disposal-workflow.ts:101 with ENOENT reading its external victim PNG; source renderer passed. The disposal test and platform harness are unchanged from baseline. macOS/Linux native renderer and all three complete-motion-release jobs passed. This is retained exact observed evidence, not an independently reproduced baseline attribution; CI owner owns stabilization and this branch leaves those paths/workflows untouched. Final-head recurrence remains a publication blocker if present.

Archive readiness is under explicit independent re-review after the observed required Windows native job failure. Task4.2 is conservatively reopened until gate classification/required terminal evidence is resolved; no archive is authorized by this report while that question remains.

Independent gate decision: required Windows job118401719665 is an unresolved implementation check under4.2. AGENTS and docs/spec-driven-development.md require every required implementation check to pass before verify/archive. The administrative publication split cannot waive or reclassify this known failure. Keep4.2 pending, publish scoped fixes with active spec, obtain fresh required Windows native evidence and coordinate CI owner if it recurs; then perform final conformance/archive and separately require exact-final-head CI. No baseline cause, exemption or archive permission is claimed.

- Final repaired raster suite: exit0,12passed/0failed/0ignored,1.58seconds, artwork-raster-ci-fix-final.log. Final unchanged strict workspace Clippy: exit0,19.15seconds, workspace-clippy-frozen-final.log; existing third-party future-incompatibility notice retained, no suppression. Full workspace tests remain in progress.
