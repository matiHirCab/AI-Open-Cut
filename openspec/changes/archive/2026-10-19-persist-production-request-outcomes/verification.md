# Verification report — persist-production-request-outcomes (#215)

Status: all required local implementation checks and independent conformance review passed. The four requirements/eight scenarios were synchronized into the living specification and this change was archived. Postarchive protected policy and pinned strict validation passed. This report was prepared before publication; exact published-head CI is recorded in PR metadata only after observation.

## Scope and conformance

Four requirements and eight normative scenarios are mapped in `traceability.md`. The implementation keeps rendering/domain authority in the existing core, four unchanged typed producer paths and existing process-local job/resource contracts. The bridge adds only explicitly opted-in bounded request/outcome metadata, five uniquely named governed tools and an opt-in capability. Admission, explicit start, queued cancellation and safe inventory reads remain separate; neither reconnect nor close replays a producer.

Independent reviewer `/root/review_177` reviewed artifacts before executable edits and reviewed the final store/service/observer/contract/projection/workflow implementation. Both reported P2 lifecycle defects were corrected and re-reviewed: observer plus diagnostic writer failure cannot retain job capacity; the POSIX crash witness always terminates its own stopped native process groups and retains original failures. The review found no remaining material implementation blockers. CODEOWNER review is separate and pending.

## Observed evidence

- Rust format, strict workspace all-target Clippy and full workspace tests passed: 1597 cases, zero failures, 14 existing ignored helpers/optional platform or explicit artifact drivers (82 test summaries). No ignored case was introduced by this change. Clippy reported an existing dependency future-compatibility notice for proc-macro-error2; no warning suppression was added.
- Bridge full typecheck and lint passed; lint checked 303 files. Full unit tests with files in series passed 1035 cases in 83 files; 15 pre-existing optional cases in ten files remain skipped, unchanged.
- Hermetic Python workers passed: 12 unittest and 12 pytest cases; no inference or model download.
- Focused implementation/compatibility revision passed 144 cases in 14 files; historical diagnostic negatives passed separately (21 cases in five files).
- Source MCP and assembled-package hermetic smoke first passed 44 and 41 cases respectively. The stable assembled-package rerun after the observer correction passed all 41; stable source MCP reached 43 passes and one existing grid-render polling timeout under concurrent native load. The subsequent complete source rerun passed all 44 cases with unchanged timeouts.
- Explicit actual native acceptance passed with source bridge and actual assembled release package, original visual/PCM fixtures and real FFmpeg/FFprobe. All four native producers produced independently checked outputs; two authoritative files and source WAV stayed byte-identical. Two queued-owner crashes and an entered native export crash retained bounded catalog/guards, exposed previous-runtime uncertainty, rejected old job adoption and caused zero replay. A published export with blocked terminal metadata remained uncertain until metadata-only graceful close, without another encode. After all Cargo builds settled, the complete real source acceptance passed again with final debug SHA256 `dfd3bce7b7e43f5be9f36a8511395a8f5377310dbb5d009d91151d6e85be8ac7`; the assembled package used release SHA256 `f95fbe45701ecbbb4197624c86b016b9bcf2de5a7c0f7ea67e38f5867b53a86c`. Both produced identical independently checked output hashes, including export `faf63713a5dec75163b580013b07ae7bca9520b4084f031675997445bc498bb6` and original WAV `5357aa9d473b37c3b4bd3504f3f928da46d4f76c6b17e280cc9ac5ce1ade0d77`.
- Prearchive unchanged Moon gate rejected only this active change, with 76 specifications passing. Standalone pinned strict validation passed all 76 items. This is expected prearchive rejection, not final protected success.

- Full unchanged `bun run contracts:check` passed all required native consumers and 646 TypeScript cases (42 files), including the new required durable fixture consumer and preserved historical negatives.

Full logs remain uncommitted under `/tmp/opencut-215-*`. All required implementation checks are complete. Own archival and postarchive final protected/strict validation subsequently passed; the prepublication results and checklist below complete local closure.

## Retained failures and corrections

The first explicit real-native attempts assumed ideal RGB/container equality despite the existing backend's YUV conversions. Their logs remain `real-source-{first,second,third}.log`. The corrected independent oracle compares every decoded pixel with a literal-color FFmpeg reference and the authored RGB value within explicit small conversion tolerances (PNG3/MP4 5); stream/audio frequency/RMS checks remain independent. No renderer behavior was changed.

Initial full unit tests found seven compatibility-test failures: two inventory counts omitted the new tools; historical negative controls walked an extra new Job diagnostic subtree; and the literal inverse wrongly assumed the original root key was still last when an unrelated negative-control property was appended. The fixes remove only independently specified exact additions before predecessor checks/mutations, preserving all six original budget mutations, raw byte pins, semantic hashes and unrelated drift. No historical pin or negative-control expectation was weakened.

Two initial source/package builds were deliberately interrupted after detecting a duplicate worktree Cargo cache, then rerun with the existing cloud cache. Their failure logs remain `integration.log` and `packaged-smoke.log`; the private target symlink is build state only and must not be committed.

A later full unit run under concurrent native workloads reached 1031 passes and four timeout failures (three existing 5-second parity cases and the existing 120-second compositing workflow), with 15 existing optional skips. Standalone diagnosis reproduced the two parity timeouts under the same load and no schema mismatch. Passing focused results do not waive these failures: the complete serial file run subsequently passed all 1035 executable cases, with original timeout/configuration/CI policy unchanged. All failed logs remain available.

## Compatibility, limits and delivery

Frozen b317 MCP and ownership predecessors have SHA256 `5148b0c3374400a76ce87c1245d29b6e13fa4d431b7f800bf02ec3be08b8edbd` and `5febc40dd7b61404226d263058fe628b27dd1e72297f0d3f8aead30c4bb4f4fe`. The required contract consumer checks the literal inverse against those raw fixtures and all existing proof chains remain enforced. New fixtures were manually authored from approved planning examples, rather than generated from live registrations. Native/headless/project/provider/error catalogs, package commands, authentication and CI policy remain unchanged.

The opted-in inventory is 64 records, 4096-byte intents, 8192-byte records, 1MiB catalogs, seven days from original admission, one fixed stage and a 36-byte nonce guard. The 2097188-byte named-content bound excludes filesystem metadata, media/exports and arbitrary foreign files. Active execution and dispatch uncertainty do not expire into permission to replay. Storage records may include caller export intent and project identity; never credentials, opaque predecessor jobs or reusable artifact tokens. Crash/foreign ownership remains an operator reconciliation boundary.

Runtime and POSIX crash acceptance is Linux evidence. It does not establish installed Windows/macOS playback, GUI readiness or subjective output quality. #208 WIP/device was never accessed or reconstructed; future central-file integration can require a semantic union. #213 Windows playback limitation and #214 pause remain separate. No merge or deployment is authorized or performed. Completion/publication must retain CODEOWNER-pending status and record exact published-head required CI results in PR metadata only after they occur.

## Prearchive conformance assessment

| Dimension | Assessment |
| --- | --- |
| Completeness | All implementation tasks through required suite execution are complete; four requirements and eight scenarios have automated evidence in traceability.md. |
| Correctness | Admission/claim/terminal debts, dispatch provenance, retention/whole-image ownership bounds, zero replay and exact additive compatibility match approved independent examples and real workflows. |
| Coherence | Core ownership, existing producers/jobs, thin injected transports, explicit opt-in and existing runtime authorization/guards are preserved. No dependency, wire, provider, package command or CI-policy change. |

No critical implementation mismatch or unresolved design warning remains. The independent review's two P2 findings were fixed, tested and re-reviewed; documentation drift was corrected. Task3.3 records this verification. The two closure tasks intentionally follow conformance: synchronization/archive, then final protected/strict success and verified publication preparation. They are not unfinished executable behavior, and no future CI result is prechecked. User authorization covers completing this required lifecycle without a separate confirmation prompt. CODEOWNER approval remains pending for the draft PR.

| Required command / acceptance | Full local evidence |
| --- | --- |
| cargo fmt --all -- --check | /tmp/opencut-215-final-rustfmt.log |
| cargo clippy --workspace --all-targets -- -D warnings | /tmp/opencut-215-final-clippy.log |
| cargo test --workspace | /tmp/opencut-215-final-rust-tests.log |
| bun run typecheck / lint | /tmp/opencut-215-stable-typecheck.log; /tmp/opencut-215-stable-lint.log |
| bun run test:unit --no-file-parallelism | /tmp/opencut-215-stable-unit-serial.log |
| bun run contracts:check | /tmp/opencut-215-stable-contracts.log |
| bun run test:integration | /tmp/opencut-215-stable-integration-retry.log |
| bun run test:smoke | /tmp/opencut-215-stable-packaged-smoke.log |
| bun run scripts/run-python-tests.ts | /tmp/opencut-215-final-python.log |
| Explicit actual source and assembled release runtime with real FFmpeg/FFprobe | /tmp/opencut-215-real-source-stable.log; /tmp/opencut-215-real-package-final.log; /tmp/opencut-215-native-package-build.log |
| Prearchive unchanged Moon / pinned strict | /tmp/opencut-215-prearchive-protected.log (only own active-change rejection); /tmp/opencut-215-prearchive-strict.log (76 passed) |

## Local closure and publication preparation

The living `durable-production-requests` specification contains exactly four approved requirements/eight scenarios. Only this change was archived, at `2026-10-19-persist-production-request-outcomes`. All eleven task checkboxes are complete; task4.2 concerns the prepared publication evidence/checklist, and does not claim future CI success. Publication is a draft PR only, with CODEOWNER pending and no merge/deployment authority.

Postarchive unchanged `moon run root:openspec-validate` passed 521 policy tests, all 76 specification items and the CI parity policy. Separate pinned strict validation also passed all 76. Full logs are `/tmp/opencut-215-postarchive-protected.log` and `/tmp/opencut-215-postarchive-strict.log`. The final metadata/checklist update receives one final prepublication gate run, whose full logs are `/tmp/opencut-215-prepublication-protected.log` and `/tmp/opencut-215-prepublication-strict.log`; publication requires those observed passes. Source/test/contract inputs remain unchanged from passing implementation evidence.

Before publishing, verify the staged implementation hashes and exact remote tree; preserve the private build cache, all retained failed logs and #208 exclusion. Observe actual published-head checks and all workflow conclusions before declaring delivery complete. Add their observed URLs/results to PR metadata rather than prechecking them here. The earlier local timeouts are resource-load failures with successful complete reruns, and are separate from the historical Windows decoder/CI investigation; no new head's failure may be called pre-existing without evidence.
