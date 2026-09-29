## Context

Issue #41 follows the active typed-channel and parameterized-curve milestones. Project schema is 24. Channels are stored on visual properties, sampled in editor-core, and edited with `set_animation_channels` directly or in `timeline_batch_edit`. Frame and range renderers consume evaluated-scene values. The living channel spec currently forbids loop input, so this change first defines an additive contract and a schema migration.

## Goals / Non-Goals

**Goals:** Reusable, bounded finite/infinite forward and ping-pong loops on active channels; exact half-open seam rules; deterministic phase independent of preview-range start; atomic editing/history/reopen; visual and audio render parity; contract and capability evidence.

**Non-goals:** Legacy `set_keyframes` loop semantics, per-item loop objects, looped media playback or duration, marker-relative keyframe times, inactive channel properties, a new edit operation, or a desktop loop inspector.

## Decisions

1. **Put an optional loop on each typed channel.** The record is `{mode:"repeat"|"ping_pong",iterations:1..10000|"infinite"}`; absent `loop` retains exact existing hold behavior. Each channel can loop independently through the existing replace-channels operation. A fixed iteration cap bounds count parsing and arithmetic even though sampling is constant work. Unknown variants/fields, fractional/zero counts and non-finite values are rejected. Alternative: add an item-wide loop; rejected because channels can have different timing spans and active audio and visual properties need independent control.
2. **Use the first and last keyframe timestamps as the loop interval.** A loop needs at least two keyframes and positive integer-millisecond span. Before the first timestamp, hold the first value. For forward repeat, one cycle is `[first,last)`; at every exact seam the first value is sampled. To ensure value continuity, first and last values must be exactly equal for `repeat`. Ping-pong traverses forward to the last value, then backward to the first; one iteration is a full round trip of `2 * span`. Reverse traversal uses the existing forward curve at the reflected time, preserving its documented numeric result. Finite exhaustion holds the repeat endpoint (which equals the first value) or the ping-pong first value. This is value continuity at seams, not a promise of equal derivatives. Alternative: allow mismatched repeat endpoints; rejected because an exact seam would jump.
3. **Anchor phase to item-local time, never a preview request.** Convert composition time through existing item/component timing, then use checked integer offset, division, remainder, and reflection. Resolve the phase before the existing curve sampler; do not accumulate one cycle at a time. The same absolute timestamp yields the same value in a single-frame request, a range beginning earlier/later, a draft, and an export. Keep channel canonical bounds and exact endpoint handling. Alternative: advance mutable playback state per rendered frame; rejected because range start, frame skipping, and worker concurrency would change output.
4. **Keep public edits additive.** Extend the checked-in channel catalog, versioned project fixture, headless/MCP schema catalog and capability lists, then update Rust and TypeScript consumers and parity tests. Preserve the existing operation names, response envelopes, protocol major, alias resolution, errors, and old request encodings. No raw renderer expression, executable SVG, path or network input enters the loop record. Alternative: add `set_animation_loop`; rejected because it would split one channel's transaction and duplicate editing surfaces.
5. **Migrate a complete durable generation.** Schema 25 represents absent `loop` on existing channels. Under the project lock, migrate schema-24 current state and each retained undo/redo snapshot, validate the whole candidate, and publish through the existing recoverable transaction. Reject loop fields in source versions below 25 and reject malformed or future snapshots before publication. No down-migration. Alternative: lazy defaults only on read; rejected because retained history would remain mixed-version.
6. **Render from one evaluated behavior.** Core's evaluated-scene sampling is authoritative for visual transform/opacity and audio gain. Any FFmpeg plan approximation must agree with core samples at equivalent timestamps and existing decoded visual/audio/timing tolerances, including a range that starts inside a later cycle. Unsupported or invalid persisted loop state fails preflight before destination inspection or output side effects. Alternative: reset phase in each render invocation; rejected because subranges would disagree with export.

## Failure, Compatibility, and Security

Invalid loop shape, one-keyframe loop, zero span, nonmatching repeat endpoints, count overflow, and unsupported channel use return non-retryable `INVALID_ARGUMENT`. Missing item, locked track, unresolved batch alias, and stale revision retain existing stable codes and precedence. A failed batch changes no revision, state, history, alias map, or artifact. Current clients sending channels without `loop` get the same wire and evaluation results. Loop input is strictly typed and contains no resource reference.

## Risks / Trade-offs

- A repeat with different endpoints would visibly jump. Reject it and document ping-pong as the motion that naturally reverses at its endpoints.
- Very short loop periods can produce many seams in a long item. Constant-work modular sampling and the existing keyframe/channel/item limits prevent per-cycle expansion.
- FFmpeg filter expressions may have different boundary rounding. Tests must compare exact seams and frames near seams across supported FFmpeg 6 and 8, including audio gain and nonzero preview start.
- Historical schema and contract catalogs are exact. Migration fixtures, Rust/TypeScript parity tests, and the MCP digest must be updated deliberately before consumers are changed.

## Migration Plan

Write schema-25 fixtures and failing migration/contract tests first. Migrate current and retained states, then implement core sampling and typed adapters. Run implementation and conformance checks in repository order, synchronize verified deltas, archive, and rerun the protected gate. A migration failure leaves the prior complete generation authoritative; recovery after interruption chooses exactly one complete generation.

## Open Questions For Approval

Approval of this design confirms the proposed per-channel wire record, the 10,000-iteration cap, exact endpoint equality for forward repeat, ping-pong count as complete round trips, and schema-25 migration. These choices make the issue's unspecified loop boundaries and continuity rule reviewable before code changes.
