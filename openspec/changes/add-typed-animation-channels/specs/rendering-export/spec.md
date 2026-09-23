## ADDED Requirements

### Requirement: Shared supported channel rendering
Supported typed channels MUST be sampled by the canonical evaluated scene for frame preview, audiovisual range preview, draft preview, and export. Render intents MUST preserve existing output when no new channels are present and MUST fail before output side effects if a persisted unsupported channel is encountered.

#### Scenario: Render supported animation consistently
- **WHEN** the same revision and timestamp are rendered through preview and export
- **THEN** both use the same sampled channel values within the existing documented output tolerance

#### Scenario: Render each active visual property
- **WHEN** position X/Y, independent scale X/Y, or opacity is animated on a compatible visual item and another property has no channel
- **THEN** frame, audiovisual range, draft, and export output reflect the sampled values and the absent channel uses its static legacy value

#### Scenario: Mix bounded audio gain with existing audio controls
- **WHEN** media with audio has a valid gain channel and base volume, mute, fade, or ducking settings
- **THEN** decoded output reflects the decibel-to-linear multiplier together with those settings at supported times, while gain values outside [-96, 12] are rejected before publication

#### Scenario: Reject unsupported persisted animation
- **WHEN** an externally edited project contains a well-formed but inactive channel
- **THEN** render preflight returns a stable typed error and publishes no artifact
