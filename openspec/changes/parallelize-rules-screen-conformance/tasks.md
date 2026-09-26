## 1. Approved boundary and baseline

- [x] 1.1 Record the 205m01s and 336m50s optimized baselines, their 11,526.743s and 19,210.915s rules-screen suites, the 3 × 5 × 5 required matrix, and unchanged reviewed references/tolerances.
- [x] 1.2 Inspect completed run 36146054095: cancelled at about six hours with 74/75 operation records; 960x540 took 120.78m, 1280x720 took 202.62m, 1920x1080 lacked its final export. Peak process-tree memory and final report checks were unavailable because the native suite did not finish. Full log: `target/pr124-rules-timeout-ci.log` (uncommitted local evidence).
- [x] 1.3 Amend and strictly validate proposal, render and repository-validation delta specs, design, and tasks; obtain explicit approval of amended artifacts before implementation.

## 2. Native and CI implementation

- [x] 2.1 Replace same-runner resolution workers with one required, closed-resolution native test. Preserve five ordered states, 25 operations, comparisons, font/dependency failures, and no-mutation assertions per invocation. Keep other golden suites and report in the original test.
- [x] 2.2 Retain 75 labeled operation timings across three jobs. Use a 250 ms process-tree memory sampler for the long suite while preserving existing 5 ms behavior and tests. Add focused tests for accepted/rejected resolutions, case counts, and required-mode failure.
- [x] 2.3 Add exactly three protected Linux rules-screen matrix jobs and make foundation parity require their aggregate result. Keep existing Render parity native/cache/report checks. Update CI policy and adversarial tests for matrix completeness, zero-test detection, exact commands/environments, missing tools/font, ignored failures, report ownership, and four-result aggregation.

## 3. Verification

- [x] 3.1 Run focused native, CI-policy, formatting, strict Clippy, workspace, TypeScript, contract parity, MCP integration, packaged smoke, Python worker, and pinned strict OpenSpec checks; capture full logs and exits.
- [ ] 3.2 Run the pre-archive protected Moon gate, allowing only rejection of this active change. Push the candidate and require all three rules-screen shards, original Render parity, Windows/Ubuntu/macOS correctness, and contract/smoke checks to pass. Verify 45 preview, 15 range, 15 export records, per-resolution peak memory, unchanged report schema/reference checks, and protected workflow wall time below 120 minutes.

## 4. Completion

- [ ] 4.1 Run `$openspec-verify-change`, resolve mismatches, synchronize both deltas, and archive only after candidate checks and time target pass. If 1920x1080 prevents the target, stop and seek separately approved renderer optimization without reducing evidence.
- [ ] 4.2 Run post-archive protected Moon and pinned strict all-spec validation; require final protected workflow and foundation gate to pass below 120 minutes before declaring PR #124 ready.
