## Why

Issue #47 requires timeline editing to retain continuous animation values, loop phase, and marker-reference lifecycle through editing and history. Current split endpoint resampling changes curve interiors; typed channels can fail duration validation after splitting or trimming.

## What Changes

- Retain original typed channel and legacy animation source keys with bounded clocks through split and trim; duplicate copies exact animation records.
- Preserve arbitrary Bézier/spring segments, finite repeat/ping-pong exhaustion, fractional inherited sampling, extended values and audio gain.
- Preserve existing marker rules: split clears both start expressions; trim clears only when numeric start changes; duplicate shifts marker offsets.
- Persist optional retained animation clocks in schema 31, migrating supported schema 1–29 current/components/history atomically. Reserve schema 30 for issue #46 and reject it in this independent branch until integration adds its compatible adapter.
- Synchronize canonical fixtures, typed consumers, capability schema claims, guides and verification evidence.

## Capabilities

### New Capabilities

- `animation-edit-semantics`: source-clock preservation and editing conformance.

### Modified Capabilities

- `animation-channels`: effective retained source-duration key bounds and source-clock sampling before curves/loops.
- `animation-loops`: retained source-clock phase and original finite exhaustion through editing.
- `animation-presets`: preserved original compilation attribution and exact source primitives/effective clocks through retained edits; fresh clocks only for newly compiled properties.
- `project-persistence`: schema 31, bounded retained clocks and atomic historical migration.

## Non-goals

No new operation, motion-pack implementation, marker lifecycle redesign, component split support, merging or deployment. No lossy sampled curve replacement or duration-dependent loop restart.

## Impact

Core animation/model/validation/timeline/persistence, headless and MCP project/channel schema consumers, canonical animation and capability fixtures, documentation and tests. Optional clock fields are additive for valid existing request payloads; schema 31 is a persisted compatibility boundary and older readers must reject it. Existing stable error codes, retryability, track restrictions, media ownership and resource limits remain unchanged. CODEOWNER is @matiHirCab; delegated specification approval and independent agent review must be recorded honestly, without implying human review occurred.
