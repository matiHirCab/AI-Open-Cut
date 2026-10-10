# Verification: repair-preview-retention-and-confined-disposal

Status: implementation verified locally; mandatory post-archive gates and exact-published-head genuine platform CI remain external delivery acceptance. Approval was explicitly supplied by the delegated reviewer on 2026-10-10. No merge/deploy authorization.

## Requirement evidence

| Scenario | Implementation | Automated evidence |
| --- | --- | --- |
| L4 inclusive budgets | JobRegistry serialized retention, producer reservations | preview-lifecycle exact count/byte independent limits and oversized cases |
| L5 owned expiry/cancellation/close | charged ownership, close settlement, native UUID roles | preview-lifecycle TTL/late cancellation; core/private native and source/default package cleanup |
| L6 safe refusal | core handle-based traversal, adapter bounded/redacted failures | native links/foreign paths; preview-disposal malformed/oversized output, exit, timeout cases |
| L7 persistent failure | reserve before dispatch, serialize debt recovery and required eviction | original unchanged-main regression failed; persistent count-only and byte-only fault cases assert one producer; overlapping reservation case |
| L8 debt/close retry | disposalDebt preserved, close promise reset on failure without reopening | persistent failed/retried close; oversized debt blocked production then safe recovery |
| L9 ancestor replacement | Unix held-directory openat/unlinkat, Windows held handles and disposition | core deterministic root/project/previews replacement; actual source/default package simultaneous ancestor swap and outside victim preservation |
| L10 compatibility/concurrency | unchanged catalogs/revision semantics, cancelled reservations retained | lifecycle concurrency/cancel/TTL; full existing contracts/integration/native/package gates |
| P8 genuine existing runtime | private bounded headless flag delegates to same editor-core owner, no new role | Linux actual native source/default package 2/2; compiled packaged smoke 33/33; actual Windows/macOS execution remains required CI |
| P9 fail closed evidence | existing required platform report/gates unchanged, cleanup assertions mandatory in both modes | existing negative report/manifest controls; no mock substitutes for actual runtime acceptance |

## Completed local implementation evidence

- Original failing main sources/results and complete independent audit remain in /tmp/opencut-review-evidence and Library libfile_82a158f94dd881919e923819d4deae7a. Baseline new persistent-failure regression failed against unchanged main: /tmp/opencut-repair-baseline-bridge.log.
- `cargo fmt --check --all`: exit0; `/tmp/opencut-repair-format-final.log`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: final exit0; `/tmp/opencut-repair-clippy-large-volume.log`.
- `cargo test --workspace`: exit0, `/tmp/opencut-repair-workspace-large-volume.log`. Includes new Linux core confinement tests and actual private-headless input tests. Existing native opt-in ignores remain subject to actual native gates. The later added Windows-only integration file was separately checked on Linux (0 applicable tests) and cross-compiled with strict Windows Clippy; no existing Rust inputs changed after this passing workspace run.
- Windows GNU headless/all-core-test compilation and focused Windows-only strict Clippy: exit0, `/tmp/opencut-repair-windows-check.log`, `/tmp/opencut-repair-windows-tests-check.log`, `/tmp/opencut-repair-windows-specific-clippy.log`. This is not genuine Windows execution.
- `bun run typecheck` and `bun run lint`: exit0, final logs; lint221 files, zero warnings.
- `bun run test:unit --maxWorkers=2`: exit0;869 passed,15 configured skips; `/tmp/opencut-repair-unit-large-volume.log`. Deadlines and assertions unchanged.
- `bun run contracts:check`: exit0; canonical Rust checks,607 bridge tests and18 desktop tests; `/tmp/opencut-repair-contracts-large-volume.log`.
- `bun run test:integration`: final exit0,36/36; `/tmp/opencut-repair-integration-final.log`. Actual native confined cleanup, ancestor racing, failed close/retry included.
- `bun run test:smoke`: final exit0,33/33; `/tmp/opencut-repair-smoke-large-volume.log`. Real compiled four-role default package.
- `bun run apps/agent-bridge/scripts/run-platform-renderer.ts`: final exit0,2/2 actual Linux source/default package tests, real rendered media and unchanged tolerances; `/tmp/opencut-repair-platform-native-final.log`. Includes final ancestor-racing and failed close/retry helper.
- Relevant minimal providers:12 unittest and12 pytest pass; `/tmp/opencut-repair-python.log`. `python3 scripts/test_setup_platform_renderer.py`:5 pass; `python3 scripts/test_setup_motion_release_backends.py`:7 pass.
- Strict active-change validation passes. Pre-archive protected `moon run root:openspec-validate`:64 strict items and normalization/policy checks pass; exit1 solely because this active change is unarchived, `/tmp/opencut-repair-presync-final.log`. This is not final protected success.

## OpenSpec conformance scorecard

| Dimension | Result |
| --- | --- |
| Completeness |13/13 implementation tasks;2/2 requirements and9/9 scenarios traced to code/tests |
| Correctness | L4–L10 and P8–P9 mapped above; no missing behavior or changed public contract found |
| Coherence | Native I/O remains editor-core owned; bounded adapter remains private; four-role package/public catalogs retained; design followed |

No critical implementation mismatch or uncovered changed scenario found. Local implementation is ready for synchronization/archival. Actual Windows/macOS execution and terminal exact-head CI are explicitly mandatory post-archive acceptance, pending publication; supplemental cross-compilation is not substituted for them. Post-archive protected/strict gate results will be recorded externally against the final tree, avoiding a self-referential documentation commit.

## Honest limits and retained attempts

Already-produced oversized/cancelled outputs may remain physical cleanup debt when deletion itself fails. Ownership/accounting is retained and further production blocked; no impossible hard physical-byte guarantee is claimed under arbitrary failed I/O. In-flight producer count is bounded, but unknown encoded sizes still require completion-time byte admission. Cleanup confines traversal against ancestor replacement; opaque UUID names and process-local registry ownership remain the authority, not durable identity or predecessor-process reclamation.

Initial full TypeScript attempt: 813 passed, 71 configured skips, one suite failed because rustc was absent from subprocess PATH. With toolchain restored, 866 passed, 15 configured skips and 3 existing suites timed out under concurrent compilation. Final rerun uses 2 workers with unchanged test deadlines and passes. Further initial attempts exhausted the temporary8.8GiB volume and failed compiler/test/package writes; a separate32GiB workspace build volume resolved this without repository configuration changes. Failed attempts remain in the logs, superseded only by recorded successful relevant checks. Initial workspace test link failed for missing xcb/xkbcommon libraries; temporary extracted Debian native libraries supply linking without editing system paths or repository configuration. System installation could not run (sudo absent; apt system state not writable). Moon initial home/cache creation and proto offline detection failed; explicit temporary tool homes plus supported global toolchain selection run the unchanged gate. All logs retained in /tmp/opencut-repair-*.log.

Local pinned Rust1.97/Bun1.4/Moon2.3.3; Rust debug and incremental compiler storage disabled; build output and TMPDIR use a dedicated temporary workspace volume. Required default CI execution remains acceptance evidence. Temporary child subreaper reaps dead orphaned test processes because container PID1 does not; no assertions/deadlines are weakened. No public contract/catalog/schema/provider, renderer/golden, protected policy or workflow changes.

## Published-head Windows fixture correction

Initial published head2d07bc54 passed all three genuine native and complete-release workflows, but Windows correctness failed in `preview_disposal_windows.rs` before its final-reparse assertion: `cmd mklink` parsed a mixed-separator preview path as a switch. Full failure log: `/tmp/opencut-repair-ci-windows-correctness-failure.log`, job114221907846/run38055128862. The unrelated parent-junction test passed, and actual source/default-package final-link and ancestor tests passed on Windows. This is a newly added fixture construction defect, not evidence of a native deletion failure.

Under approved post-archive delivery task5.3 (repair scoped recoverable failures), the fixture rebuilds both mklink arguments from native `Path` components without lossy Unicode conversion. Production cleanup, assertions, scenarios and deadlines are unchanged. Formatting, Windows-focused strict cross-Clippy and the Linux-inapplicable test check are rerun; unchanged production/full-workspace/bridge/provider evidence is reused. Genuine Windows fixture execution and all terminal exact-final-head CI remain required against the corrected published head. This correction requires no new normative behavior or approval.
