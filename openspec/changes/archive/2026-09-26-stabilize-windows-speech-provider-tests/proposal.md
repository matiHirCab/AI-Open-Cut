## Why

PR #125's Windows correctness job failed when two fake speech-provider tests reached Vitest's five-second default deadline, although the provider's existing control deadline is ten seconds. The fixture must allow the bounded worker startup and teardown it exercises on loaded Windows runners while still detecting a genuine provider timeout or wrong result.

## What Changes

- Give only the 11 real-child fake-provider tests in the speech fixture a bounded test deadline longer than the configured provider control timeout, including cleanup margin. The full suite also timed out in a cancellation case after the initial three cases were adjusted.
- Make the fake worker's `hang` operation remain pending until it is terminated, so a delayed cancellation cannot race with the fixture's previous ten-second sleep and produce a false success.
- Keep the provider's production control and synthesis deadlines, semantic assertions, and Windows correctness gate unchanged.
- Verify the focused suite and full Windows correctness job, including failures that exercise provider timeout and cancellation.

No breaking change is introduced. Non-goals: altering the production provider protocol, relaxing runtime deadlines, globally increasing unit-test timeouts, or skipping speech tests.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repository-validation`: Define bounded timing for fake speech-provider integration-style unit fixtures on Windows and preserve all correctness assertions and failure propagation.

## Impact

The change affects the speech-provider test and fake worker in `apps/agent-bridge/tests/` and the repository-validation specification. It does not change public contracts, persisted data, production provider behavior, or dependencies.
