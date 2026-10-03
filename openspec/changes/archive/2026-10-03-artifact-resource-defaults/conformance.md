# Issue #74 implementation conformance

## Assessment

The four artifact-resources requirements and all eight named scenarios are implemented and covered. Independent specification approval, the canonical-URL amendment, and implementation re-review are recorded in approval.md. No unresolved implementation, contract or scenario mismatch remains. Post-verification sync/archive and final protected validation are ordered lifecycle steps, followed by draft delivery and exact-head CI; no merge/deploy is authorized.

## Traceability

| Requirement / scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| Metadata-first job content; completed outputs / no output | server/artifacts.ts jobWithArtifactResource, server/shared.ts previewJobResponse, server/context.ts job-status projection; schemas.ts artifactResource | artifact-responses canonical fixtures for PNG/range/export/WAV, absent/false flags, identical JSON/text metadata, no speech-service reads with absent files; pending/running/failed/cancelled and conflict metadata preserved |
| Explicit binary delivery and compatibility; legacy request / validation-discovery | server/jobs.ts includeBinary, shared responder, projects.ts bridge-only artifact_resources_v2; artifact-delivery-v2 and narrow MCP catalog updates | real MCP explicit PNG/WAV equality and video links, boolean rejection, contracts.test structural schemas/resource/capability and major-2 policy, architecture degraded-health capability, source and packaged smoke |
| Owned local retrieval; success / URI-path-ownership attacks | server/jobs.ts canonical SDK-normalized URL and UUID lookup; artifacts.ts confined open/lstat/realpath/O_NOFOLLOW; existing HTTP middleware unchanged | PNG/WAV/MP4 blobs, canonical/plain/encoded-dot aliases, invalid/encoded/foreign UUIDs and suffix/query/fragment rejection; traversal/drive/UNC/network/control/path attacks; root/project/directory/file symlinks and directory refusal; trusted parent alias retrieval with escaping descendant refusal; generated-media and ttsWorkDirectory roots; unauthenticated HTTP resources/read 401 |
| Retention and safe failures; missing-expired / preserve state | registry TTL/eviction unchanged, speech previewAudio retained-token authority unchanged, safe VALIDATION_FAILED and GENERATED_ARTIFACT_NOT_FOUND errors | deletion leaves metadata poll available; explicit missing safe errors; job expiry/eviction/restart/foreign registry; consumed/discarded speech through real source/packaged workflows; unit sanitization of private token/path; no revision/session mutation or repeated inference/render |

Canonical behavior fixtures are consumed by artifact-responses.test.ts, now included in the standalone contracts:check command, alongside existing complete structural MCP parity. Catalog changes are manually scoped to job_get_status includeBinary, the shared job descriptor, one resource template and one bridge capability; existing animation/provider sections are preserved. Headless unions, core artifact responses, renderer output, project schemas, migrations and domain validation do not change.

## Completed implementation checks

All commands use the repository's pinned Bun 1.4.0, Rust 1.97.0 and Moon 2.3.3. Full output is retained locally under /tmp/issue74-*.log; build/cache/provider outputs and logs are not committed.

| Command | Final result / log |
| --- | --- |
| cargo fmt --check --all | exit 0; issue74-fmt.log |
| cargo clippy --workspace --all-targets -- -D warnings | exit 0; issue74-clippy.log |
| cargo test --workspace | exit 0; 856 passed, 9 existing ignored; issue74-rust-tests.log |
| bridge bun run typecheck | exit 0; issue74-types-last.log |
| bridge bun run lint | exit 0; 87 files; issue74-lint-last.log |
| bridge bun run test | exit 0; 459 passed, 1 existing optional native opt-out; issue74-unit-last.log |
| bridge bun run contracts:check | exit 0 including native subset and 389 TS cases; issue74-contracts-complete.log |
| bridge bun run test:integration | exit 0; 15 real MCP workflows; issue74-integration-final.log |
| bridge bun run test:smoke | exit 0; 9 packaged workflows with artifact delivery; issue74-packaged-final.log |
| bun run apps/agent-bridge/scripts/run-python-tests.ts | exit 0; hermetic Kokoro 10 and transcription 5; issue74-python.log |
| pinned OpenSpec validate --all --strict --no-interactive | exit 0; 37 items before synchronization; issue74-spec-before-final.log |
| moon run root:openspec-validate before archive | expected exit 1 only for this active change; all strict specs/policy tests pass; issue74-moon-before-archive.log |

## Initial failures and environment repair

The first desktop workspace test link failed because the container lacks unversioned xcb/xkbcommon linker names. A local toolchain library directory points to the existing system runtime libraries; no repository code, dependency, suppression or assertion was changed. The full workspace command then passed.

The existing render-worker descendant-kill assertion initially observed a zombie because container PID 1 did not reap the orphan. An uncommitted Python PR_SET_CHILD_SUBREAPER wrapper reaped the killed child; the identical assertions and commands then passed. The wrapper uses /proc stat enumeration (this kernel does not expose task children files), reaps only its own orphaned children, and leaves production code/tests untouched. The final unit/contract logs record one reaped orphan.

Initial source and packaged group-workflow checks hit their unchanged timeouts while native workspace/contract checks were running concurrently. Subsequent runs passed with the workload reduced and no timeout/test selection changes. Earlier failed logs are retained (issue74-unit.log, issue74-contracts.log, issue74-integration.log, issue74-packaged.log). Optional real provider inference and optional unconfigured native suites are not claimed executed; this issue changes MCP delivery, not providers or rendered semantics.

Moon initially encountered read-only default cache paths and proto network detection. Tool/cache homes were confined to /workspace/toolchain, and MOON_TOOLCHAIN_FORCE_GLOBALS uses the exact installed pinned versions. All protected task contents and policy checks remain unchanged; the pre-archive protected gate rejects only artifact-resource-defaults as required. Post-archive gate success is required before verified commit/draft delivery.

## Limits and review status

Explicit blob/inline reads buffer requested bytes, as the existing MCP content semantics require; ordinary polling opens no artifact file. Links retain current path-backed export semantics, including later overwrite. Authorized local filesystem writers remain trusted, so no new OS sandbox or immutable-snapshot guarantee is claimed. Existing clients of one authenticated server share its job registry just as before; activeProjectId is not an ACL. Speech tokens remain intentional commit fields and are absent from links and safe retrieval errors.

Human CODEOWNER review and GitHub exact-head CI are separate delivery evidence; neither is fabricated by this local conformance report.

## Lifecycle verification

openspec-verify-change: 7/7 implementation tasks complete, four requirements and eight scenarios covered; no critical, warning or coherence mismatch. Independent reviews resolved, including canonical-root portability refinement. All required implementation checks passed before synchronization.

openspec-sync-specs: created openspec/specs/artifact-resources/spec.md from the approved ADDED delta; existing living specifications are unchanged. openspec-archive-change: all artifacts/tasks complete and delta synchronized; archive only this verified change to 2026-10-03-artifact-resource-defaults under the delegated authorization. Final protected/strict gates run after archival; outcomes are appended before commit.

Post-archive protected Moon gate: exit 0, 37 strict specifications and all policy checks pass (issue74-moon-after.log). Separate strict all-spec validation: exit 0, 37/37 (issue74-spec-after.log). Only this change was archived, and its living requirements are synchronized. Implementation verification and local merge-readiness gates are complete; external draft delivery and exact-head CI remain separate evidence.

## Portability follow-up

Independent review found literal realpath equality would reject macOS trusted parent aliases such as /var to /private/var. The reader now lstat-checks the configured root, traverses from its canonical real path, rejects descendant links and verifies the opened file identity and exact canonical target. O_NOFOLLOW is used where available; fallback checks fail closed. The added alias regression verifies successful owned retrieval and rejection of an escaping descendant junction. Directory-link fixtures use junctions for Windows portability. Specification clarification and implementation re-review are approved in approval.md.

Affected checks passed again: typecheck and lint; 460 unit cases (one existing optional skip); full native contract gate plus 390 TS cases; 15 source integrations; 9 packaged smoke workflows. Evidence is issue74-portability-{types,lint,artifact,unit,contracts,integration,packaged}.log. The first full unit rerun hit the unchanged 5-second catalog expansion timeout during concurrent validation; the sequential rerun passed unchanged. Existing Rust workspace/Python evidence remains applicable because their implementation and inputs are unchanged. Protected OpenSpec validation passed after the clarification and is rerun after this report update.

## Main reconciliation and compatibility verification

Conflict repair uses ordinary merge of main 5e6472a110bc15dc699d47252533bec337d5d16e (merged PR #135 motion pack/schema 30) into source artifact head e4e3e4499f8d9ee4e9123e3f2fb42e279b98d634. Both capability and ownership entries are retained; the derived MCP digest follows semantic union. Independent Sol medium specification and implementation reviews approved the resolution and docs/contract-integration.md. Publishing scope remains artifact delivery plus the requested coordination documentation; no pending speech/animation PR is transplanted.

Reconciliation checks pass: Rust formatting, strict Clippy, 889 workspace tests (9 existing ignored), bridge typecheck/lint, 462 unit cases (one existing optional skip), native contract gate plus 390 TS cases, and 15 source workflows. Logs: issue74-reconcile-{fmt,clippy,rust,types,lint,unit,contracts,integration}.log.

The required local packaged gate remains an inherited executor timing failure: both complete reconciliation runs passed 8/9 tests but the unchanged group/component workflow exceeded 30 seconds. An independent full packaged run at untouched main 5e6472a1 reproduces the identical 8/9 timeout; its 33.015-second pause after status, near client.listTools, has no headless request in flight. Reconciliation pauses were 28.777/28.696 seconds in the same region. Test/timeout/CI configuration and production behavior remain unchanged. Logs issue74-reconcile-packaged-{first,second}.log and issue74-main-packaged-baseline.log preserve failure evidence. No local packaged pass is claimed for this reconciliation. The published exact-head CI packaged job must provide separate terminal evidence; slow baseline tools-list/schema handling is a follow-up concern rather than a fabricated green gate.

Disposable combined workspace /workspace/issue74-combined uses current main 5e6472a1 plus the reconciled artifact delta and corrected speech PR #137 head 6a835ac3f79c66a77c07f066ed673a6becf16b19. Only its derived MCP digest conflicted; semantic union computes 18919b8ca8ebaddb3a2f20f942b6dda91dd5251a9bae0964c7a062f618465a49. Both independent reviews find separate artifact/status fields and no semantic collision. Combined automated gate is running; results will be appended, and none of its speech changes enter PR #138.

The combined complete gate passed (native suites plus 391 TS cases), along with 58 focused speech cases and hermetic Python 12 Kokoro+5 transcription cases (issue74-combined-{contracts,speech,python}.log). PR #137 then published its main reconciliation at 584df7ed516274e32bc1acaaafa9691ea86aa9be. A fresh disposable union using that exact head has zero changed governed/check inputs by SHA256 manifest (all apps/crates/contracts/scripts/.github/.moon/.codex and build configuration), and the same 18919b8c full digest above. Thus full native/Python evidence remains applicable to the identical tree. Fresh current-head type and focused combined contracts/artifact/speech checks run separately; equivalence evidence is issue74-combined-current-equivalence.log. This avoids redundant completed native checks while verifying current exact heads.

Fresh current speech-head typecheck and all 113 focused contracts/artifact/tts/speech cases pass (issue74-combined-current-{types,focused}.log). Implementation_review independently inspected the refreshed union and 434-entry hash manifest: approved with no semantic compatibility blocker, subject to final protected/public gates and honest baseline packaged reporting.

Final protected Moon gate passes; separate strict validation passes 38/38 specifications (issue74-reconcile-{moon,spec}.log). Immediately before publication, fetched main still equals reviewed merge parent 5e6472a110bc15dc699d47252533bec337d5d16e. The artifact/motion semantic-union digest is b9bac4938bec7693a3c9a2f01b1002795c1a29f21ee7cd9c21ce7476607ef015. This resolution remains a draft candidate with an independently demonstrated local baseline packaged timeout; exact published-head CI must settle the remote required checks.
