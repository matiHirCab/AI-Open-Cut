## ADDED Requirements

### Requirement: Required integrated native narration gate
Configured required native CI SHALL invoke the existing integrated six-cue narration oracle with actual FFmpeg, FFprobe and the declared font, preserving all predecessor commands, independent negative controls and thresholds. The CI policy MUST reject omission or failure masking of that exact integrated invocation.

#### Scenario: Enforce integrated narration evidence
- **WHEN** required native CI runs with configured tools
- **THEN** integrated cues, presets, captured events, ducking and normalized preview/export execute the existing independent oracle

#### Scenario: Reject missing or masked integrated evidence
- **WHEN** the configured integrated narration command is removed or its failure is masked
- **THEN** policy validation fails instead of attesting complete native coverage
