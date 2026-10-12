## ADDED Requirements

### Requirement: Observable native sampled visual preparation
Range previews and exports SHALL report successful sampled visual-frame preparation through the existing numeric progress event before final FFmpeg execution. Preparation updates MUST represent completed work, remain finite and bounded, and retain the admitted dimensions, frame counts, clocks and resource limits.

#### Scenario: Slow sampled preparation advances before encoding
- **WHEN** a valid range or export performs deliberately slow multi-frame visual preparation
- **THEN** clients observe increasing positive preparation progress before final encoding starts, without changing rendered samples or requested timing

#### Scenario: Empty or single-frame preparation
- **WHEN** a valid request has no sampled visual work or only one sampled frame
- **THEN** preparation completes safely without division by zero or nonfinite progress and ordinary rendering remains compatible

### Requirement: Bounded truthful reported-work progress
Native range/export progress SHALL remain monotonic in [0,1]. For sampled visual work the documented fixed allocation SHALL reserve [0,0.5] for preparation and [0.5,0.99] for final encoding; requests without sampled work SHALL map encoding to [0,0.99]; one MUST be emitted only after successful publication and artifact validation. Cache hits MUST obey the same publication-completion rule. Progress SHALL describe reported work rather than predict elapsed or remaining time.

#### Scenario: Encode and publish successfully
- **WHEN** preparation and encoding succeed and the artifact is published and validated
- **THEN** progress crosses the phase boundary without regression, encoding completion stays below one, and successful publication produces one

#### Scenario: Fail preparation encoding or publication
- **WHEN** preparation, final encoding or publication fails before a validated artifact is returned
- **THEN** progress does not falsely report one, canonical errors remain unchanged and no successful completion event is emitted

#### Scenario: Publish a cached preview
- **WHEN** cached bytes are used for a range preview
- **THEN** publication and validation still precede progress one, and publication failure cannot masquerade as completed rendering

### Requirement: Preserve revision cancellation and render safety
Preparation reporting MUST preserve validation and revision-conflict precedence, immutable rendering state, existing destination/publication ownership, cleanup, retained history and cancellation policy. Updates after terminal cancellation MUST NOT revive a job or change its terminal state.

#### Scenario: Reject invalid or stale requests
- **WHEN** a request is invalid or carries a stale revision
- **THEN** its existing validation or revision error wins before preparation/execution and no preparation update or state/output mutation occurs

#### Scenario: Cancel during preparation
- **WHEN** an active native job is cancelled while preparing visuals
- **THEN** existing cancellation and cleanup behavior applies, no completed artifact is claimed, and late progress cannot revive the cancelled job

### Requirement: Govern compatible numeric progress forwarding
Core, standalone/persistent headless and MCP job status SHALL retain the existing numeric progress/event shapes and finite [0,1] bounds. Canonical behavior evidence and every listed consumer MUST agree; existing schemas, operations, stable errors, retryability and protocol/project versions SHALL remain unchanged. Exact prior declarations and negative controls MUST survive current-main and pending-lane integration.

#### Scenario: Forward preparation progress through actual transports
- **WHEN** preparation updates pass through headless and source or freshly packaged MCP job execution
- **THEN** the same numeric values reach job polling before terminal response, ordinary compatible events remain valid, and public metadata contains no private paths or arguments

#### Scenario: Preserve canonical and overlapping owner evidence
- **WHEN** this behavior is governed and combined with exact current main or pending startup diagnostics
- **THEN** the complete canonical gate preserves every prior declaration/pin/negative and independent resolution review records the semantic integration before publication
