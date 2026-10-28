# Correction verification evidence

## Scope and retained failure

Base fc48d575d2f813e02718feb4c8b82855ae810a77, PR229, main b350b5cc85b4a9d31578992fde83f598e4392c68. Remote refs reconfirmed unchanged before correction verification. The original public integration failure is retained: [job122835122356](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/40693804457/job/122835122356), fake-provider source integration45pass/1fail, cannot start FFprobe (ENOENT); packaged step skipped. The test helper accidentally overrode supplied fake backends with ambient ffmpeg/ffprobe. No runtime defect is claimed.

## Tool and environment controls

Linux; task-local Bun/Rust/desktop development links and the same Node24.19 environment as the integration ledger. CI observed Node22.23.3; primary Node22 zlib documentation records crc32 added22.2.0, compatible with CI. No runtime/dependency/toolchain/workflow/profile/timeout changes. The existing corrected child subreaper /tmp/opencut-reap-tests-v2.py is a task-local launcher for container orphan reaping; no test assertion is altered. Full unit/contracts use explicit local --maxWorkers=1; no repository concurrency configuration changes. Other suites retain their defaults. Logs remain uncommitted under /tmp/opencut-hermetic-*.

## Focused proof

- Typecheck/lint and final backend regression tests3/3 pass; portable version delegation compares the real interpreter's direct and delegated status/stdout/stderr, including propagated failure, on every platform. Final logs: portable-backend.log, typecheck.log, lint.log.
- Hermetic source no-ambient proof: PATH=/tmp/opencut-hermetic-interpreters supplies only bun/node/python/python3, with no ffmpeg/ffprobe. Combined case1pass,45unselected. source-no-ambient.log.
- Hermetic assembled package no-ambient proof: official assembleRuntimePackage/verifyRuntimePackage with compiled bridge and release headless; same interpreter-only PATH; combined case1pass,41unselected. package-assembly.log and package-no-ambient.log. Explicit OPENCUT_TEST_BRIDGE_PATH and OPENCUT_TEST_HEADLESS_PATH identify the assembled executable pair.
- Genuine source proof: OPENCUT_ARTWORK_DURABLE_GENUINE=1 and explicit /usr/bin/ffmpeg,/usr/bin/ffprobe, combined case1pass,45unselected. genuine-source.log. Independent decoded RGBA length64*64*4 checked. Final genuine-source-final.log also passes1case, and genuine-package-full.log passes42/42 with the independent64x64 decoding assertion.
- Focused name selections are supplemental targeted proofs; unselected cases are not aggregate passes and do not substitute for mandatory full suites.

## Retained intermediate failures

First private probe returned stream metadata without frames, and native rendering rejected unusable metadata. source-focused.log and source-diagnostic.log retained; fixed by deriving frames from the same validated actual PNG dimensions, followed by source-focused-fixed.log pass. Initial private checksum implementation caused strict bitwise lint errors; replaced with supported built-in node:zlib crc32 without suppression. These initial results are not represented as passes. Original integration ledger and published artifacts are untouched.

## Reused unaffected evidence

Diff against fc48d575 confirms crates, headless, desktop, production bridge src, workers, contracts, artwork inputs, dependencies/lockfiles, workflows, profiles, shared fake-media-tool.mjs and prior archives unchanged. Prior integration verification therefore remains applicable for Rust formatting/strict Clippy, workspace1682pass/14existing ignored, explicit native desktop1pass, Python12unittest+12pytest and actual artwork acceptance. This reuse does not prove Windows behavior; new published-head supported CI remains required. Required bridge aggregate suites are rerun for this test correction.

## Aggregate and lifecycle results

Passed so far:
- bun run typecheck: exit0; typecheck.log.
- bun run lint: exit0,363files; lint.log.
- Full units with --maxWorkers=1:1188pass/15existing optional skips,1203total; unit-full.log. The portable delegation adjustment occurred during this aggregate, and its final focused3/3 run passed; a final stable-tree full-unit rerun is still required below.
- bun run contracts:check --maxWorkers=1 through the child launcher: exit0,533native+708TypeScript, no failures/skips; contracts-full.log.
- Root bun test scripts:627/627; root-policy.log.
- Strict all-spec validation:81/81; strict-prearchive.log.
- Protected pre-archive Moon:538policy controls and81spec items pass, then only preserve-hermetic-artwork-durable-acceptance is rejected by archive-only policy. This is expected rejection, not a passed gate; protected-prearchive.log.

- Full source integration:46/46, exit0; source-full.log.
- Full assembled package hermetic smoke:42/42, exit0; package-full.log.
- Explicit genuine source targeted case:1pass/45unselected, exit0; genuine-source-final.log.
- Full assembled package explicit genuine smoke:42/42, exit0; genuine-package-full.log; combined acceptance reports genuine-media-tools and checks independent64x64 RGBA decode.

- Final stable-tree full unit rerun:1188pass/15existing optional skips,1203total, exit0; unit-stable-full.log. No executable file changed during this rerun.
- Final hermetic source interpreter-only PATH proof:1pass/45unselected, exit0; source-no-ambient-final.log.

Conformance was reviewed against all four approved originals, the implementation diff and every scenario. Final lifecycle results are recorded after synchronization/archival below.


## Final lifecycle outcome

The approved additive requirement was synchronized into openspec/specs/durable-production-requests/spec.md and only this change was archived. Final unchanged protected Moon passes538policy controls plus80spec items, exit0 (protected-final.log). Separate strict all-spec validation passes80/80 (strict-final.log). Final root script policy passes627/627 (root-policy-final.log). No other active change or historical archive was altered.

Task4.1 remains intentionally unchecked in this committed local-verification snapshot: publication and exact new-head supported CI occur after local archival/final gates and are reported externally with the exact SHA and actual terminal statuses. This snapshot does not claim remote checks have passed. Genuine CODEOWNER acceptance, PR approval and merge/deployment permission remain separate. The denied earlier PR-description update is not retried; no public comments are posted.
