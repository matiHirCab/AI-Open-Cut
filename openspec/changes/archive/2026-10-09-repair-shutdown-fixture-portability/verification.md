# Shutdown fixture portability verification

Base repair head: `571a4246964d6c173000ca8702d72611f1eff365`, draft PR #167.

## Evidence and scope
Windows run 37977760627/job 113980235575 failed before requesting SIGTERM: readiness saw a failed job. Its stderr was discarded and job error omitted; the underlying error is therefore not recoverable from that log. Do not call it a demonstrated provider shutdown regression. The old fixture launched a .cmd file through the direct headless spawn boundary. Node documents that .cmd/.bat require a shell (https://nodejs.org/api/child_process.html). The identical probe is now compiled to a native executable, retaining production direct launch semantics.

A separate test defect was established from Node's documented Windows process.kill behavior: SIGTERM unconditionally terminates a Windows target (https://nodejs.org/api/process.html), so that call cannot test the bridge's graceful handler. The test now owns actual child streams and uses the SDK's public JSON-RPC stream transport; Windows exercises the existing stdin-end shutdown entry, and POSIX retains SIGTERM plus adds stdin EOF. No provider, bridge or domain production code changed.

## Requirement/scenario matrix
| Scenario | Evidence |
|---|---|
| Native launch and active alignment | Compiled unchanged probe; real bridge job remains non-failed and real Python worker writes its PID before shutdown |
| Pre-shutdown failure diagnostics | Original readiness assertion retained, now includes structured status and last 16KiB bridge stderr, captured from process start |
| Supported platform entry and cleanup | SIGTERM and stdin EOF both pass locally on Linux; actual bridge and 60s-sleeping worker are gone within the original 3s limit, before original 10s inference timeout |

No readiness or cleanup assertion/deadline was weakened. The Windows SIGTERM variant is replaced by a supported graceful entry, not skipped; Linux gains an independently exercised EOF variant.

## Checks and limits
Focused final shutdown: 2/2, 1.56s. Typecheck and lint passed. Complete source integration: 33/33, 66.34s. Compiled packaged smoke: 30/30, 36.63s. Final full unit recheck completed successfully: 755 passed, 11 existing gated skips, 140.92s; no failures. Initial unit launch started before the local target symlink was recreated: 55 tests failed with headless ENOENT; its log is retained. Rerun uses the existing native executable explicitly after host setup. Earlier original repair Rust/Python/render results remain evidence for unchanged inputs; these are test/spec-only edits.

Pre-archive protected gate passed its 505 policy tests and strict 59-item validation; final rejection names only this active change. Conformance verified: all three tasks, one requirement and three scenarios trace to the scoped test changes; no critical mismatch remains. The accepted requirement was synchronized and the change archived. Final protected Moon gate passed: 505 policy tests, strict validation of all 58 living specs, and CI parity policy. Push follows these completed local checks. Windows and macOS execution for this follow-up are unverified until exact-head CI completes. Local logs remain outside the repository.
