## MODIFIED Requirements

### Requirement: Stable motion-graphics foundation status
Repository validation MUST publish one stable aggregate foundation status that waits for dedicated OpenSpec policy, contract-parity, render-parity, and rules-screen-parity statuses; executes after every terminal prerequisite outcome; reports all four results and the OpenSpec completion attestation; and succeeds only when all four results are exactly `success` and the attestation is exactly `true`. The reviewed bootstrap MUST emit attestation only after validating the complete workflow, Moon, and proto boundary and successful shell-free protected Moon execution. A failed, cancelled, skipped, neutralized, masked, or ignored-failure prerequisite MUST produce a non-successful aggregate status suitable as the single branch-protection target.

#### Scenario: All four required boundaries complete successfully
- **WHEN** the approved Moon policy task succeeds with `true` attestation and policy, contract, original render, and three-resolution rules-screen parity all report `success`
- **THEN** the foundation gate logs all four results and the attestation and succeeds

#### Scenario: Any rules-screen shard fails or is omitted
- **WHEN** any required rules-screen resolution job fails, is cancelled, skipped, duplicated, or absent
- **THEN** policy validation or the unconditional foundation assertion fails even if the original Render parity job succeeds

### Requirement: Automated CI gate policy validation
The pinned validation workflow MUST structurally verify the existing OpenSpec, contract, original Render parity, and foundation boundaries plus an exact three-resolution rules-screen matrix with bounded concurrency, non-short-circuiting failure collection, exact native test discovery and execution commands, required step-local tools/font/resolution environment, no golden mutation mode, no ignored failure, and no conditional or zero-test success path. It MUST verify that the original Render parity job retains native cache evidence and strict report validation before its unchanged report publication, and that foundation parity directly requires and checks all four prerequisite results and the OpenSpec attestation. Existing isolated Bun/Moon/proto bootstrap validation and permission boundaries MUST remain enforced.

#### Scenario: Accept all three required resolution shards
- **WHEN** the workflow lists exactly 960x540, 1280x720, and 1920x1080 with at most three parallel Linux jobs, the approved tool/font environment, exact test-discovery and test-execution commands, default failure propagation, and a four-result foundation assertion
- **THEN** policy validation accepts the distributed rules-screen evidence while preserving the original report and all existing gates

#### Scenario: Reject missing, duplicated, or zero-test evidence
- **WHEN** a resolution is missing or duplicated, the matrix or test selector changes, the exact test cannot be discovered, a shard becomes optional, or its command/environment is neutralized
- **THEN** policy validation or the shard fails before the foundation gate can succeed

#### Scenario: Reject weakened aggregate or report ownership
- **WHEN** the foundation gate omits either render result or accepts failure/skipping, or the original Render parity report validation/publication order or path changes
- **THEN** policy validation rejects the workflow

### Requirement: Protected optimized golden command
Repository policy MUST pin the exact optimized native golden command for non-rules-screen suites inside the existing required Render parity step and exact optimized native test discovery and execution commands inside every rules-screen matrix job. All other approved Render parity commands, environments, cache and report ordering, failure propagation, and publication guarantees MUST remain unchanged. A test selector that matches zero tests MUST fail rather than silently count as a successful shard.

#### Scenario: Accept reviewed split optimized commands
- **WHEN** both render jobs contain their exact approved optimized commands and protected environments
- **THEN** CI policy validation accepts the workflow without weakening the required render or foundation gates

#### Scenario: Reject altered or empty render selection
- **WHEN** a native command is removed, made optional, selects zero tests, changes profile or target, or moves outside its protected step
- **THEN** CI policy validation or explicit discovery fails before the protected foundation gate can succeed
