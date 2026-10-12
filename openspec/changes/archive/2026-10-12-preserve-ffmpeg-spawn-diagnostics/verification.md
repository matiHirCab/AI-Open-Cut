# Issue178 conformance verification

Local implementation and independent resolution review are complete: 12/12 tasks, 5/5 requirements and 10/10 scenarios have traceable evidence. No critical or warning mismatch remains among code, approved design, requirements and tasks. Synchronization/archive, final protected gates and exact published-head hosted CI remain separate mandatory milestones. Human CODEOWNER review is pending.

## Requirement and scenario evidence

| Requirement / scenarios | Implementation and automated evidence |
| --- | --- |
| Safe startup metadata: missing/denied and native fallback/privacy | Core closed kind/platform/native-code constructor and final run_command capture only. Actual missing executable and Unix nonexecutable controls, canonical kinds/native codes, hostile custom-error privacy and ordinary-error absence pass. /tmp/issue178-focused-spawn.log; /tmp/issue178-actual-access-denied.log; full Rust/canonical receipts below. |
| Compatible typed forwarding: actual startup and non-startup errors | Shared ErrorBody::from covers standalone/status/worker; serde omission, strict bridge schemas, BridgeError/errorBody and failed jobs carry the field. Actual typed canonical headless producer, overlapping worker/one-shot failures, retained failed-job diagnostic, ordinary/readiness/pipe/child-exit compatibility and source/package transport witnesses pass. /tmp/issue178-headless-producer.log; /tmp/issue178-focused-contract-ipc.log; source/package receipts below. |
| Atomic safety: renderer intents and stale/invalid precedence | Actual renderer facades inject safe PermissionDenied; healthy execution controls and frame/range/draft/export rejection preserve complete project/history/draft/destination snapshots, remove temporary workspaces and publish no artifact/completion. Stale revision and invalid bounds win before execution. /tmp/issue178-cleanup-precedence.log; full Rust receipt. These are failure-adapter controls, distinct from successful native media evidence. |
| Governed additive contracts: malformed metadata and overlapping lanes | Manually authored canonical schema/fixtures and ownership, six optional MCP references, exact main84 raw predecessor pins and captured headless bytes agree with actual registration. Valid/invalid metadata and missing-field/unrelated-drift controls pass. The exact corrected PR197 union retains both declarations/constructors/transports/catalogs and passed full canonical checks plus fresh independent review. No historical pin, assertion or deadline was weakened. |
| Truthful platform evidence and causal limits | Documentation distinguishes actual kernel failures, injected OS-code adapters, transport fixtures and actual native media regressions. Linux evidence below is local precommit evidence. Windows/macOS actual observations remain required in exact published-head hosted CI. The original private project/media and installed-build commit are unavailable; the conditional controlled Windows root comparison cannot be automated with these inputs. No path-length causal claim is made. |

## Passing required local checks

| Check | Result / full log |
| --- | --- |
| Rust workspace | 1,524 passes, 12 existing configured ignores, 77 result blocks; /tmp/issue178-required-workspace.log |
| Strict workspace/all-target Clippy; fmt | Pass; /tmp/issue178-required-clippy.log; /tmp/issue178-final-fmt.log |
| Bridge typecheck / lint | Pass; 241 files; /tmp/issue178-final-typecheck.log; /tmp/issue178-final-lint.log |
| Complete bridge unit suite | 900 passes, 15 existing opt-in skips, 79 files; /tmp/issue178-required-unit-verified.log |
| Full unchanged canonical command | 461 Rust + 609 TS passes, all 38 TS files; /tmp/issue178-required-contracts-verified.log |
| Full source MCP / fresh packaged smoke | 39/39 and 36/36; /tmp/issue178-required-source.log; /tmp/issue178-required-packaged.log |
| Actual Linux source/default-package native renderer | 2/2, 75.46 seconds; /tmp/issue178-required-native-platform.log; artifacts /tmp/issue178-platform-native-evidence |
| Full motion release | Required isolated/sampler controls, optimized noninstrumented native scene and strict validator, then instrumented source/compiled MCP 2/2 pass; /tmp/issue178-required-motion-release.log; evidence /tmp/issue178-motion-release-evidence/95ea7590-193f-4687-b0f7-a37b0498f011 |
| Complete native golden | Pass, 306.13 seconds, unchanged references and independent illustrated pixels/label/range controls; /tmp/issue178-required-golden.log |
| Hermetic Python | 12 unittest + 12 pytest; /tmp/issue178-required-python.log |
| Strict pre-archive specs | All 66 items pass; /tmp/issue178-review-prearchive-strict.log |
| Protected pre-archive gate | Rejects solely this active change, as expected; /tmp/issue178-review-prearchive-protected.log |

Native release/platform uses reviewed font SHA256 7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954. Golden uses distinct reviewed ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280. Local native reports identify dirty main84 and tracked-diff/test-binary hashes truthfully, not a published head. Transport fixtures delegate non-render operations to real headless and emit controlled canonical render failure plus execution markers; no production test hook or launch bypass exists. Actual native success uses real media tools independently.

Earlier failed attempts remain documented in full logs: type/test setup, manual schema key order, owned predecessor composition, whole-schema restart comparison and a transport fixture that incorrectly failed get_state were corrected within scope. No production provider, budget, runtime or CI policy was changed. Final complete receipts above resolve these failures with all old cases and pins retained.

## Exact disposable union and independent review

Verified source snapshot edbc7b16eb4397f8dfa1c781cdfe04db3aa657fd (tree 75303a1320323655411111554914160536c9c749) is combined with corrected PR197 head 2eb81e58ecb4bfbcc173f7d2e94ba6955831a668 at c4cb4cfb450ca07d0bb15454e86d2966bf65eab1. Thirteen shared textual conflicts were resolved preserving both complete owner declarations, constructors, tests, thin forwarding, canonical categories/examples and six MCP references. Semantic normalization yields c5c5b957db893ee6606484cb453513a8d0f35c01f1bf2672fa1638297b720b39. All 30 publishing implementation/catalog/test/documentation paths match the verified snapshot; no union changes were copied into this branch.

Initial review found that the budget remover accepted all six properties missing with its definition retained. The parent explicitly authorized correction in owning PR197. Its definition-aware guard and direct/raw negative plus historical identity controls were pushed at the corrected peer head before rebuilding the union. A disposable-only combined entrypoint witness preserves all three predecessor pins. Fresh independent reviewer /root/review_issue178_corrected_union accepted both specification and implementation resolution at the exact corrected union, with no blocking findings. Acceptance is recorded separately in approval.md from pending human CODEOWNER review.

Union checks: 57 focused tests and 249-file lint pass; full unchanged canonical command passes 462 Rust + 610 TS across 38 files with typecheck (/tmp/issue178-corrected-union-contracts-serial.log). Local VITEST_MAX_WORKERS=1 reduces file-level contention while preserving every case/assertion/deadline and repository configuration. Prior full default-concurrency attempts each passed 462 Rust and 609 TS but had one existing timeout (queued transcription, then catalog expansion); logs /tmp/issue178-corrected-union-contracts.log and -retry.log remain. The affected transcription file separately passes 26/26. Source and catalog did not change between attempts.

## Remaining mandatory milestones and limits

Synchronize/archive only this verified capability and require final protected/strict gates before normal commit/push and an unmerged draft. Re-fetch main immediately before publication; if it advances, repeat reconciliation and affected review/checks. Require all 18 exact published-head hosted jobs and the unchanged duration audit to pass; local Linux runs are not hosted cross-platform evidence. Do not amend a verified published head merely to record later CI. Human CODEOWNER review remains pending; the designated owner is also the author, so do not repeat the rejected self-review request. No merge, auto-merge, deploy, credential change or desktop/video task is authorized.

## Final archived-tree gate receipt

Only this capability was synchronized and archived on 2026-10-12. Final protected Moon validation and strict all-spec validation pass, all 66 items (/tmp/issue178-final-protected.log; /tmp/issue178-final-strict.log). The 30 implementation/catalog/test/documentation paths remain byte-identical to the independently reviewed source snapshot. Publication and terminal exact-head hosted CI remain mandatory and pending.
