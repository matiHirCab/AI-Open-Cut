## Why

The required Render parity job on PR #124 took 345 minutes. Its log attributes 20,448 seconds to the single debug-profile `native_golden_render_conformance` test, while compilation and the remaining tests took only minutes. Earlier commits spent three to five hours in the same test. This feedback loop is too slow for a protected pull-request gate, and the monolithic test does not report which conformance suite consumed the time.

## What Changes

- Run the existing native golden conformance test in Rust's optimized test profile within the same required Render parity step, without removing test cases, assertions, or report capture.
- Emit elapsed time for each existing golden conformance suite and the sampled capture so future slowdowns have an attributable source.
- Update the exact CI policy expectation and regression tests for the reviewed command. Measure the before/after job and test durations; retain the current gate if optimized execution changes deterministic output or does not materially shorten the run.

No public API, persisted schema, canonical contract, rendering output, golden reference, tolerance, or merge-gate strength changes. This change does not introduce a universal timing pass/fail budget.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: Require the same native golden evidence under an optimized test profile, with per-suite timing observations.
- `repository-validation`: Pin and verify the revised protected Render parity command while preserving the closed workflow and failure behavior.

## Impact

Affected files are the Render parity GitHub workflow, its exact policy validator and tests, and the test-only native golden harness. The Rust release test artifact may add CI compilation time, but it should substantially reduce the expensive conformance execution. The other native tests and all protected job dependencies remain as currently approved.
