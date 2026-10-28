## ADDED Requirements

### Requirement: Hermetic combined acceptance with retained genuine-media proof
Combined artwork/narration/durable source and package acceptance MUST use actual core and explicitly configured test backends in hermetic mode without ambient media tools, readiness skips or weakened common assertions. Probe facts MUST derive from bounded validated owned bytes. Explicit genuine mode MUST retain independent FFmpeg decoding and require explicit genuine tools. Evidence MUST distinguish modes; runtime, gates, profiles, timeouts and historical oracles SHALL remain unchanged.

#### Scenario: Hermetic source and package
- **WHEN** ambient media tools are absent and configured fake backends are supplied
- **THEN** the complete combined scenario passes, checking exact fake output and all common assertions without claiming pixels

#### Scenario: Explicit genuine source and package
- **WHEN** genuine mode is selected
- **THEN** explicit genuine tools independently decode the actual64x64 frame; missing tools fail without fallback/skip and common assertions remain

#### Scenario: Invalid configuration or fixture
- **WHEN** configured tools are absent or a probe input is unknown, malformed, changed or outside owned bounds/root
- **THEN** acceptance fails explicitly without ambient fallback or fabricated success
