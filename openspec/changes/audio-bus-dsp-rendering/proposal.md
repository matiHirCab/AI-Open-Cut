## Why

Issue66/epic8 require a controlled mix with normalized bus gain, stereo balance, parametric EQ and compression. Verified65 provides routing only; dependency13 is closed and64 is verified at6b5d4fa5/all11 CI37754444838.

## What Changes

- Add optional nonnull closed normalized dsp on built-in schema42 buses and typed audio_bus_set_dsp; preserve routing operations/IDs and old-media defaults.
- Share immutable evaluated bus instructions across every render intent, sum routed input before bus processing, and apply each path node once through master.
- Preserve absent/neutral/unreachable-DSP plans/filter graphs and existing role ducking. Captured event bus overrides track routing for that event. Stereo balance, EQ order and compressor timing are explicit.
- Preserve legacy readiness; check additional DSP filters only for active DSP before artifact side effects and advertise audio_bus_dsp_v1 rendering only when available.
- Adopt1..41 current/undo/redo atomically preserving41 events/draft roots,40 definitions and39 routing; forbid premature DSP and unknown future versions.
- Manually govern additive protocol1 fixture/projections and preserve every frozen predecessor proof and native threshold.

## Capabilities

### New Capabilities
- audio-bus-dsp: bounded normalized settings, transactional edits, shared evaluated mixing, conditional readiness and public parity.

### Modified Capabilities
- project-persistence: atomic schema42 adoption.
- project-audio-buses: schema42 normalized field and activated rendering while retaining historical routing-only semantics.
- timeline-audio-events: captured bus selects the DSP starting route.

## Impact

Existing model/validation/timeline/migrations/store/evaluated_scene/render_plan/render_process/renderer owners, thin headless/MCP contracts, fixtures/tests/docs. No new top-level owner/import edge, custom bus, external resource, raw filter/expression, side-chain67, analysis68, normalization69, provider, merge/deploy/issue closure or unrelated edit. PR targets main, cumulative order157→158→159→160→successor; branch starts exact verified64.
