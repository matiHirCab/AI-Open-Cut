## Context

Issue #20 follows common visual properties, Transform2D, and schema-9 stacking already implemented here. ADRs 0002, 0003, and 0004 govern contracts, ownership, and ancestor order. No active approved change existed at intake.

## Goals / Non-Goals

Goals: typed editable groups, bounded parent DAGs, atomic migrations/history, shared deterministic evaluated output, and agent-addressable standalone/batch workflows.
Non-goals: components, effects, isolated group compositing, audio parenting, group animation, recursive subtree operations, keep-world reparenting, and desktop authoring controls.

## Decisions

1. Keep GroupItem in the existing timeline sum type with common visual properties and a static identity Transform2D. Add optional parent {scope,id} to common visual properties. Root is the only activated composition scope; cross-track root references are valid. Reject other scopes rather than activating components. A separate group table or child list would duplicate timeline ownership and ordering.
2. Parent IDs resolve only to groups. Build a unique-ID index after count checks and validate iteratively, including hidden/inactive records. Limit each path to 32 ancestor edges and root visual/group records to 4096, in addition to current project limits. Deterministic timeline traversal defines error selection. Recursive unbounded traversal is rejected.
3. Add add_group and item_set_parent as typed core mutations and thin transport adapters. Creation takes trackId, startMs, durationMs, optional transform2d and parent; existing project/revision envelopes apply. Creation uses the established single-ID result alias. Resolve aliases inside parent.id and itemId through existing machinery. Explicit null detaches. Preserve local transforms; keep-world mode would need inverse/singular handling outside this issue.
4. Reject group deletion that leaves references and preserve the existing empty-track-only deletion rule. Callers detach/reparent children, delete items, then delete the empty track. The user explicitly approved this correction in the task conversation on 2026-09-04. Validate every evolving batch operation. Child detachment before deletion makes intent explicit; automatic cascading could remove unrelated content. Node-only duplication preserves the same parent without cloning descendants. Existing visual child split/move/duplicate retains its parent. Group split/keyframes/audio/transitions/legacy transforms are explicitly unsupported.
5. Reuse canonical Transform2D and render preparation. Compose local matrices nearest-parent outward and multiply opacity per visual; normalized group anchor/position uses composition dimensions. Root absolute timing intersects ancestor windows. Parent visibility affects visuals only, retaining audio behavior. Do not create nested offscreen group compositing or change flat stacking. Existing bounds must apply to composed geometry, including measured text and oriented media.
6. Keep all domain validation in core validation/timeline/migration/evaluation owners; headless decodes and translates, bridge registers injected typed workflows, and desktop exhaustive matches only adapt to the new non-drawing variant. No dependency edge changes are planned. Update ADR 0004 activation notes and group usage docs.
7. Govern new runtime fixtures separately as group-parent-v1, leaving other roadmap concepts fixture-only. Update ownership, headless/MCP catalogs, native types, capabilities, and parity together before accepting implementation. Stable errors and retryability remain unchanged. A new project variant requires a group-aware client; no blanket old-reader compatibility claim is made.

## Risks / Trade-offs

- Composed affine overflow: validate derived geometry before writes/allocation; prove independent matrix and pixel fixtures.
- Hidden cyclic nodes: validate the entire graph before filtering.
- Destructive group deletion: reject surviving references; require explicit ordered detachment.
- Public closed unions: capability discovery and schema 10 signal new content; old simple operations remain valid.
- Path/resource safety: no new locator fields; reuse bounded read-only path-safe measurement and existing bindings.
- Approval choices: root-wide scope, local-preserving reparenting, node-only duplication, and interval/visibility inheritance are proposed decisions requiring approval with this change.

## Migration Plan

After approval, introduce schema 10 using the existing locked migration chain, default parent to absent, and migrate all current and retained snapshots atomically. Validate the complete envelope before publication. Exercise every persistence fault-injection point, mixed old histories, future versions, invalid graphs, undo/redo and deterministic reopen. Preserve media/provenance and legacy output. Rollback requires a compatible schema-10 reader or a separately approved forward migration; never downgrade grouped files through an older binary.

## Verification Plan

Implementation detail: legacy animated descendants retain root-time keyframes. Pure geometry finalization uses position/scale extrema to validate object geometry and derive a canvas-clipped sampling envelope before allocation. The renderer consumes those bounds and evaluated keyframes; it does not expand a frame table or persist ancestor matrices. Verification evidence is in verification.md.

Every delta scenario needs automated evidence, mapped in tasks: core graph/property/lifecycle tests, canonical Rust/TypeScript payload and semantic fixtures, persistence fault injection, headless/MCP standalone/batch integration, and renderer golden/oracle checks. Run the repository-mandated checks, then openspec-verify-change, synchronize and archive, and rerun Moon with an archive-only change inventory. Approval of planning artifacts is not implementation verification or CODEOWNER contract review.

## Open Questions

No unresolved behavior is delegated to implementation. The proposed decisions above require explicit user/reviewer approval before coding. If review changes semantics, update these artifacts first.

## Approved review corrections

Preserve all nine legacy text anchors using the measured scaled styled box in static and animated local transforms. Explicit Transform2D anchors remain authoritative. Replace trajectory-sized raster allocation with a composition-clipped conservative sampling envelope, tiled deterministically in at most 4096-by-4096 regions before artifact creation. Keep finite-coordinate and per-object geometry validation before clipping; travel distance is not a complexity violation. Sample tiles in global coordinates without seams or repeated alpha. Empty intersections produce no visual work and leave audio unchanged. Shared validation rejects either transition endpoint resolving to a group, including hidden records and retained snapshots, with INVALID_ARGUMENT and no publication.

### CI verification implementation

The equivalent-selection SSIM oracle trims both decoded inputs to one frame and resets each timestamp before framesync. Limiting output frames alone is insufficient on FFmpeg 6.1.1 and can include later frames outside the ancestor interval. This corrects the verification procedure without changing rendering behavior or audiovisual tolerances. RGB test decoding uses fixed-size array chunks compatible with strict Rust 1.98 Clippy.
