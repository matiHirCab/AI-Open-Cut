## Why

Issue67 and epic8 need narration attack/release control through explicit buses. Verified issues65/66 provide routing and summed DSP; current per-track role ducking cannot select a routed narration bus.

## What Changes

- Add optional closed normalized bus ducking and typed audio_bus_set_ducking under schema43, with atomic1..42 current/history migration and original transaction/draft semantics.
- Derive bounded narration clip-activity envelopes from explicit source-bus routes and shared root/component clocks; apply their minimum attack/hold/release control once after bus compression and before balance/output.
- Preserve existing per-item role ducking exactly. Explicit opt-in bus controls compose with those settings; omission, disabled/identity controls and inactive sources retain previous scenes/plans/output.
- Add conditional audio_bus_ducking_v1 readiness and manually governed protocol1 contracts/projections while preserving every historical proof and native tolerance.

## Capabilities

### New Capabilities
- audio-bus-ducking: normalized controls, transactions, canonical activity/envelopes, conditional readiness and public parity.

### Modified Capabilities
- project-persistence: complete atomic schema43 adoption.
- project-audio-buses: optional43 ducking and activated explicit-bus envelopes.
- audio-bus-dsp: declared optional ducking order between compression and stereo balance.

## Impact

Existing core model/validation/migrations/timeline/store/evaluated_scene/render_plan/render_process/renderer owners, thin headless/MCP models, fixtures/tests/docs and mandatory CI consumers. Additive protocol1 with schema43 reporting; no breaking operation/error change. Branch starts final verified66 a3138a44 (all11 CI37796389148/startup37796389099); PR targets main with cumulative merge order161→67→68→69→70.

## Non-goals

No sample-amplitude detector, compressor threshold/ratio surface, new custom bus, provider, analysis68, normalization69, raw filter/expression/path/resource, new top-level owner/import edge, parallel project validator, unrelated edit, merge/deploy/issue closure or weakened gate/timeout/profile/oracle. Narration clip activity is conservative and explicitly documented; decoded silence/automation do not masquerade as amplitude detection.
