## Context

The Windows correctness job for PR #125 ran 413 bridge unit tests successfully but two fake speech-provider cases reached Vitest's five-second default deadline. They exercise real Python child startup and pipe requests, and the provider already has a ten-second control-request deadline. The test runner therefore can abort before the behavior under test reaches its own bounded outcome. After the runner deadline was raised in the 11-case file, the full local suite revealed a second fixture race: the fake `hang` operation returned after its fixed ten-second sleep because load delayed the cancellation timer. The cancellation test then correctly rejected that false success.

## Goals / Non-Goals

**Goals:** Give the real-child fake-provider fixture cases sufficient bounded wall time for their existing assertions and for provider-level timeout reporting; make the fake `hang` operation remain pending until the provider terminates it so cancellation and timeout tests do not race a finite sleep; make Windows CI demonstrate that the cases still execute and fail on wrong outcomes.

**Non-Goals:** Change production timeouts, provider protocol, fake-worker response semantics, global unit-test timeout, or remove any test.

## Decisions

- Apply an explicit 20-second Vitest deadline to each of the 11 real-child fake-provider cases in the one speech test file. The deadline exceeds the provider's ten-second control timeout with room for Windows process startup and cleanup. The actual provider limits and all assertions stay unchanged. A global timeout increase was considered but would obscure hangs throughout the bridge suite. Limiting the deadline to the first three cases was tried and failed under full-suite load in a cancellation case.
- Make the fake worker's `hang` request wait indefinitely in a low-CPU loop until the parent process terminates it. The provider's cancellation and synthesis timeout are the test's bounded stop conditions; Vitest still has a 20-second backstop. A longer finite sleep was considered but retains the same race under extreme scheduling delays. Retain the real process boundary; replacing it with a mock would lose startup, pipe, and cleanup coverage.
- Verify the focused cases and full bridge suite locally, then require the complete Windows correctness job in the protected workflow. A green local run alone cannot establish hosted Windows stability.

## Risks / Trade-offs

- A genuine stall in any of these 11 cases can now take up to 20 seconds to report. Provider requests retain their shorter runtime limits, and the tests remain bounded.
- Hosted Windows timing varies. The full correctness job is the acceptance evidence; a repeated 20-second deadline failure requires investigating process startup or teardown rather than increasing the deadline again without evidence.
- The indefinite fake-worker wait must be reaped by the provider on cancellation or timeout; the bounded test deadline surfaces any cleanup regression.

## Migration Plan

No data, contract, or deployment migration applies. Reverting the test deadline restores the original five-second runner behavior. This change is confined to tests and repository validation requirements.

## Open Questions

None.
