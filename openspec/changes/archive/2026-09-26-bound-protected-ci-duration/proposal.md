## Why

The protected OpenCut CI workflow recently took 2h03m39s even after distributing rules-screen renders. It has no enforceable elapsed-time budget, so future slowdowns can pass without a recorded reason or a bound.

## What Changes

- Set a 120-minute default elapsed-time budget for the required CI workflow and make a measured overrun fail the protected foundation status.
- Permit a narrowly scoped, dated, CODEOWNER-reviewed exception with a stated cause, measured baseline, owner, expiry, and hard upper bound. The initial exception covers the measured 1920x1080 rules-screen render while its renderer cost remains unresolved.
- Publish the measured elapsed time, effective budget, and exception reason in CI; reject missing, expired, malformed, or unbounded exceptions.
- Extend the workflow policy validator and adversarial tests to prevent bypassing the duration audit, its dependencies, or the foundation assertion. Keep all existing test cases and output assertions.

No public API, persisted schema, MCP contract, renderer output, or test selection changes. This is an additive CI policy change, with no breaking product compatibility impact.

### Non-goals

- Reducing or removing render coverage, tuning the production renderer, or compacting `contracts/mcp-surface-v1.json`.
- Guaranteeing GitHub queue or infrastructure time outside this workflow's own run interval.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repository-validation`: enforce and report a reviewed elapsed-time budget for the protected workflow, including bounded exceptions and fail-closed aggregate wiring.

## Impact

The GitHub Actions workflow, CI policy validator and tests, repository validation specification, and CI gate documentation change. The existing `@matiHirCab` CODEOWNER review boundary applies to the workflow and policy files. The unrelated `.codex/config.toml` edit remains untouched.
