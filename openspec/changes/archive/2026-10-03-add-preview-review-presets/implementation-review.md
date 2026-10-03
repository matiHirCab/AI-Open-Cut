# Independent implementation review

Status: **delegated implementation acceptance; final verification incomplete**.

This is delegated implementation review under the user's standing authorization, not human CODEOWNER acceptance, merge approval, or deployment approval. Reviewed root AGENTS.md, the specification lifecycle, approved proposal/design/deltas/spec-review/approval/tasks, the complete tracked implementation diff and new review fixture, tests and documentation on `codex/issue-71-preview-presets`.

## Findings resolved on re-review

1. **Essential instruction-prefix regression.** `apps/agent-bridge/src/instructions.ts:2` expands the preview guidance enough to remove polling, cancellation and overwrite guidance from the first 512 characters. The existing normative regression test at `apps/agent-bridge/tests/schemas.test.ts:134` fails (`/tmp/issue71-unit.log`: expected prefix to contain `Poll`). Keep the new review recommendation and restore concise essential workflow guidance within the tested prefix; rerun the schema and unit suites.

2. **Real new-operation workflow and cleanup coverage missing.** `apps/agent-bridge/tests/preview-review.test.ts:60` mocks both headless and `jobs.start`; `RenderWorker.accepts` at line 107 only checks classification. Neither integration `tests/smoke.test.ts` nor packaged `tests/packaged-smoke.test.ts` currently invokes the new tool. Thus task 3.3 lacks evidence for a real review job, artifact/default audio and explicit silence, ready/unavailable support, and MP4 cancellation/timeout cleanup through the newly added branch at `apps/agent-bridge/src/headless.ts:286`. Exercise the actual MCP → job → new headless worker route, packaged discovery/execution, and new-operation owned temporary cleanup. Reuse existing jobs/envelopes rather than redesigning them.

3. **Successful render immutability/repeat/reopen scenario incomplete.** State/reopen assertions at `crates/editor-core/tests/preview_review.rs:84` occur after pure options resolution; headless protocol coverage only renders failures. The native renderer parity test covers default audio/silence and decoded export parity on an in-memory Project, but not successful persistent repeated review followed by reopen with state/history/drafts unchanged. Add this approved scenario through an actual core/headless/MCP workflow and compare state plus persisted history/drafts around repeated successful reviews.

All three original findings above are resolved in the re-reviewed tree:

- `instructions.ts` now keeps the essential workflow concise and places detailed preset/audio guidance later. The complete unit rerun passes the existing prefix assertion.
- `preview-review-workflow.ts`, called by both integration and packaged suites, exercises the real MCP/job/headless route for default, 540p, 720p, project and custom silent review. It polls completed MP4 jobs, checks ready capability discovery, stale and malformed requests, and reopens persistent state. These workflows use the established fake FFmpeg/FFprobe adapter; native audio and decoded export parity remain covered separately by the real FFmpeg renderer test.
- The workflow retains a draft and compares exact project.json, history.json and every draft JSON byte before/after successful reviews and reopen; undo/redo verifies retained history. Default and explicit project selection repeat the same settings.
- The existing worker cancellation test now covers both draft PNG and new review MP4 temporary paths. Fake worker output matches each extension. The original descendant-PID absence assertion and published-output preservation checks are retained. A task-local Linux child-subreaper harness addresses this container's orphan-zombie behavior without relaxing repository assertions.

No remaining implementation-review findings. This acceptance does not mark pending task/check gates complete.

## Reviewed implementation without additional defects found

- Height-based sizing implements nearest-even ties-up and minimum width 2. Bounds validated before arithmetic make all u64 arithmetic safe for the u32 source inputs; canonical output settings validation remains core-owned.
- Strict custom Serde DTO plus untagged preset/custom union rejects malformed selections. New defaults select project settings and audio true; explicit false is retained.
- New renderer API delegates to existing range render semantics. Legacy numeric range validation and legacy MCP required resolution/fps/audio-off defaults remain unchanged; the above-bound fake-adapter regression exercises this compatibility distinction.
- Headless immutable revision validation precedes rendering. New worker operation classification and bridge temporary MP4 ownership are wired consistently with the legacy route.
- Ready-only capabilities use the existing rendering capability source; canonical protocol/worker/MCP fixture changes, manual catalog/digest update, ownership entries and CODEOWNERS are synchronized in the reviewed diff. The reviewed renderer → validation edge is reflected in both ADR 0003 and its architecture matrix.
- No persisted model/schema or job-result redesign is introduced. Existing preflight tests now include the new renderer facade for missing media/path failures.

## Evidence limitations

Inspected passing evidence: `/tmp/issue71-unit-3.log` (436 passed, one skipped), `/tmp/issue71-integration.log` (16 passed), `/tmp/issue71-native-review.log` (focused native visual/audio/export parity passed in 1.43s), `/tmp/issue71-lint-2.log` and `/tmp/issue71-typecheck-3.log`. Earlier instruction/worker unit failures are superseded by the passing unchanged assertions.

`/tmp/issue71-policy-prearchive-5.log` reaches the ordinary protected task and rejects only this active change, as expected before archival. Using `MOON_TOOLCHAIN_FORCE_GLOBALS=1` with actual binaries verified against unchanged .prototools pins (Bun 1.4.0, Moon 2.3.3, Rust 1.97.0) is an environment bootstrap accommodation, not protected-policy weakening. Prior Wasmtime/proto startup failures are superseded by this result; it is not a passing protected gate.

Workspace, packaged, semantic-union contract and release native-golden checks remain running or unresolved at re-review time. The earlier debug native-golden failure still needs passing final native evidence/triage. All mandated suites, conformance verification, synchronization/archival and postarchive protected gates remain the implementer's completion obligations. No final-check, merge-readiness or human CODEOWNER acceptance is claimed here.

## Final conformance acceptance

Independent Sol medium reviewer accepted all four requirements and ten scenarios after all executable gates passed, including release native golden, generated report validation, final integration16 and packaged10. No remaining implementation findings. Prearchive protected rejection remains expected and is not represented as a passed protected gate. Human CODEOWNER acceptance remains separate. Post-verification delivery gates remain explicitly tracked in tasks.md and verification.md until executed.

## Current-main reconciliation acceptance

Independent Sol medium reviewer accepted rebased scope at9d9aadf3 over current main480bd8d7. Merged speech contracts are preserved; no PR138 features were added. Independently recomputed digest matches7df40e56127433bfbc9dafbf5839e1d1a49c39bd6e734ee99661827ca3414e28. Post-verification delivery prose preserves all lifecycle obligations without claiming future actions complete. No new review blockers; affected check terminal results are recorded in verification.md before publication.

## CI fixture correction review

Independent reviewer accepted appending the new review fixture to preserve all legacy positional consumers and the regression test protecting those positions. Feature-enabled native worker4/4, formatting and strict Clippy evidence passed. Correction remains governed by the existing legacy compatibility requirement; production behavior and tolerances are unchanged. Final contract/native cache checks and replacement CI remain delivery gates.
