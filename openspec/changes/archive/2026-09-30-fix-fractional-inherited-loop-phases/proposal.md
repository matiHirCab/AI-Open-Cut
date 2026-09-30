## Why

Issue #42 review reproduced a parent repeat channel wrapping at local time 99.5 ms because render planning rounds it to 100 ms. Frame, draft, range and export share this wrong result. The living repeater scenario also incorrectly excludes controller stagger when timeOffsetMs is zero.

## What Changes

- Preserve fractional mapped visual loop phase through inherited parent and descendant sampling, including repeat seams, ping-pong turns and finite exhaustion.
- Add independent scalar and native four-intent regressions for the 0.995-rate reproduction, nested clocks, stagger and signed copy offsets.
- Clarify integer persisted timestamps versus fractional derived clocks, and correct the repeater scenario to include controller delay.
- Record corrected verification evidence while preserving both earlier issue-42 archives.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `animation-loops`: Distinguish stored integer keyframe offsets from fractional mapped visual loop phase and preserve existing integer sampling.
- `inherited-animation-timing`: Require independent sub-millisecond loop conformance around seams and exhaustion across all four render intents.
- `repeaters`: Correct the additional-copy activity scenario for ranked controller stagger plus signed copy offsets.

## Impact

Owners are editor-core animation and render planning, focused core/native tests, timing documentation and OpenSpec artifacts. No new dependency edges or transport validation are intended. Public fields, capabilities, schema 26, stable errors, revisions, persistence, resource bounds and independent audio remain unchanged. Fractional visual loop output is corrected; integer-clock and unrelated zero-offset output remains compatible. No canonical golden replacement or tolerance change is authorized.

## Non-goals

No new animation properties, audio loop or tempo changes, media frame-resampling changes, migrations, provider work, unrelated refactors, commits or pushes. Both prior archived changes remain immutable. Proposal, delta specs, design and tasks require explicit approval before implementation under AGENTS.md.
