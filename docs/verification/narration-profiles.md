# Narration profile verification (#207)

Local implementation verification completed on 2026-10-16. This report captures prepublication checks; exact-head CI and designated CODEOWNER review remain subsequent mandatory delivery gates. PR223 was merged by the user on 2026-10-16; main is `883fabfe429227e9a5a8dcf9bd5e7d60b143a97c`, whose Git tree exactly equals predecessor head `31c63d2c0cb90843d1c2b191e8afe4903ee3c728`. The incremental draft will target main. Main's separately reported timeouts are not evidence for this change.

## Requirement traceability

| Requirement/scenario | Evidence |
| --- | --- |
| Shared bounded library / two projects | Core shared/versioned snapshot test and actual `verifyNarrationProfilesWorkflow`: both projects generate with identical pinned effective requests; updates do not change existing assets. |
| Shared library / stale writer | Core stale-update byte assertions, immutable versions, duplicate names and caller-visible REVISION_CONFLICT. |
| Shared library / invalid storage and values | Count/size/future/corrupt/duplicate/Unicode/control/boundary tests; Unix symlink target byte preservation. Bounded file operations belong to persistence; store retains domain validation/root-lock/atomic publication. |
| Explicit application / precedence and empty list | Core override/canonical literal tests, MCP omitted-versus-empty schema regression and actual pinned application. Missing textOptions remains absent; explicit empty pronunciations replaces the whole object. |
| Explicit application / unavailable profile/version/voice | Core missing/stale-version behavior; actual MCP stale-version and unsupported saved voice; service unavailable-provider and actionable voice errors without synthesis/project commit. |
| Attribution / independent reopen | Core committed asset survives library update/removal, reopen, undo/redo and stale project revision rejection; actual MCP regeneration preserves the earlier request/version. Profile preview retains its full snapshot across library edits and conflict retry without new inference. |
| Legacy lifecycle / cancellation during preflight | Deterministic filesystem barrier test failed before correction with TTS_TIMEOUT instead of JOB_CANCELLED; after correction it rejects before request dispatch, completes its FIFO successor and leaves only the successor artifact before cleanup. Original MCP cancellation remains unchanged. |
| Attribution / migration and incompatible metadata | Current/history45 adoption without invented attribution, unknown future47 rejection and four premature metadata cases (current/retained, null/object) preserving both authoritative files. |

The pure render-plan equivalence test covers every existing alignment case with both absent and present profile attribution, without any library/provider. The synthetic byte-transfer comparison uses an exact current in-memory fixture manifest and rejects manifest drift; its frozen observations, quantitative limits and historic pins are unchanged.

The provider receives existing fields only, without profile attribution. Provider-mutated text options are rejected and its owned artifact is cleaned. Canonical rollback fixtures invert only explicitly authored NP1 paths and keep all existing historical pins and unrelated fields; ten current-schema markers preserve their exact independently captured45 predecessors.

## Current local evidence

- Complete Rust workspace:1,582 passed,12 existing ignored, zero failures; format and strict workspace/all-target Clippy pass. Linux narration library:13/13 including symlink rejection after the persistence-owner correction. Focused architecture23/23 and slots10/10 also pass.
- Bridge focused:114/114 across seven suites and73/73 across the nine historical marker suites. Final full unit pass:954 executed cases,17 existing environment guards, zero failures (77 passing files plus10 existing guarded files), including the deterministic preflight regression. Types and lint pass. Worker tests:12 unittest plus12 pytest. Agent-context18/18. Protected CI policy498/498, including new omission/masking controls for both profile consumers.
- Pinned OpenSpec1.5.0 scoped validation and full strict validation pass75/0. The stale external bunx entrypoint was not inspected or modified; an owned cache under the authorized verification directory supplies the same pinned CLI.
- Full contract parity passes:645/645 TypeScript cases plus every native contract target. Final Linux source integration passes43/43 with the new profile case before the destructive restart. The first complete packaged run passed39/40, including the new profile case; the unchanged combined hero-card workflow exceeded its original30s limit while the full contract suite was still active. The complete isolated rerun passes40/40 with identical binaries, fixtures, tests and deadlines, plus the existing package assembler and integrity verifier. The new cancellation regression and provider/profile tests pass48 cases with one existing platform guard; synchronized ownership and historical predecessor controls pass82/82. Final types and lint pass. The complete final unit rerun passes954/954 with17 existing platform guards.
- The first two full Linux integration attempts exposed an existing cancellation-during-preflight race and downstream queued-job timeouts; the first additionally found the corrected schema45 documentation example. The deterministic regression confirmed the race before the narrowly specified correction. The full contract run overlapped the ownership/explicit-inverse update and failed7 matching-category assertions; those exact consumers now pass together after synchronization. These failures are preserved, not counted as successful checks.
- Ordinary native Linux packaged bootstrap built the native release headless successfully but pinned Bun1.4.0 compilation failed with ELF ftruncate EACCES, matching the independent one-line readiness probe. The supported pinned Windows-to-Linux cross-build passes compilation; the existing package assembler/integrity verifier and complete smoke acceptance pass40/40 in the isolated run. Native ordinary bootstrap and every exact-head CI gate remain mandatory.
- After the cancellation fix, all42 predecessor source-MCP cases passed; the newly appended profile test exposed its placement after the intentionally destructive bridge-restart test (Not connected). It now runs before that existing test in both source and packaged suites; neither predecessor order nor lifecycle assertions changed. The complete final source rerun passes43/43.
- Earlier failed/interrupted attempts remain preserved, including stale contract markers, a worker startup timeout, the interrupted Linux run and the earlier disconnected execution service. They are not counted as successful checks.

Full uncommitted logs are retained only at `D:/Users/Codex/AI-Open-Cut/verification-issue-207-2026-10-15`. Final conformance, synchronized/archived OpenSpec, protected Moon, scoped commit/push, owner review request, cumulative draft PR and its exact-head protected CI are mandatory delivery gates. No merge/deployment.

## Final local commands and logs

All logs below are uncommitted in the directory stated below. Passing evidence is reused only for unchanged relevant inputs. The final test-order edit affects the two MCP suites, both subsequently passed in full; unit/contract suites exclude those entrypoints.

| Check | Result | Log |
| --- | --- | --- |
| Rust format / strict workspace all-target Clippy | exit0 / exit0 | final-rust-format.log; final-clippy.log |
| cargo +1.97.0 test --workspace --no-fail-fast | 1,582 passed;12 existing ignored; exit0 | final-workspace.log |
| bun run typecheck / bun run lint | exit0 / exit0 | final-typecheck-test-order.log; final-lint-test-order.log |
| bun run test:unit --no-file-parallelism | 954 passed;17 existing platform guards; exit0 | final-unit-cancellation.log |
| bun run contracts:check --no-file-parallelism | every native target and645 TypeScript cases pass; exit0 | final-contracts-synchronized.log |
| Native Linux profile suite | 13/13 including symlink; exit0 | final-linux-profiles.log |
| bun run test:integration (Linux) | 43/43; exit0 | final-linux-integration-final-order.log |
| Existing package assembler/integrity + full smoke (supported pinned cross-build) | 40/40; exit0 | final-linux-cross-package-isolated.log |
| Ordinary native package bootstrap (Linux) | native release headless builds; Bun compile fails ELF ftruncate EACCES | final-linux-native-package.log |
| Hermetic workers | 12 unittest +12 pytest; exit0 | resume-workers.log |
| Agent context / CI policy | 18/18 +498/498; exit0 | resume-agent-context.log; resume-ci-policy-tests.log |
| Pinned strict all-spec validation | 75/75; exit0 | final-strict-prearchive.log |
| Protected prearchive Moon | rejects only the active change; expected exit1, not gate success | final-protected-prearchive.log |

Full evidence directory: `D:/Users/Codex/AI-Open-Cut/verification-issue-207-2026-10-15`. Native ordinary package bootstrap, platform/GUI/foundation CI and designated review remain mandatory at the published head; this report does not substitute cross-build success for those delivery gates. No merge, deployment, paid inference or model download occurred.

Final conformance found no remaining implementation mismatch across4 requirements and10 scenarios. Both living capabilities were synchronized and the completed change archived at `openspec/changes/archive/2026-10-16-save-versioned-narration-profiles/`. Pinned strict postarchive validation passes75/75 living specs with no active change, and final protected `moon run root:openspec-validate` passes (exit0). Logs: `final-strict-postarchive.log` and `final-protected-postarchive.log`. The final runtime manifest verifies62 identical runtime/fixture/consumer files between the deliverable source and the Linux verification checkout; documentation evidence and archived planning are tracked separately from runtime inputs.

## PR224 current-contract consumer repair

Initial head8e83736 ended with13 successful and5 failed checks: three Complete motion release platforms rejected schemaVersion, Render parity rejected78 versus83 in both masked-hero cases, and Foundation propagated the failed prerequisite. Acceptance remains pending for the repaired head.

The retained Ubuntu artifact12047829840 proves actual schema46 with the original recipe hash and an independent clean source-identity record matching requested/checkout head8e83736 and run nonce. The private validator and its synthetic fixture still required45. Two new regressions also exposed accidental optionality in the documentation marker. Both consumers now require the exact unwrapped current status-schema literal; stale45, future47, absent and string metadata fail. The intact actual report replays successfully without altering any measurement, recipe/font hash, witness, count or envelope. This is old-head retained evidence, not new-head native capture.

The five extra MCP names are exactly narration_profile_apply/create/get/list/update. Native masked-hero discovery now matches the entire independently authored canonical name inventory, requires each of those five once and excludes only exact approved additions before the unchanged78-tool assertion. Historical catalog/reference bytes, other native assertions, media tolerances,120s deadlines and CI commands are untouched. The governed consumer list and its explicit inverse add this test without changing any predecessor pin.

| Repair check | Result | Log |
| --- | --- | --- |
| Failing-before report/documentation controls and native artifact replay |2 failing controls/16 pass; replay reproduces schema mismatch | release-schema-before-tests.log; release-schema-before-replay.log |
| Corrected controls / intact native report replay |18/18; replay exit0 with independent source identity and unchanged envelope | release-schema-after-tests.log; release-schema-after-replay.log |
| Final types / lint / full unit |exit0 / exit0 /956 passed,17 existing platform guards | release-consumers-typecheck.log; release-consumers-lint.log; release-consumers-unit.log |
| Complete governed contract command, serial |all native targets and645/645 TypeScript pass; exit0 | release-consumers-contracts-serial.log |
| Actual affected source / assembled package documentation workflows |1 passing each;42/39 explicitly unselected in focused runs | release-schema-source-documentation.log; release-schema-packaged-documentation.log |
| Actual native masked-hero standalone and alias cases |2/2;89.805s and71.786s within unchanged120s deadlines | release-consumers-masked-hero-release.log |
| Policy/process/preflight |49/49 | release-schema-policy.log |
| Strict prearchive / protected prearchive |76/76; protected rejects only active correction | release-consumers-strict-prearchive.log; release-consumers-protected-prearchive.log |

Local debug WSL/NTFS native verification exceeded120s and the following case encountered the inherited occupied runtime; both failures are preserved in release-consumers-masked-hero-native.log. The successful run uses the existing default-feature release headless and a fresh owned per-run TTS-work directory, preserving every original case/oracle/deadline. No previous lock was deleted or adopted. Ordinary debug native CI remains required at the new published head. The first contract rerun overlapped unit use of the Windows executable and failed replacement with access denied; release-consumers-contracts.log preserves it, and the complete serial rerun passes. Earlier failures remain retained, not counted as success.

Rust production/core/headless sources, canonical API/recipe/model catalogs, dependencies, workers and bridge runtime remain unchanged by this repair. Prior full workspace1,582/12 ignored, workers12+12, context18 and complete source43/package40 evidence is retained for unchanged relevant inputs; affected document execution was rerun through both actual paths. All18 exact-new-head checks, ordinary bootstrap/native/platform/foundation acceptance and designated CODEOWNER review remain mandatory. The final seven repair inputs match the Linux snapshot by SHA256 in release-consumers-verified-snapshot.json. No dependent implementation, merge, deployment, paid inference or outside-tree inspection occurred.

The correction is synchronized across the three affected living capabilities and archived at `openspec/changes/archive/2026-10-16-align-current-release-schema/`. Final pinned strict validation passes75/75 living specs with no active change and protected `moon run root:openspec-validate` passes (exit0). Logs: release-consumers-strict-postarchive.log and release-consumers-protected-postarchive.log. Commit/push, exact-new-head CI and designated review follow as mandatory delivery gates.

Final review restored the affected Markdown from the immutable base as explicit UTF-8. Every unmodified requirement block is verified byte-for-byte against8e83736, while the seven executed code/fixture inputs retain their verified SHA256 values. The failed commit attempt did not create or push a commit; final specification integrity is recorded in release-consumers-spec-integrity.json.
