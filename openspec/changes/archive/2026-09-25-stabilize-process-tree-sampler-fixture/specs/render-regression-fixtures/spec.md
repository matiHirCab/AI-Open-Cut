## MODIFIED Requirements

### Requirement: Bounded process-tree sampler lifecycle
Every started benchmark process-tree sampler MUST signal and join its worker exactly once on explicit completion and on every early return or panic after startup. Explicit completion MUST surface a worker panic; cleanup during unwinding MUST never panic. Sampler shutdown MUST complete after the worker's current refresh and at most one declared sampling interval, and MUST NOT leave a detached worker that can consume resources or perturb later benchmark observations. The cross-platform child-allocation regression fixture MUST keep its allocated child alive until the sampler has observed the required process-tree memory increase or a bounded observation deadline expires; the fixture MUST release and reap the child in either outcome and MUST continue to fail when the required increase is not observed.

#### Scenario: Finish a measured capture
- **WHEN** benchmark capture completes normally
- **THEN** explicit sampler completion stops and joins the worker before returning the maximum observed process-tree resident memory

#### Scenario: Unwind after benchmark failure
- **WHEN** encoding, decode, composition, or later conformance work fails after the sampler starts
- **THEN** RAII cleanup stops and joins the worker before unwinding continues without replacing the original failure

#### Scenario: Observe a held child allocation
- **WHEN** the regression fixture starts a child that allocates the declared memory and signals readiness on Windows, Linux, or macOS
- **THEN** the child remains alive while the sampler observes the required increase, after which the fixture releases and reaps it and passes the same memory-delta assertion

#### Scenario: Observation or child readiness fails
- **WHEN** the child exits early, fails to become ready, or the sampler does not observe the required increase within the declared deadline
- **THEN** the fixture terminates within a bounded time, releases and reaps the child if it is still running, and fails with the relevant condition rather than passing or hanging
