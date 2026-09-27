## ADDED Requirements

### Requirement: Bounded fake speech-provider worker tests
The repository's bridge unit suite MUST exercise the fake speech provider through its real child-process boundary on Windows, macOS, and Ubuntu. Every test in that real-child fixture, including status, voices, generation, readiness, cancellation, timeout, queueing, and failure cleanup, MUST have a bounded runner deadline longer than the provider's configured control-request deadline, without changing the provider deadline or weakening any semantic assertion. The correctness job MUST fail when those tests report a wrong result, an unexpected provider timeout, or an unhandled runner deadline.

#### Scenario: Worker starts within its provider deadline on a loaded runner
- **WHEN** a supported correctness runner exercises any real-child fake-provider operation within the provider's own deadlines
- **THEN** the assertions complete without the test runner timing out first, and the test verifies the same provider metadata, generated artifact, path readiness, cancellation, timeout, queueing, and cleanup behavior as before

#### Scenario: Unexpected worker stall or incorrect result
- **WHEN** an ordinary status, voice, or generation request unexpectedly exceeds its provider deadline, or its returned result violates an existing assertion
- **THEN** the test fails within its bounded runner deadline and the correctness job fails without ignoring or retrying the failure

#### Scenario: Cancellation of a fake hanging generation under delayed scheduling
- **WHEN** a cancellation or synthesis-timeout test submits the fake worker's hanging generation and scheduling delays the parent callback
- **THEN** the fake generation remains pending until the provider terminates it, and the test observes the expected retryable cancellation or timeout with owned cleanup instead of a successful generation
