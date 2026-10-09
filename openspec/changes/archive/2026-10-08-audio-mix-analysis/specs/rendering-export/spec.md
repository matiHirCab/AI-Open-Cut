## ADDED Requirements

### Requirement: Shared audio-only analysis semantics
Audio analysis SHALL consume the same complete root EvaluatedScene and canonical audio source/input/per-item/summed-bus lowering as preview/export. Selected range trim SHALL occur after root processing at exact48kHz stereo sample bounds, preserving warm compressor/ducking/component/event clocks. It SHALL validate the full canonical model while selecting only evaluated audio media bindings through the existing resource owner; it MUST not rasterize/open unrelated visuals or fonts. Analysis-only limits and output MUST leave all historical frame/range/draft/export plans, commands, Debug and native visual/audio/timing tolerances unchanged.

#### Scenario: Compare analysis to complete root output
- **WHEN** processed root/component/event/bus/ducking audio is analyzed over a selected range
- **THEN** original analyzed PCM matches independently checked full-root processed PCM cropped at identical sample indices within the existing RMS/timing tolerance

#### Scenario: Select only relevant file resources
- **WHEN** selected audio is safe but unrelated visual/font files are missing, or selected audio is missing/unsafe
- **THEN** unrelated files are not opened and analysis succeeds in the first case, while the canonical existing failure precedes process/publication in the second

#### Scenario: Preserve original rendering acceptance
- **WHEN** all original native/render-plan/worker fixtures run after the additive feature
- **THEN** their unchanged historical assertions, filter/command evidence and SSIM/RMS/timing tolerances pass without using analysis work limits to constrain them
