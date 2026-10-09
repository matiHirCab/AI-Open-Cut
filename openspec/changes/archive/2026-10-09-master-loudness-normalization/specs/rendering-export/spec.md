## MODIFIED Requirements

### Requirement: Shared audio-only analysis semantics
Audio analysis SHALL consume the same complete root EvaluatedScene and canonical audio source/input/per-item/summed-bus and enabled authored master-normalization lowering as preview/export. Selected range trim SHALL occur after root processing at exact48kHz stereo sample bounds, preserving warm compressor/ducking/component/event clocks. It SHALL validate the full canonical model while selecting only evaluated audio media bindings through the existing resource owner; it MUST not rasterize/open unrelated visuals or fonts. Analysis-only limits and output MUST leave all historical frame/range/draft/export plans, commands, Debug and native visual/audio/timing tolerances unchanged when authored master normalization is absent/disabled; active master processing remains shared before any final range crop.

#### Scenario: Compare analysis to complete root output
- **WHEN** processed root/component/event/bus/ducking audio is analyzed over a selected range
- **THEN** original analyzed PCM matches independently checked full-root processed PCM cropped at identical sample indices within the existing RMS/timing tolerance

#### Scenario: Select only relevant file resources
- **WHEN** selected audio is safe but unrelated visual/font files are missing, or selected audio is missing/unsafe
- **THEN** unrelated files are not opened and analysis succeeds in the first case, while the canonical existing failure precedes process/publication in the second

#### Scenario: Preserve original rendering acceptance
- **WHEN** all original native/render-plan/worker fixtures run after the additive feature
- **THEN** their unchanged historical assertions, filter/command evidence and SSIM/RMS/timing tolerances pass without using analysis work limits to constrain them

## ADDED Requirements

### Requirement: Shared prepared root master normalization
Frame, audiovisual range, materialized draft and export SHALL consume the same complete-root authored settings and checked finite measured preparation through the canonical evaluated scene and owning renderer ports. Active preparation/readiness and actual target verification MUST precede final publication, with existing model/media/matte/path failure precedence, request-owned cleanup and inactive availability unchanged. All precodec PCM comparisons SHALL preserve existing RMS/timing and visual SSIM tolerances.

#### Scenario: Compare complete root and selected render intents
- **WHEN** enabled normalization renders complete export, frame preview, selected audiovisual range or materialized draft
- **THEN** independent full-root signal/processing references and crop comparisons prove one shared master behavior with unchanged historical inactive visual/audio evidence

#### Scenario: Preserve active failure and cancellation safety
- **WHEN** active preparation or final target verification fails, required filters are absent or an owned phase is cancelled
- **THEN** the existing safe error occurs with no final artifact/state/history change and unrelated request work/outputs remain intact
