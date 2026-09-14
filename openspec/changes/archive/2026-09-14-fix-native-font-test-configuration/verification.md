# Verification report

Status: all implementation checks and conformance verification passed. Synchronization, archival and final protected gates follow.

## Approval and failing evidence

User approved the harness proposal and explicitly approved adding native font CI coverage on 2026-09-14. CI run 34871014552 at 94c99d6865328f89cbc8606f4995af6fcc6aa6f7 failed both native font tests on all correctness platforms and contract parity because FFmpeg was absent. Its passing render job did not run these tests. Previous local verification configured tools globally and missed unconfigured execution.

The configuration regression was added before its helper: opencut-ci-red.log records exit 101 (missing helper). Three workflow mutation tests were added before the command extension: opencut-ci-policy-red.log records 223 pass / 3 fail. Logs are uncommitted under the Windows user TEMP directory.

## Scenario traceability

| Scenario | Evidence | Result |
| --- | --- | --- |
| Unconfigured ordinary job | font_resolution suite, opencut-ci-unconfigured.log | 16 pass; two native bodies omit work |
| Required or partial configuration | native_font_configuration_is_explicit_and_required_mode_fails_closed covers all six partial combinations in both modes; opencut-ci-required-missing.log | Parser passes; required missing execution exits 101 with diagnostic |
| Unusable configured dependencies | opencut-ci-invalid-tool.log and opencut-ci-invalid-font.log | Both expected exit 101 with tool/font diagnostic |
| Configured native conformance | opencut-ci-native-7.log, opencut-ci-native-8.log | All 16 pass with FFmpeg 7.1.1 and 8.1.2; native assertions execute |
| Native font coverage cannot be bypassed | Three validate-ci-gates mutation cases; opencut-ci-policy.log | 234 policy/runner tests pass |

## Completeness, correctness and coherence

The test-local parser follows the existing Transform2D configuration policy. The early return applies only to entirely absent optional configuration. Readability and renderer readiness reject configured errors before project setup. Existing rendering assertions are unchanged. The workflow adds the suite under the existing required environment; exact-command enforcement and mutation tests prevent removing or masking it. Documentation lists the same commands. No production code, public contract or persisted format changed.

## Repository gates

Rust 1.97.0 formatting and strict workspace Clippy pass. TypeScript typecheck, lint and all 392 unit tests pass. Python passes 10 unittest and 5 pytest tests. Strict OpenSpec validation passes all 27 items. Logs: opencut-ci-fmt.log, opencut-ci-clippy.log, opencut-ci-typecheck.log, opencut-ci-lint.log, opencut-ci-unit.log, opencut-ci-python.log and opencut-ci-spec.log.

The first workspace run failed an unrelated process-memory sampler assertion (opencut-ci-workspace.log). Its isolated retry passes (opencut-ci-sampler-retry.log); full workspace rerun with RUST_TEST_THREADS=2 passed (opencut-ci-workspace-retry.log). The first configured native attempt referenced a removed temporary FFmpeg directory and failed readiness (opencut-ci-native.log); restored 7.1.1 passes.

Moon in the shared checkout rejects this active change and unrelated reduce-agent-context-overhead (opencut-ci-moon.log). Unrelated files remain untouched. Finalization will use a clean checkout of the scoped PR commit, subject to all implementation checks passing; no protected policy is weakened to clear the inventory gate.

Contract parity passed its governed Rust suites and 328 TypeScript tests (opencut-ci-contracts.log). MCP integration passed 11 tests (opencut-ci-integration.log); packaged smoke passed 6 (opencut-ci-smoke.log).

OpenSpec verification: completeness, correctness and coherence pass for both requirements and all five scenarios. Implementation tasks complete; finalization remains pending until archive and post-archive gates. Clean PR checkout 74160e19 pre-archive Moon validated all 26 items and rejected only this active change (opencut-ci-clean-prearchive.log). No unrelated failure blocks archival in the PR tree. Implementation inputs match the checked original checkout; passing implementation evidence is reused without rebuilding unchanged code.


Final local verification: specifications synchronized and change archived on 2026-09-14. Post-archive Moon passes and strict validation passes all 25 living specs (opencut-ci-postarchive.log, opencut-ci-postarchive-strict.log). All 9 tasks complete. Remote PR CI will be inspected after pushing; its results are not implied by local passes.
