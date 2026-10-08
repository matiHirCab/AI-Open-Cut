## Why

Issue64 and epic8 require agents to place registered semantic sounds at absolute or scoped marker-relative times. Verified issue63 supplies deterministic content-addressed definitions; closed issue40 supplies scoped timing.

## What Changes

- Add protocol1 timeline_add_audio_event in standalone, ordered batch and durable-draft paths, with scope, trackId, event, at, optional durationMs, gainDb (default0), variantSeed (default saved definition seed).
- Persist schema41 semantic audio provenance on ordinary media items through optional nonnull closed audioEvent metadata. Retain numeric startMs plus existing marker startTime; snapshot selected asset, default gain, bus and seed so later definition replacement affects future placement only.
- Play audio-bearing variants as audio only with additive dB gain via the canonical evaluated scene. Existing media/items, role ducking, visual output and routing-only behavior remain exact.
- Migrate source1..40 current/history atomically, preserving schema40 libraries and schema39+ buses.
- Add independently governed canonical fixtures, ownership consumers, capability timeline_audio_events_v1 and exact schema reporting transitions.

## Capabilities

### New Capabilities
- `timeline-audio-events`: bounded placement, immutable variant provenance, scoped timing, transactions, shared audiovisual evaluation and governed public parity.

### Modified Capabilities
- `project-persistence`: atomic schema41 provenance adoption preserving populated sound libraries and all retained generations.
- `media-assets`: centralized selected-event asset roots and immutable content ownership.

## Impact

Canonical model, validation, timeline, migrations, asset references, evaluated scene, headless/MCP declarations, contracts, native/transport tests and docs. Additive protocol1 surface; persisted schema41 requires deterministic upgrade and future-version rejection. Optional metadata omission preserves old media shape; present null fails closed. No provider, arbitrary resource input, custom bus/DSP/side-chain/loudness change, definition deletion, unrelated edit, merge or deployment. Existing original parity coverage and frozen historical proofs remain mandatory. Every PR targets main and cumulative merge order is #157 → #158 → #159 → successor.
