## OpenSpec verify-change audit

The user approved the proposal, design, both delta specs, and tasks before implementation. The approved weekly cadence is Monday 03:17 UTC. No public or persisted contract, golden reference, tolerance, scene rule, or renderer output was changed.

| Dimension | Result |
| --- | --- |
| Completeness | 9 of 11 tasks complete before sync/archive; the two remaining tasks cover archival, the postarchive gate, and the final PR run. All eight modified requirements and their scenarios map to implementation and tests. |
| Correctness | The PR selector checks all five lifecycle semantics and renders 25/25/6 operations at 960/1280/1920. The separate scheduled definition selects the unchanged full 1920 path. Policy mutation tests enforce both definitions and the duration budget. |
| Coherence | One native conformance harness retains the reviewed fixture and assertions. The new workflow and validator stay in repository validation; no dependency edge or contract changed. |

No unresolved implementation, scenario, or design mismatch was found. The first execution of the new weekly workflow remains pending until it reaches the default branch; structural policy acceptance is not runtime evidence for that future run.

## Required check evidence

- `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`: exit 0. Full local logs: `target/rules-split-cargo-fmt.log`, `target/rules-split-clippy.log`, `target/rules-split-workspace-tests.log`.
- The selector/count unit test passed 1/1 (`target/rules-split-selector-test.log`). An initial invocation with an incomplete exact test name selected zero tests; the corrected invocation is the passing evidence.
- Policy and duration tests: exit 0, 377 passed (`target/rules-split-policy-tests-final.log`). The initial run failed because the isolated bootstrap fixture lacked the new scheduled workflow file (`target/rules-split-policy-tests.log`); the fixture was corrected and the suite rerun.
- Agent bridge typecheck, lint, unit tests (418 passed, one skipped), contract check, MCP integration (12 passed), and packaged smoke (7 passed): exit 0. Full logs: `target/rules-split-bridge-typecheck.log`, `target/rules-split-bridge-lint.log`, `target/rules-split-bridge-test.log`, `target/rules-split-contracts.log`, `target/rules-split-integration.log`, `target/rules-split-smoke.log`. No provider worker code or protocol changed, so a separate local Python worker rerun was outside this change's affected scope; the PR correctness jobs cover it.
- Strict all-spec OpenSpec validation before archive: exit 0, 30 items passed (`target/rules-split-openspec-strict-prearchive.log`). The direct policy validator and pinned Moon gate exited 1 solely for the expected active-change boundary (`target/rules-split-policy-prearchive.log`, `target/rules-split-moon-prearchive.log`).
- [PR CI run 36440359545](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36440359545) passed contract, original render, all three correctness platforms, packaged smoke, and all three rules-screen jobs. The 960 and 1280 logs each contain 25 named render operations; the 1920 log contains exactly six, plus state totals for undone, redone, and reopened. Full downloaded logs: `target/rules-split-ci-960-pr.log`, `target/rules-split-ci-1280-pr.log`, `target/rules-split-ci-1920-pr.log`. The 1920/960/1280 native test commands each selected one test and passed. The interim OpenSpec job failed only because this change was still active (`target/rules-split-ci-open-spec-prearchive.log`); its foundation job measured 90.76 minutes against the restored 120-minute budget and failed only on the missing OpenSpec attestation (`target/rules-split-ci-foundation-prearchive.log`).
- A [previous successful full main-branch run](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36252986065) passed the unchanged 1920 five-state, 25-render path in about 192 minutes. This is retained baseline evidence, not a claim that the new weekly workflow has run.

Local Windows FFmpeg 6 native attempts were stopped after the first 960 preview took 513699 ms and the first 1920 preview took 948374 ms; the complete required Linux CI shards above are the conformance evidence. Attempt logs are `target/rules-split-native-960-pr.log`, `target/rules-split-native-1280-pr.log`, and `target/rules-split-native-1920-pr.log`. No assertion failure appeared before interruption.

## Finalization

The three render-regression-fixtures requirements and five repository-validation requirements were synchronized into their living specs, preserving unrelated requirements. The change was archived at `openspec/changes/archive/2026-09-28-split-rules-screen-pr-and-scheduled-parity/`. Postarchive strict all-spec validation passed 29/29 (`target/rules-split-openspec-strict-postarchive.log`); the protected local Moon gate passed with 29/29 specs and valid CI policy (`target/rules-split-moon-postarchive.log`); direct policy validation passed (`target/rules-split-policy-postarchive.log`). The final PR protected aggregate must pass on the archived commit before merge readiness. The first actual weekly or manual full 1920 run cannot occur until the workflow file is on the default branch.
