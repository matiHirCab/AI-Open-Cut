## Context

Issue #24 follows stored component definitions (schema 11) and typed slots (schema 12). Root component instances currently fail validation; the evaluator explicitly skips the variant. The private scene uses integer spans today, so rounding at every nested boundary would lose valid fractional timing. Existing group affine evaluation and path-safe preparation remain the foundation.

## Goals / Non-Goals

**Goals:** Render ordinary supported component content end to end, with typed root editing, nested local clocks, isolated override resolution, deterministic hierarchical order, bounded work and equivalent preview/export semantics.

**Non-Goals:** See proposal.md. In particular, this is flattened per-descendant compositing, not an offscreen component texture or component clipping mask.

## Decisions

1. **Root edits and schema 13.** Add add_component_instance with trackId, componentId, startMs, trimStartMs, durationMs, timeScale and optional slotValues/transform/transform2d/hidden/zIndex/parent using existing common-field spellings and defaults. Core generates the item ID. component_instance_update takes itemId and the complete componentId/startMs/trimStartMs/durationMs/timeScale tuple; omitted slotValues preserves overrides, an explicit map replaces them. Common visuals use existing generic edits. Both participate in drafts and batches; create produces resultAlias. Earlier aliases resolve in trackId, componentId, itemId and parent.id, never slot keys or local IDs. Root overlay placement makes the capability addressable. Merely exposing a test-only evaluator would not satisfy the issue. Advancing the schema rather than silently broadening schema 12 ensures old readers reject new root records.

2. **One pure traversal.** Validate the complete project and effective slot candidates before expanding reachable root instances. Reuse the canonical slot resolver and apply typed values to ephemeral local items only, including rich-text runs. Maintain an instance path for identity and ordering; never mutate shared definitions or synthesize persisted IDs. Repeated occurrences remain independent. Keep resource bindings outside the private scene and deduplicate logical assets without conflating input source intervals.

3. **Affine local clock.** At each edge local(t) = trimStartMs + (parent(t) - startMs) * timeScale, active on the half-open parent interval. Compose slope/intercept with finite checked arithmetic, intersect every enclosing interval, and retain fractional derived milliseconds until final output sampling. Source trims, animation, fades and internal transitions use the same local clock. Audio playback uses the product of scales with pitch-preserving tempo adjustment; finite rates outside backend-supported preparation limits return INVALID_ARGUMENT before writes, never silently render at another rate. Decompose tempo into bounded stages (at most 32 factors in [0.5,2]); cumulative supported rate is [2^-32,2^32]. Existing requested durations remain integer milliseconds; no new public fractional fields.

4. **Coordinates and order.** Definition dimensions establish local normalized coordinates. Instance position uses its parent composition; normalized instance anchor uses referenced component dimensions. No automatic fit scaling. Compose descendant matrices outward through local groups, instance, and outer groups; multiply opacity per descendant. Instances form contiguous visual blocks in parent order; recursively sort local children by track/zIndex/stackOrder/ID. Local transitions cannot cross instance occurrences. Descendants may extend beyond the component canvas; only existing root clipping applies. Alternative offscreen isolation would change opacity/overlap semantics and require new raster surfaces.

5. **Visibility and audio.** Instance item/track hidden suppresses the entire occurrence including audio. Group visual parenting retains its existing independence from audio. Local media tracks retain mute/role/gain behavior; local audio timing, automation, fades and voiceover activity are mapped before the existing global mixing/ducking rules. This avoids renderer-specific flattening and silent loss of component sound.

6. **Expansion budgets.** Preserve depth 16, stored graph/slot bounds, 4096 scene visual/audio/resource/transition limits, 10000 keyframes per channel and voiceover ranges. Count at most 65536 visited item occurrences (including groups/instances/hidden occurrences) before allocation using saturating preflight counts; a small DAG can otherwise expand exponentially. Finite derived clocks, matrices, geometry and the tempo-stage bound are checked before preparation side effects. Resource measurement remains bounded and read-only.

## Risks / Trade-offs

- Fractional nested clocks can expose integer assumptions throughout render planning → use independent boundary oracles, fractional-rate media fixtures and one shared clock representation.
- Hierarchical visual order can collide with local IDs → identify occurrences by complete instance path and test repeated diamonds and transition endpoints.
- Slot-rich text already persists without rendering → preserve style runs through measurement/rasterization; do not flatten to plain text.
- Pitch preservation and sampling precision → test audiovisual fixtures at non-unit rates against existing canonical tolerances; no new relaxed thresholds.
- Public contract growth → canonical fixtures first, all ownership-listed consumers and designated CODEOWNER review.
- A valid stored graph can exceed expansion limits → typed deterministic evaluation failure before filesystem writes, with direct-render tests.

## Migration Plan

After approval, update canonical schema/operation fixtures first, then schema 12→13 migration and every governed consumer. Under the project lock migrate current plus all retained snapshots in one existing recoverable transaction. The migration changes only schema version; valid old root content, definitions, slots, revisions and provenance remain exact. Validate old snapshots before upgrading so formerly invalid root instances are not legalized by relabeling. Unknown future schemas fail without writes. Rollback requires an intact pre-migration generation/backup with the older binary; do not auto-downgrade schema-13 edits. Render failures never mutate project or history and retain existing atomic artifact publication.

## Verification Plan

Every requirement/scenario in specs maps to tasks and automated tests: independent time/matrix/order oracles; exact limit and limit+1 cases; managed asset and slot failures; standalone/batch/draft edits and alias rollback; revisions, locks, undo/redo and reopen; old current/history migration with failure injection; shared frame/range/export goldens including audio; cross-language contracts, source integration and packaged smoke. Full commands are in tasks.md. Record verification evidence before archive, then run the archive-only Moon gate on the final tree.

## Open Questions

The user approved this proposal and its root-edit/schema-13 scope on 2026-09-05. No unresolved implementation choices permit scope expansion without updating these artifacts and obtaining approval.
