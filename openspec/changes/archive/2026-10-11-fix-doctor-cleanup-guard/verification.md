# Verification: fix-doctor-cleanup-guard

## Pre-archive conformance

| Dimension | Result |
| --- | --- |
| Completeness | 6/6 implementation and verification tasks complete; all planning artifacts done |
| Correctness | 1/1 changed requirements and 5/5 scenarios covered |
| Coherence | Implementation follows the approved completion, cleanup and ownership design |

No critical, warning or suggestion findings remain in implementation conformance. This report verifies the implementation before synchronization and archival. The archived tree must subsequently pass the unchanged protected gate; remote CI and review are reported in the draft PR.

## Requirement and scenario evidence

The requirement **Doctor readiness is independent from cleanup ownership** is implemented at `apps/agent-bridge/src/index.ts:53`: diagnostic completion is tracked separately from readiness, and guard release remains after all three successful service closes. Report readiness and unsuccessful exit status remain unchanged. Guard identity/admission code is unchanged.

Test paths below are relative to `apps/agent-bridge/`.

| Scenario | Evidence |
| --- | --- |
| Complete unsuccessful diagnostics with clean shutdown | `tests/doctor-cli.test.ts:88` runs the real source CLI with unavailable dependencies, verifies unsuccessful diagnostics, admits a fresh real registry task and repeats doctor. The controlled unready case and ordinary directory failure cover other clean negative reports. |
| Complete successful diagnostics with clean shutdown | `tests/doctor-cli.test.ts:105` runs the actual CLI/guard boundary with a controlled ready report, verifies exit 0 and fresh admission. |
| Preserve ownership after diagnostic cleanup failure | `tests/doctor-cli.test.ts:150` injects actual filesystem probe-deletion failures, including a later cleanup failure after an earlier ordinary directory failure. The thrown doctor case at line 131 and existing `tests/doctor.test.ts:174` cover fixture cleanup. |
| Preserve ownership after service cleanup failure | The three close-failure cases at `tests/doctor-cli.test.ts:131` preserve the nonce and reject fresh acquisition after headless, speech or transcription close fails. |
| Preserve a predecessor guard | `tests/doctor-cli.test.ts:170` verifies refusal without changing the foreign marker or private bytes and without exposing the private root in stderr. Existing restart/ownership tests remain passing. |

`apps/agent-bridge/src/doctor.ts:39` preserves safe `VALIDATION_FAILED` for owned artifact cleanup failure. The private writeability helper at line 86 settles every probe before evaluating errors, so ordinary directory unavailability cannot hide cleanup debt. Transcription fixture cleanup uses the same helper at line 174. No versioned contract, schema, provider protocol, capability, project revision, marker format or crash-reconciliation code changes.

## Validation evidence

All commands below completed with exit 0 on the final relevant inputs unless explicitly identified as the expected pre-archive rejection. Full logs remain uncommitted outside the repository in `/tmp/doctor-fix-logs/`.

| Check | Result | Log |
| --- | --- | --- |
| New regression before production fix | 2 expected failures, 6 passes in the initial eight-case CLI suite; both failures reproduced clean unready retention | `regression-before.log` |
| Bridge typecheck | Passed | `typecheck-complete.log` |
| Bridge lint | 231 files checked; no fixes/suppressions | `lint-complete.log` |
| Focused doctor/restart lifecycle | 24 passed, including 11 new CLI cases | `focused-complete.log` |
| Full bridge unit suite | 894 passed; 15 existing opt-in native cases skipped | `unit-complete.log` |
| Rust workspace formatting | Passed | `rust-fmt.log` |
| Strict workspace Clippy, all targets | Passed with `-D warnings`; third-party future-incompatibility notice only | `clippy.log` |
| `cargo test --workspace` | 1,516 passed; 12 existing ignored helper/explicit native-evidence cases | `workspace-serial.log` |
| Contract checks | 460 Rust parity tests and 608 TypeScript tests passed | `contracts-serial.log` |
| Source MCP integration | 37 passed | `integration-serial.log` |
| Packaged smoke | 34 passed against freshly built package | `package-complete.log` |
| Hermetic Python workers | 12 unittest and 12 pytest cases passed | `python.log` |
| Pinned strict OpenSpec validation | 65 items passed, including this active change | `spec-final-prearchive.log` |
| Protected pre-archive bootstrap | Expected exit 1, naming only `fix-doctor-cleanup-guard` as unarchived; not reported as gate success | `protected-prearchive-final.log` |

Bun 1.4.0, Moon 2.3.3 and Rust 1.97.0 match repository pins. Native dependencies/tool caches were isolated outside tracked configuration. Earlier broad native attempts failed from the environment's 8.8 GB disk quota, including an explicit ENOSPC persistence assertion; they were not counted as passes. Rebuildable caches were pruned and the required suites rerun serially without changing tests or policy. The formerly failing narration/history fixture passed in the final workspace run.

## Limits

Default opt-in rendering/reference/platform cases and intentional Rust helper/capture cases retain their existing skip/ignore behavior; this bridge fix does not claim their specialized native evidence. Real model-backed inference/quality tests were not enabled. The regression exercises actual unready CLI diagnostics and real runtime admission; controlled reports and close failures supplement that boundary. No required local command is unavailable or failing.

## Pre-archive assessment

All required local implementation checks passed. Requirements, design, code, tests and documentation agree. Ready to synchronize and archive, followed by mandatory protected validation on the archived tree. No merge, deployment or private audit publication is part of this change.

## Archived-tree validation

The delta was synchronized into the living `job-restart-semantics` spec and this change archived on 2026-10-11. Strict validation passed all 64 living specs (`spec-final-archived.log`). The unchanged protected bootstrap invoked the actual pinned Moon root task successfully: 505 policy tests passed, 64 specs passed, and CI parity policy validation passed (`protected-final-ready.log`). Its initial Proto-offline setup failure was resolved by provisioning the pinned tools in isolated caches; repository policy/configuration was unchanged. All required local checks are complete. Remote CI and review remain distinct from local verification.
