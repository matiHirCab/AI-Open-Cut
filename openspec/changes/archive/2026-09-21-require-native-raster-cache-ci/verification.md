# Verification: require-native-raster-cache-ci

Status: complete. All 13 tasks are complete; conformance, synchronization, archival and final protected validation passed.

## Approval and scope

The user approved the proposal, design, both delta specs and tasks with "yes" on 2026-09-21 before implementation. Only CI workflow/policy/tests and documentation change. Existing issue-36 implementation and both archives remain preserved. No contracts, migrations, dependencies or renderer behavior changed.

## Traceability

| Scenario | Implementation and evidence |
| --- | --- |
| C1 / P1 | Expanded required Render parity sequence; native core cache, feature-enabled worker, real native bridge tests; positive complete-workflow policy test |
| C2 / P2 | Exact five-variable cache environment and instrumented build; mutation tests remove/disable every native prerequisite, flag, feature and command, alter dependency setup/workspace; existing native suites fail on absent required tools or stats |
| C3 / P3 | Default headless rebuild then default package tests; policy regressions reject absent/reordered restoration and masked/conditional/custom-shell commands; separate packaged job remains default-only |

## Environment and reused evidence

Pinned Bun 1.4.0, Rust 1.97.0 and Moon 2.3.3, as verified in the immediately preceding independent review. Windows native runs use local FFmpeg/FFprobe 7.1.1 and reviewed `crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf` (SHA-256 ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280). No runtime inputs or toolchain changed during this CI-only correction.

Unchanged-input evidence inspected in that review remains applicable: workspace strict Clippy (`opencut-worker-clippy-default2.log`, `opencut-worker-clippy-final3.log`), serial workspace Rust (`opencut-worker-workspace-serial.log` with affected final worker/core reruns), bridge typecheck (`opencut-worker-typecheck-final.log` and contracts), lint (`opencut-worker-lint-final2.log`), serial unit suite (`opencut-worker-unit-serial.log`, 412 passed and one separately executed native opt-in test), contract parity (`opencut-worker-contracts-final.log`), MCP integration (11, `opencut-worker-integration-final.log`), packaged smoke (6, `opencut-worker-smoke-final.log`), and hermetic Python (10 unittest plus 5 pytest, `opencut-worker-python.log`). These commands exclude the changed root policy scripts; those scripts receive their complete own policy suite below. No source changes invalidate the reused runtime results. Historical concurrent TTS cancellation timeout remains recorded in the previous archive, not erased by serial success.

## Fresh checks

Full logs are external under `C:/Users/matia/AppData/Local/Temp/`, prefixed `opencut-cache-ci-`. Fresh results are recorded below. Initial focused policy test: 275 passed, exit 0 (`policy-focused.log`).

## Limits

No remote workflow was dispatched; local Windows native success is not a claim of Linux CI execution or POSIX process-lifetime validation. The existing real-Moon BASH_ENV integration regression returns early on Windows; this host cannot execute its POSIX branch. Required Linux CI still invokes that unchanged regression. This platform limitation must remain explicit even if Bun reports the test as passed. Golden references and assertions remain unchanged.

| Fresh command/check | Result | Log suffix |
| --- | --- | --- |
| Complete protected policy suite (three files) | exit 0, 283 passed; POSIX-only integration branch returns early on Windows as noted | policy.log |
| `cargo test -p opencut-editor-core --lib raster_cach` | exit 0, 13 passed, native configuration required | core.log |
| Feature-enabled `render_worker` target | exit 0, 3 passed including actual raster reuse and Windows crash containment | worker.log |
| Instrumented headless build | exit 0 | build-hooks.log |
| Exact Bun run --cwd bridge command from design/workflow | exit 0, 1 actual native test passed; no skip | native-bridge.log |
| Default headless rebuild | exit 0 | build-default.log |
| Default `cargo test -p opencut-headless` | exit 0, 5 unit + 28 protocol + 2 worker passed | default-headless.log |
| Rust formatting and Git whitespace | exit 0 each | fmt.log, diff.log |
| Strict pinned all-spec validation | exit 0, 29 items | specs-prearchive.log |
| Prearchive protected Moon gate | expected exit 1 solely for active require-native-raster-cache-ci; policy tests and 29 specs pass | prearchive-gate.log |

Native checks ran serially with RUST_TEST_THREADS=1, required flags, the pinned Rust toolchain and unchanged native paths/font. No threshold, reference or assertion changed. No failed implementation check remains; the prearchive rejection is the mandated active-change boundary, not a final gate pass.

## OpenSpec conformance review

Applied openspec-verify-change using complete CLI status and apply context. Completeness: all thirteen tasks are complete, including synchronization, archival and the final protected gate. Correctness: both delta requirements and all fourteen scenarios (eight preserved render-policy scenarios, C1-C3 and P1-P3) map to the exact workflow/policy sequence, existing guard tests, 49 additional policy tests, and native results above. Coherence: approved serial command order and five-variable cache environment match design, with locked JS setup, default restoration before compatibility and unchanged report validation/upload. Existing ownership and public/persisted contracts remain unchanged. No unresolved implementation/spec mismatch was found. The known host-specific verification limit above is retained.

## Final archival and protected gate

Synchronized the render-regression-fixtures requirement and added the repository-validation requirement, preserving unrelated requirements and prior archives. Archived the full change with `.openspec.yaml` as `2026-09-21-require-native-raster-cache-ci`.

- `moon run root:openspec-validate`: exit 0, complete policy suite and 28/28 living specifications pass; `opencut-cache-ci-final-gate.log`.
- `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`: exit 0, 28 passed, zero failed; `opencut-cache-ci-final-specs.log`.

Both checks are repeated after this final task/evidence update using the same log paths. Remote Linux CI remains unexecuted locally; the POSIX-only integration branch limitation remains as stated above. No commit, push, pull request, golden update or assertion weakening occurred.
