## Context

Issue #19 builds on schema-v8 common visuals and Transform2D. Core currently evaluates arrays in declaration order; UpdateTrack already supports indexed movement and locks. The roadmap requests persisted stack order, while living architecture uses array order for equal z-index. Keeping them synchronized reconciles these contracts.

## Goals / Non-Goals

Goals: explicit deterministic visual stacking, item/track reorder APIs, atomic history migration, compatible existing operations, and shared render evidence.
Non-goals: hierarchy, new compositing effects, temporal edits, z-index animation, audio routing, and desktop inspectors.

## Decisions

1. Persist flattened zIndex (signed 32-bit integer, default 0) and stackOrder (unsigned 32-bit ordinal) through common visual properties on every item. Each track's item at index i has stackOrder i. Schema-9 documents must include valid canonical ordinals; old schemas receive them during migration. Array order remains externally meaningful and is never sorted by z-index. Alternative: sparse monotonic creation counters require overflow/rebalancing rules and conflict with existing array semantics.
2. Visual evaluation orders by (track array index, zIndex, stackOrder, stable item ID), ascending bottom-to-top. IDs only resolve synthesized equivalent keys. Audio and transition records retain ordering metadata but are not extra composited layers. Audio mixing, ducking, timing, and transition endpoint association retain their existing semantics. Alternative: global z-index would unexpectedly cross track boundaries.
3. Add item_set_z_index {itemId,zIndex}, item_reorder {itemId,index}, and track_reorder {trackId,index}, using existing projectId/expectedRevision standalone envelopes and batch discriminator conventions. Index is the final zero-based position in the existing owning collection; item reorder never changes tracks or timing and does not alter z-index. It therefore changes visual order among equal z-index peers. Track reorder delegates to indexed UpdateTrack semantics. Z-index accepts visual media, text, solids, rectangles, and captions; audio-only media and transitions reject it with INVALID_ARGUMENT. Item reorder accepts every item kind to preserve collection semantics.
4. Reindex affected item arrays after creation, deletion, move, split, duplication, caption creation, speech placement, and reorder through core-owned helpers. New creations default z-index to zero; split/duplicate/move retain source z-index. Preserve existing insertion behavior (split adjacent, duplicate appended, move according to existing operation). Every item whose ordinal changes is included in changed IDs alongside existing primary results, deduplicated deterministically. Creation result aliases must still resolve to the created ID even if other ordinals changed. Alternative: transport-local normalization would duplicate domain semantics.
5. Retain existing scene limits and preflight before sorting; bound z-index to i32 and ordinals to u32. Sort visual instructions only after existing resource/reference validation. No renderer performs its own ordering. No raw expressions, paths, network access, new package dependencies, or provider inputs are introduced.
6. Govern runtime stacking through contracts/stacking-v1.json and an ownership entry, updating headless-protocol-v1, mcp-surface-v1, schemas, native consumers, fixtures, and discovery together. Advertise stacking only with complete support. Keep remaining motion-graphics-v1 concepts fixture-only. @matiHirCab reviews the public contract changes.

## Risks / Trade-offs

- Redundant array/ordinal state: validate exact equality for schema 9 and normalize only trusted mutation candidates or older-schema migrations; reject malformed persisted current-schema values.
- Reindexing alters multiple records: test exact changed-ID sets and alias creation separately.
- Reordering audio tracks could change numerical summation order: enforce existing audio tolerance and unchanged routing/gain/ducking semantics; do not apply visual z-index sorting to audio instructions.
- Old clients cannot display new controls: omission preserves existing requests; old binaries fail closed on schema 9.
- Active changes block protected policy preflight: validate the proposal with the pinned CLI, then complete verification and archive before the final protected validation.
- User approved the artifacts on 2026-09-04; implementation is authorized.

## Migration Plan

Upgrade supported schemas 1 through 8 using the existing chain and then assign zIndex 0 and stackOrder from each snapshot's own arrays. Preserve IDs, transforms, timing, assets, provenance, revisions, and ordering. Validate the entire current/history envelope before managed-asset writes; publish under lock through the existing recoverable transaction. Fail closed on schema 0, unknown future versions, malformed state, or invalid history. Fault-injection tests cover precommit rollback and postcommit recovery. No downgrade migration: rollback requires restoring a complete pre-upgrade project/history generation.

## Verification

Every delta scenario maps to an automated test recorded in tasks/evidence during implementation. Use core mutation and migration tests, headless protocol tests, canonical Rust/Zod parity, MCP integration and packaged smoke, and render fixtures with both legacy and Transform2D layers. Check deterministic plans plus visual SSIM >= 0.99, aligned float-PCM RMS <= 0.0001, and timing within one output frame.

## Open Questions

No unresolved design choices are required. Implementation approval was received; designated contract-owner review of the resulting diff remains pending.

## Approved rendering validation correction

The user explicitly requested implementation of the corrective plan in this task. This approves the evaluated_scene -> validation ownership edge, to be recorded in ADR 0003 and its architecture checks. Review reproduced a schema-9 first-item stackOrder of 7 publishing a preview. Extract the existing read-only ordering validator and share it between persistence validation and scene evaluation immediately after complexity preflight. Validate all items including hidden tracks/items, audio and transitions; preserve VALIDATION_FAILED and its message, reject without normalization, and retain the comparator. No public shapes or migrations change. Add all-facade rejection/immutability/no-side-effect regressions for gaps, duplicates and swapped ordinals, valid and empty fixtures, then rerun required checks. This implements the existing Reject malformed persisted ordering scenario; final contract-owner review remains separate.
