# Verification: add-speech-timestamp-capabilities

Completed 2026-10-03 using the repository-local openspec-verify-change workflow after loading status/apply context and all proposal, design, delta, task, and approval artifacts.

| Dimension | Result |
| --- | --- |
| Completeness | 9/9 implementation tasks complete; 3 requirements and 5 scenarios covered |
| Correctness | Each scenario maps to native tests and canonical fixture evidence below |
| Coherence | Provider/bridge ownership preserved; no project, headless, migration, rendering, job, alignment generation, or persisted provenance changes |

## Requirement and scenario evidence

- Independent speech timestamp support / independent declarations: `speechTimestampSupportSchema` and normalized `ttsStatusSchema`, service and adapter preserve flags; canonical `timestampSupportCases.valid` contains all eight combinations; `speech.test.ts` tests no synthesis/commit, `tts.test.ts` tests actual JSON-lines worker metadata, and `contracts.test.ts` validates canonical combinations.
- Independent speech timestamp support / truthful Kokoro: `worker.status` emits all false; Python `test_timestamp_support_matches_canonical_unsupported_in_every_state` covers dependency readiness and backend load combinations; `tts.test.ts` missing-Python fallback asserts all false.
- Compatible strict timestamp metadata / legacy: whole-object default only; `SpeechSynthesizer.status` accepts schema input; existing fake worker omits metadata; service and real adapter tests expose canonical all-false defaults. Existing synthesis/provenance parity, insertion, history, and lifecycle suites pass unchanged.
- Compatible strict timestamp metadata / malformed: nine canonical negative cases cover null, arrays, booleans, strings, missing keys, non-boolean values, and unknown granularity. Adapter and service safe parsing produce non-retryable `TTS_INVALID_CAPABILITIES`. Service tests invoke registered `tts_get_status` and assert the public MCP error envelope plus no inference/commit. Transport/startup errors remain outside schema translation.
- Governed parity: only `tts_get_status.outputSchema` changed in the canonical MCP catalog; the expanded digest deliberately updates to `07e3d60d5985c2722293c40712de5aa09ec83b3a2eef6a8679775559cafcefbe`. Contract tests compare every live registration. TS/Python consume canonical flag cases. Rust consumes unchanged synthesis/provenance and its existing gate passes. Ownership and CODEOWNERS include affected speech consumers.

## Required implementation checks

Full logs remain uncommitted outside the repository. All commands below completed with exit 0 except the deliberately pre-archive protected rejection listed separately.

| Check | Result | Full log |
| --- | --- | --- |
| `cargo fmt --check --all` | pass | `/tmp/issue59-fmt.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass | `/tmp/issue59-clippy.log` |
| `cargo test --workspace` | 856 passed, 9 existing ignored | `/tmp/issue59-workspace-tests.log` |
| bridge `bun run typecheck` | pass | `/tmp/issue59-typecheck.log` |
| bridge `bun run lint` | pass | `/tmp/issue59-lint.log` |
| bridge `bun run test:unit` | 467 passed, 1 existing opt-in skip | `/tmp/issue59-unit.log` |
| bridge `bun run contracts:check` | Rust contract suites and 362 TS tests pass | `/tmp/issue59-contracts.log` |
| bridge `bun run test:integration` | 15 passed on unchanged-timeout retry | `/tmp/issue59-integration-retry.log` |
| bridge `bun run test:smoke` | 9 packaged tests pass | `/tmp/issue59-packaged-smoke.log` |
| `bun run apps/agent-bridge/scripts/run-python-tests.ts` | 12 Kokoro + 5 transcription tests pass | `/tmp/issue59-python.log` |
| pinned OpenSpec `validate --all --strict --no-interactive` | 37 passed | `/tmp/issue59-strict-prearchive.log` |

Pre-archive `moon run root:openspec-validate` exited 1 solely because `add-speech-timestamp-capabilities` remained active, after its strict 37-item validation and policy/unit checks passed. This is the expected pre-archive rejection, not a passing protected gate. Log: `/tmp/issue59-prearchive-moon.log`. Final protected validation remains mandatory after synchronization/archival.

## Resolved check failures and environment evidence

- Initial typecheck caught missing test import and mock input types; corrected before passing checks. Initial Python state test had invalid context-manager grouping; corrected before the 12-test pass. Initial MCP catalog comparison caught key-order drift; preserved original catalog order and reran complete unit/contract gates.
- Container PID 1 retains orphan zombies, causing an unrelated process-cancellation unit assertion to fail. Tests subsequently run through `/tmp/issue59-subreaper.py`, which adopts/reaps descendants using Linux `PR_SET_CHILD_SUBREAPER`; no production code, assertions, retries inside tests, or timeouts changed.
- First workspace test build could not link desktop because installed runtime libraries lacked unversioned linker names. Local `/tmp/issue59-tools/native-libs` symlinks with `LIBRARY_PATH` enabled the complete unchanged workspace suite. No system or repository dependency edits were needed.
- Initial integration timed out at 60 seconds in the unrelated group/component workflow while other full suites ran. Complete retry passes with the same configured timeout. Original failing log retained at `/tmp/issue59-integration.log`.
- Pinned tools were installed locally: Bun 1.4.0, Rust 1.97.0 with rustfmt/Clippy, Moon 2.3.3. Moon/proto/cache paths use writable `/tmp`; supported `PROTO_OFFLINE=false` enabled downloads after network-probe failure. Repository policy, toolchain pins, and validation tasks are unchanged.
- Existing ignored/opt-in Rust and real-TTS tests are not newly skipped. No claim of real model inference or generated alignment validation is made; this issue implements metadata only.

## Reviews and approval

Independent spec review identified the status error translation gap, addressed explicitly in design and tests. Independent implementation review found no critical issues. Delegated issue-scoped approval is recorded in `approval.md`; it is not a human or CODEOWNER artifact approval. Designated owner review is requested on publication.

No critical, warning, or suggestion conformance findings remain. Ready for synchronization and archive; final merge-readiness and publication evidence will be recorded separately after the unchanged protected gate runs on the archive-only tree.
