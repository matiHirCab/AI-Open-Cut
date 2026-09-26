## Context

The [Windows correctness job in run 36026988708](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36026988708/job/107726086872) failed `process_tree_sampler_observes_a_child_allocation`. Its isolated child allocated 64 MiB and exited after a fixed 300 ms sleep. The sampler refreshes processes on a background thread; Windows did not observe the child before it exited. The nested helper's actual failed assertion was `with_child >= baseline + 32 MiB`. The complete job log is preserved locally at `target/pr124-windows-failure.log`. A focused local Windows run of `cargo test -p opencut-editor-core renderer::golden::process_tree_sampler_observes_a_child_allocation -- --exact --nocapture` passed (log: `target/pr124-sampler-local-repro.log`), confirming the fixture is timing sensitive rather than proving a production sampler regression.

## Goals / Non-Goals

**Goals:** Make the existing child-allocation test deterministic across supported CI platforms, preserve the 32 MiB observation threshold, give failures bounded deadlines and useful diagnostics, and leave no child process running.

**Non-Goals:** Change the production sampler, sampling interval or metric; relax golden tolerances or remove the test; change CI gate structure, contracts, schemas, or rendering output.

## Decisions

1. Use a per-test temporary directory as a cross-process handshake. The helper writes a complete readiness marker after allocating 64 MiB and waits for a release marker. The isolated parent waits for the complete marker before checking the sampler's peak. This avoids relying on a fixed child lifetime or test-harness stdout buffering. Merely extending the 300 ms sleep was rejected because slow or loaded Windows runners can still miss a fixed window.
2. Poll the sampler's existing atomic peak for at least the current `baseline + 32 MiB` threshold with a declared finite deadline. Once observed or timed out, write the release marker, wait for the child, finish the sampler, and assert the result. The helper also has its own finite release deadline so an interrupted parent cannot leave it waiting indefinitely. Polling the peak does not change sampling behavior; it only coordinates test teardown.
3. Report readiness, early exit, and observation timeout distinctly. Keep cleanup before assertions where possible so failure cannot strand the child. Use the current ignored child helper, rather than adding a production pathway or external dependency.

## Risks / Trade-offs

- [Filesystem marker visibility or partial writes] → Read the complete expected readiness content and use a dedicated per-test directory. Release is monotonic and idempotent.
- [Unexpectedly slow CI] → Use bounded deadlines substantially longer than the former 300 ms hold; report measured baseline and peak on failure. The helper's own deadline prevents an indefinite process.
- [Child exits before observation] → Detect early exit, release/reap, and fail without masking a sampler defect.

## Migration Plan

No migration or rollout is needed. The test-only fixture changes in the existing PR. Revert the fixture edit if cross-platform checks uncover a regression.

## Open Questions

None.

## Verification evidence

- Final Windows fixture passed 20/20 focused runs, including the successful allocation observation. Separate focused tests passed for observation timeout, early child exit, missing readiness, and direct ignored-helper execution. Logs are in ignored `target/pr124-sampler-*.log` files.
- `cargo fmt --check --all`, strict workspace Clippy, and `cargo test --workspace` with default parallelism passed after the final fixture edit.
- Bridge typecheck, lint, 415 unit tests (one existing skip), 349 contract parity tests, 12 MCP integration tests, six packaged smoke tests, and 10 unittest plus five pytest Python tests passed. The bridge unit suite timed out twice in the restricted local sandbox's worker-process tests but passed fully outside that sandbox; the passing run is `target/pr124-sampler-bridge-test-unsandboxed.log`.
- Pinned strict OpenSpec validation passed 30/30. The pre-archive Moon gate rejected only the expected active change `stabilize-process-tree-sampler-fixture`.
- Candidate CI [run 36032914723](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36032914723) passed Windows, Ubuntu, and macOS correctness, contract parity, packaged smoke, and required Render parity. The Windows log confirms the original sampler observation case and all new failure-path tests passed. CI OpenSpec validation rejected only this active change, as expected before archival. Render parity took 20,210 seconds; the unchanged `rules_screen` suite accounted for 19,211 seconds, versus 11,527 seconds in the previous optimized run. All golden assertions passed.

## Conformance verification

The one modified requirement has four scenarios. Normal completion maps to `process_tree_sampler_finish_signals_and_joins_worker`; unwind cleanup maps to `process_tree_sampler_drop_signals_and_joins_during_unwind`; held-child observation maps to `process_tree_sampler_observes_a_child_allocation`; and timeout, early exit, and missing readiness map to `process_tree_sampler_observation_timeout_reaps_child` and `process_tree_sampler_child_exit_and_readiness_timeout_are_bounded`. All ran in the workspace suite, and the Windows CI log confirms the three repaired-fixture cases. The code uses the approved temporary-directory handshake, retains the 32 MiB threshold and production sampler, and reaps the child before returning or asserting. Proposal, delta spec, design, tests, and implementation agree; no conformance mismatch remains. The two pending tasks are specification synchronization/archival and the final post-archive protected gate.
