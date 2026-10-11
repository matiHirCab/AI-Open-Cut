# Verification: define-ephemeral-job-restarts

## Scope and approval

Issue #85 has no subsequent comments. Dependencies #15 and #73 are closed and present in main `431a5e38bba964d780cfdbe0fe4ddf287ea21c58`, re-fetched before delivery. The initial checkout was clean. Proposal, requirements, design and tasks were approved under the explicit issue-scoped standing authorization before implementation. Separate desktop/video files and runtimes are outside this checkout and were untouched.

## Completeness and correctness

| Requirement/scenario | Implementation and automated evidence |
| --- | --- |
| Discover additive restart semantics | `contracts/job-restart-v1.json`, MCP capability catalog/ownership, `src/job-runtime.ts`, editor status and CLI health; canonical contracts/architecture tests and source/packaged restart workflow assert bridge-only discovery. Independently captured predecessor hashes and exact addition rollback preserve old schema/catalog pins; negative controls reject unrelated changes. |
| Restart every job state | `job-crash-process.ts` is killed as a real child process with pending running producers plus completed, failed and cancelled jobs. Existing real-provider FIFO tests separately verify provider queue behavior; no synthetic snapshot is presented as real provider-queue evidence. A separate admitted-queued fixture kills itself at the existing logging boundary before dispatch. Fresh processes return JOB_NOT_FOUND for every prior ID; source/packaged MCP additionally test old status, cancellation and artifact resources. No registry or token is persisted. |
| Reconcile uncertain commit before retry | Source/packaged restart workflow performs real generated-media insertion, kills the bridge, then reads identical authoritative committed state with an advanced revision in the successor while its old job is missing. `docs/job-restart.md` documents state/history/export reconciliation and avoids replay/exactly-once promises. |
| Repeated process crashes fail closed | Producer admission in `JobRegistry` acquires the exclusive guard before invocation. Three actual successor processes fail before any producer output; marker inventory/nonce and predecessor private bytes remain identical. |
| Graceful cleanup permits fresh work | Independent guard/registry test closes and releases the owner then admits a fresh guard with new IDs. Existing cancellation/retention tests remain applicable. Provider close waits for worker retirement and rejects remaining cleanup debt before guard release. |
| Unsafe or failed cleanup preserves ownership | Tests refuse foreign/replaced guards and Unix symlink roots/markers without deleting foreign targets; failed preview disposal retains guard until successful retry. A real signal-refusing worker exposed a cancelled late-write leak before the fix; final cleanup now waits for confirmed retirement and the regression asserts no WAV remains. Actual provider output is replaced with a directory to induce cleanup failure; close reports safe VALIDATION_FAILED and retries after repair while synthesis stays closed. Speech-service close also retries provider debt. Doctor fixture deletion failure rejects with safe VALIDATION_FAILED rather than claiming successful cleanup. Source/packaged doctor rejects an occupied crash guard without leaking paths or removing it. |
| Preserve transport and private runtime isolation | Distinct roots/runtimes generate independent IDs and reject predecessor lookups. Existing HTTP authentication/transport tests run unchanged. Shared authorized HTTP runtime access is documented as sharing rather than a per-project ACL. |

## Coherence

The implementation follows the ephemeral decision and conservative recovery trade-off. The guard stores a nonce only, never reconstructs a job, adopts an artifact or sweeps a shared directory. Existing UUID/revision/TTL/cancellation/error contracts and inward dependency boundaries remain unchanged. No Rust production, project migration, provider wire schema or headless capability changes are introduced. Provider and doctor lifecycle corrections make successful cleanup truthful; provider debt can be retried while admission remains closed. These are required before releasing crash admission.

## Check evidence

All required implementation checks passed before synchronization/archival. Full outputs remain uncommitted under `/tmp/issue85-logs`.

| Check | Result and full log |
| --- | --- |
| `cargo fmt --check --all` | Exit 0, `rust-fmt.log`. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0, `clippy.log`. |
| `cargo test --workspace` | Exit 0, 1516 passing tests across 77 result sections; 12 existing opt-in ignored cases, `rust-tests.log`. Rust implementation/test inputs remained unchanged by the bridge-only final lifecycle corrections. |
| Bridge `bun run typecheck`, `bun run lint` | Exit 0, `typecheck-final18.log`, `lint-final18.log`. |
| Bridge `bun run test --no-file-parallelism` | Exit 0, 66 files/883 tests; existing optional opt-ins skip 10 files/15 tests, `unit-final17.log`. A later test-only clarification removes a fabricated provider-queue snapshot; all 9 affected restart tests pass again in `restart-final18.log`, without changing production code or other unit inputs. |
| Bridge `bun run contracts:check --no-file-parallelism` | Exit 0, every prescribed Rust parity section and 38 TypeScript files/608 tests, `contracts-final17.log`. The protected package command is unchanged; the CLI flag serializes its final Vitest phase. |
| Bridge `bun run test:integration` | Exit 0, 37 source MCP tests, `integration-final17.log`. |
| Bridge `bun run test:smoke` | Exit 0, assembled/verified executable package and 34 MCP tests, `packaged-final17.log`. |
| `bun run apps/agent-bridge/scripts/run-python-tests.ts` | Exit 0, 12 unittest and 12 pytest cases, `python.log`; production worker/protocol inputs are unchanged. The modified fake-worker fixture is also exercised by the late-write regression and source/package suites. |
| Focused final lifecycle/conformance | 76 provider/speech/restart tests pass (`focused-final14.log`); 13 doctor/restart tests pass (`doctor-final16.log`); 11 exact predecessor/restart tests pass (`projection-final16.log`); 9 clarified crash/restart tests pass (`restart-final18.log`). The signal-refusing-worker regression failed with a leftover WAV before its fix (`cancel-race-negative.log`) and passes afterward. |
| Strict all-spec and protected pre-archive policy | Strict validation passes all items. The protected gate rejects only this active change, expected before archive and not claimed as gate success; final logs `spec-prearchive-delivery.log`, `prearchive-delivery.log`. |

Initial broad runs exposed additive capability/work-directory assumptions, predecessor projection integration and incorrect new CLI-health assertions; these were corrected without rewriting historical digests or weakening canonical commands. Contended runs also exceeded unchanged native polling/startup deadlines. One intermediate run read updated live ownership bytes against an earlier imported addition fixture while the final scope update was in progress; the unchanged fresh-process predecessor rerun and complete stable-tree rerun pass. Final bridge suites ran serially. This Linux container does not reap adopted grandchildren, so an external uncommitted subreaper wrapper supplied normal orphan reaping while preserving child-liveness assertions and timeouts.

## Verification assessment

Completeness: 7/7 implementation tasks, 2/2 requirements and 7/7 scenarios covered. Correctness: requirement/scenario mappings above include real process death, authoritative durable-state reconciliation, repeated blocked admission and cleanup failure/late-write negative controls. Coherence: ephemeral storage, conservative private ownership, existing contracts and layer boundaries agree across approved artifacts, code and guidance. No critical mismatch, uncovered normative scenario or undocumented design divergence remains. The final pre-archive validation confirmed that only this active change blocked the gate. The accepted delta is synchronized to `openspec/specs/job-restart-semantics/spec.md` and archived at `openspec/changes/archive/2026-10-11-define-ephemeral-job-restarts`.

## Limits and delivery

Local evidence is Linux x86_64 using pinned Bun 1.4.0, Rust 1.97.0 and Moon 2.3.3. Native development headers and caches are isolated under `/workspace/toolchain`; no machine configuration or repository policy was weakened. Optional real-model/provider/native suites retain their existing opt-in skip conditions. No controlled production long-job distribution, third-party retry telemetry or host-power-loss recovery is claimed. Windows/macOS verification and CODEOWNER review remain delivery requirements.

Protected post-archive `moon run root:openspec-validate` and strict all-spec validation passed with exit 0, 64 specification items and unchanged CI parity policy (`postarchive-gate.log`, `postarchive-strict.log`). Final policy is rerun before committing the report. At archival, commit/push, draft PR/review request and exact-head CI are the remaining external delivery steps. Their later evidence is recorded in the issue-linked draft PR and external checkpoint rather than claiming success before execution. No merge, auto-merge or deployment is authorized.
