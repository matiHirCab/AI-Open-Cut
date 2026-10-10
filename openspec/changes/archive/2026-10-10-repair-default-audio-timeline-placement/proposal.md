## Why

Independent actual H264/AAC exports from exact main64335ba1 demonstrate a500ms clip authored at2000ms playing at time0 with absent/identity DSP; activating only gainDb=-1 restores placement. Latest main16b41 retains the same audio owners. Timestamp-only shifts before sequential sample mixing do not preserve authored silence.

## What Changes

- Place ordinary evaluated audio on its canonical timeline before mixing, regardless of absent, identity, unreachable or active DSP and routing; retain local source trim/fade/automation and global ducking clocks.
- Add mandatory native decoded-PCM evidence for delayed, separated, overlapping, trimmed and routed clips across audiovisual preview/export and history/reopen; preserve component/retained-clock behavior.
- Explicitly narrow existing exact-legacy-output requirements to permit this reviewed timing correction. Preserve original historical witnesses; any impacted current golden reference must receive separate concrete reviewer approval with provenance and independent clock expectations before installation.
- Preserve public/persisted contracts, existing cleanup fixes, error/cancellation/publication boundaries and all required gates.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rendering-export`: require independently witnessed canonical sample placement for ordinary audio across DSP modes and output intents.
- `audio-bus-dsp`: permit the approved ordinary-clip timing correction within exact-legacy compatibility guarantees.
- `project-audio-buses`: require neutral routing equivalence against corrected authored audio clocks.

## Impact

Core render-plan audio placement and focused owning tests/native media evidence; possible readiness checks for any newly required base filter, and narrowly affected reviewed golden references/documentation. No new runtime role, provider, transport validator, dependency, schema/protocol/catalog change or public API is planned. A necessary public-contract change returns for scope approval before implementation.

## Non-goals

No unrelated refactor, Windows-specific workaround, TTS/narration authoring redesign, cleanup redesign, merge or deployment. The exact Windows Brock project is unavailable; repair the independently reproduced shared defect and report that coverage limitation.
