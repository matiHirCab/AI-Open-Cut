# Verification: motion release gates (#79)

Implementation conformance reviewed on 2026-10-10. This is local Linux evidence; exact published-head Windows/macOS/Linux CI remains a required acceptance condition before #80. No merge or deployment is authorized.

## Scorecard and traceability

| Dimension | Result |
|---|---|
| Completeness | Seven implementation tasks covered; four added requirements plus the retained two native-reference scenarios covered |
| Correctness | R1–R11 and F4/F5 mapped below; local executed cases passed, remote execution explicitly pending |
| Coherence | Owning-layer test assertions, private collectors/orchestration, unchanged renderer/contracts/schema/provider behavior |

- R1/R2: `motion-release-cases.ts` closed eighteen exact descriptors across eight areas, its twelve guard tests, new owning-layer particle boundaries, actual selected canonical passes. Existing complete-reference tests retain atomic/stale/missing/invalid/history behavior.
- R3/R4, F4/F5: `motion_release.rs` unchanged recipe/support, all ten groups/67 operations/six cues, direct actual frame/range/export, independent gold and original voice/gap witnesses, altered-witness refusal, matching media, history/reopen, invalid time and missing media preservation. Existing source/compiled native cases run separately. `motion-native-cache-trace.ts` preserves the original recorded ELF/PE trace and checks strict Mach-O bypass; three negative-control tests. No smaller fixture replaces #77.
- R5/R6/R7: isolated release collector and `motion-release-report.ts`, one discarded warm-up/three measurements, complete build/platform/font/tool/fixture/nonce identities and strictly scoped Linux inclusive bounds. Five report tests exercise equality, one-unit excess per capture, nonfinite/absent/false fields, witnesses and metadata. Actual AAC duration and decoded stereo frame count enforce six seconds within one 10fps frame.
- R8: actual touched 32MiB child observed with at least 24MiB increase; readiness, zero-observation and early-exit controls; stop/join plus owned-child kill/wait. Exact isolated two controls executed in the driver; ordinary workspace also covers them.
- R9/R10: separate mandatory three-platform workflow; 34 workflow-policy/frozen-input tests, exact required execution and report guards. Actual zero-test and two-skipped native JSON reports refused. Original workflows, schema44/protocol1, recipe/gold media and #15 CURRENT frozen by hashes. Exact-head remote acceptance remains pending, not inferred from local evidence.
- R11: real descendant fixture deliberately detaches on Unix; four tests cover successful/failed parent, deadline and malformed/missing executable failures. Owned identity-checked Unix cleanup and suspended-before-Job-assignment Windows cleanup; Windows strict metadata compilation passed, actual Windows runtime awaits CI.

## Executed final local verification

All logs are uncommitted local files under `/tmp/mg79-*.log`; `.exit` records command status.

| Check | Result / log stem |
|---|---|
| Full Rust workspace | 1509 passing, zero failures, 12 configured ignores, 75 result blocks; `workspace-first` |
| Strict workspace Clippy, all targets/features | pass; `clippy-final` |
| Rust formatting | pass; `fmt-final` |
| Bridge units | 845 passing, 15 configured skips; `unit-final` |
| TypeScript types and lint | pass; `typecheck-duration`, `lint-final` |
| Scoped verification-script style | pass; `scripts-style-final` |
| MCP integration | 34 passing; `integration-final` |
| Default packaged smoke | 31 passing; `smoke-final` |
| Hermetic actual dependency setup Python | five passing; `python-setup-final` |
| Policy/preflight/real descendant controls | 45 passing; `root-controls-final` |
| Windows owner fixture metadata compilation | pass, runtime not claimed; `windows-owner-final` |
| Complete final native driver | 18 canonical, two collector, one default native and two source/compiled MCP cases; no required skips; `full-driver-final` |
| Strict prearchive validation | 62 passing; `prearchive-all-strict` |
| Protected prearchive validation | 505 policy tests and strict specs pass; rejects only this active change; `prearchive-protected`, expected exit1 |

The normal workspace run deliberately ignores the configured native workload and nested allocation helper. Their mandatory exact release invocations pass with zero ignored required cases. Final native execution includes the later AAC duration/decoded frame checks and isolated collector lock. Source identity honestly records dirty7057cde4 checkout; these local results are not exact published-head CI evidence. Only specification/evidence archival follows the tested implementation.

No governed contract/schema/provider implementation changed. Existing #77/#78 same-input protocol/provider/migration/GUI evidence is reused explicitly; complete workspace, bridge units/integration and default packaged smoke were freshly executed. No desktop presentation changed. Local Python3.12.14 differs from pinned CI3.11; remote Python setup is mandatory.

## Genuine final native measurements

Evidence directory: `/tmp/mg79-final-native-evidence/a720c567-0bdf-4c6b-b93b-caef9d833ca9`. Report SHA256 `5c93a2a060696f746828b9c57b4098e2e7f1d05339329feb4abb9ef29c842fb1`; accepted summary SHA256 `434bf8a57b07043b5cf384be3f1d1bec7684cde55ebff36f7ec859e2a9fda660`. Artifact bytes and full logs remain outside Git; workflow uploads the actual evidence at each published head.

| Capture | Whole elapsed ms | Maximum sampled tree RSS bytes | Samples | Cold/warm final FFmpeg calls | SSIM | PCM RMS |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 17128 | 127434752 | 2061 | 1/0 | 1.0 | 0.0 |
| 2 | 15573 | 129372160 | 1920 | 1/0 | 1.0 | 0.0 |
| 3 | 17079 | 131735552 | 2165 | 1/0 | 1.0 | 0.0 |

Actual native descendant count is positive in every capture; alignment offset zero. Stock FFmpeg/FFprobe, fixed four-font identities and actual H264 60-frame192x10810fps/AAC48k stereo six-second media are in the raw report. Sampling sleeps five milliseconds between refreshes; these observations are sampled RSS, not unsampled true peaks or guaranteed five-millisecond complete cycles. The coarse envelope applies only to this Linux x86_64 default release workload. #15 schema3/stage-definition2 remains unchanged and report-only.

## Review and limitations

No unresolved implementation/specification mismatch, critical issue or local-check failure remains. Failed exploratory harness attempts were retained, including incorrect asset lookup, cache-ineligible forwarding proxy, missing sysinfo command refresh and incorrect missing-media expected code; corrected tests now follow existing authoritative behavior without renderer changes. The initial private-trace guard omitted mandatory requestId; fixed closed shape preserves distinct request identity and original recorded counters.

External main08c8ea27 Windows target_measurement/shutdown fixture-readiness failure is inherited byte-identically on this branch; it does not establish a descendant leak. Its 1500ms readiness and 5000ms media assertions remain unchanged. Assess separately using `/workspace/mg79-regression-assessment.md`; exact-head Windows CI must pass, and repeat failure blocks advancement. The Windows Moon dual-output declaration concern remains separate; direct builds are not Moon task evidence. Neither assessment expands this issue's implementation.

Postarchive protected validation passed (505 policy assertions/tests and the protected gate, exit0; `/tmp/mg79-postarchive-protected.log`). Pinned strict all-spec validation passed62/62, exit0 (`/tmp/mg79-postarchive-all-strict.log`). `git diff --check` passed. All three native platforms and all prior/focused checks at the new exact head are mandatory before considering #79 complete or starting #80.
