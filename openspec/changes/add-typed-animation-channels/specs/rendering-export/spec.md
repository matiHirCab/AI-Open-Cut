## ADDED Requirements

### Requirement: Shared supported channel rendering
Supported typed channels MUST be sampled by the canonical evaluated scene for frame preview, audiovisual range preview, draft preview, and export. Render intents MUST preserve existing output when no new channels are present and MUST fail before output side effects if a persisted unsupported channel is encountered.

#### Scenario: Render supported animation consistently
- **WHEN** the same revision and timestamp are rendered through preview and export
- **THEN** both use the same sampled channel values within the existing documented output tolerance

#### Scenario: Reject unsupported persisted animation
- **WHEN** an externally edited project contains a well-formed but inactive channel
- **THEN** render preflight returns a stable typed error and publishes no artifact
