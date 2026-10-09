## ADDED Requirements

### Requirement: Portable bridge shutdown conformance fixture
The bridge shutdown conformance fixture MUST launch its headless probe through a host-native executable without relying on shell or batch execution. It MUST retain readiness, shutdown ordering and worker cleanup assertions, and expose failed job details plus bounded bridge diagnostics instead of masking pre-shutdown failures.

#### Scenario: Launch a native fixture before shutdown
- **WHEN** the shutdown regression runs on a supported host
- **THEN** the probe executes through the production direct executable boundary and alignment is proven active before shutdown is requested

#### Scenario: Diagnose pre-shutdown readiness failure
- **WHEN** the fixture job fails before shutdown
- **THEN** the unchanged readiness assertion fails with structured job and bounded bridge stderr evidence, rather than claiming a disposal failure

#### Scenario: Exercise a supported shutdown entry on each platform
- **WHEN** Node runs the regression on Windows or POSIX
- **THEN** stdin EOF exercises the existing graceful shutdown entry on both platforms, POSIX additionally retains real SIGTERM coverage, and active worker plus bridge cleanup meet the unchanged deadline
