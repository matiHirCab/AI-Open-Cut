## MODIFIED Requirements

### Requirement: Shared deterministic audio-only event evaluation
The canonical scene SHALL evaluate event-bearing media as audio-only, including video-bearing variants, and apply audio.volume * 10^((captured defaultGainDb+gainDb)/20) once while retaining existing fades/mute/volume animation/source timing and role ducking. Preview/draft/export MUST share that behavior and existing SSIM>=0.99, aligned float-PCM RMS<=0.0001 and one-frame bounds. In schema42 approved bus DSP, captured bus identity SHALL select the starting route ahead of track explicit/role fallback without rewriting captured content/gains; older/neutral DSP remains identity. Projects without audioEvent metadata MUST retain exact evaluated plans, filter graphs, RGB and PCM.

#### Scenario: Compare native event and equivalent media
- **WHEN** fixed audio and audio-bearing video are placed via semantic events and equivalent ordinary audio-only media with the same effective gain/timing and bus route
- **THEN** native preview/export/draft match original RGB/PCM/timing oracles and video events do not add visual layers

#### Scenario: Preserve legacy role and render behavior
- **WHEN** a project contains only ordinary media or a definition is registered/replaced without placement, and DSP is absent/neutral/unreachable
- **THEN** every existing plan/filter/ducking and native pixel/audio result remains exact without altering old role ducking or claiming unimplemented explicit-bus side-chain support

