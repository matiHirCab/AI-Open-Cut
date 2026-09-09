## Context

Repeater expansion currently follows two paths in `editor-core`: flat shape/group copies are produced while evaluating flat project items, while component-instance copies are appended later during recursive component expansion. A group copy therefore sees flat descendants but not component occurrences that are resolved afterward. The component path also builds generated order with a textual occurrence ID, which misorders numeric sibling positions such as 10 and 2. Persistence activation from schema 16 to 17 is implemented, but schema 16 is absent from the generic fault-injection matrices. Review of the first correction also showed that group-source preflight descends only one parent level and that rich-text documents are applied after a child scope has already produced its local repeater copies. Review of the resulting fix then found that recursive preflights can inspect malformed component/group graphs before canonical cycle validation and that the evaluator deep-clones its ordinary snapshot and generated layers before all expanded vector/raster/memory budgets have been projected.

The living specs already require complete transitive group occurrences, preserved subtree order, shared `EvaluatedScene` rendering, and atomic migration. This change makes implementation and evidence conform without changing public or persisted representations.

## Goals / Non-Goals

**Goals:**

- Expand repeaters only after all ordinary occurrences in the containing scope, including recursively resolved components, are available.
- Use one copy path for shape, group, and component sources so transform, opacity, interval, identity, limits, and ordering cannot diverge by source kind.
- Preserve source-relative numeric order across mixed flat/component descendants and any sibling count.
- Count every transitive group controller/descendant exactly once per source occurrence during preflight without double-counting the ordinary flat scope.
- Preserve resolved rich-text documents and runs through local and outer repeater copies without matching generated identities heuristically.
- Reject malformed group/component graphs deterministically before any recursive preflight can overflow the call stack, including hidden and unused definitions.
- Prove all expanded visual, vector, composed-transform, raster-surface, and aggregate-memory budgets before cloning any generated layer.
- Prove schema 16-to-17 recovery at every existing publication fault boundary.

**Non-Goals:**

- No public API, contract catalog, model, schema-version, validation-rule, or renderer-interface changes.
- No time offsets, new source kinds, persisted generated items, or desktop authoring work.
- No weakening or renumbering of preflight limits, public errors, or error precedence for missing references versus invalid graphs.

## Decisions

### Assemble ordinary scope occurrences before one repeater pass

For each root or component-definition scope, exclude repeaters from flat evaluation, evaluate ordinary flat items, recursively expand ordinary component instances, and retain the resulting occurrence/order metadata as an immutable source snapshot. Then iterate the scope's repeaters and select bases from that snapshot: the exact shape occurrence, every transitive occurrence belonging to a group, or the complete resolved occurrence range of a component instance. Repeaters in a nested component are resolved inside that component before its occurrences return to the parent, so an outer component/group source naturally includes the nested resolved result.

This replaces source-kind-specific expansion timing. Merely extending the late component loop to recognize ancestor groups was rejected because mixed flat/component descendants would still be emitted by separate ordering paths and could not reliably preserve their interleaving.

### Carry source-relative numeric order into generated blocks

Each generated key starts with the containing scope prefix and the repeater's canonical track/z-index/item position, followed by a numeric one-based copy marker and the base occurrence's `InstanceOrder` relative to the containing scope. Stable item identity remains only the final tie-breaker. Generated identities continue to derive from scope, repeater ID, copy index, and base occurrence identity.

Reformatting IDs with padded decimal segments was rejected because it would encode ordering into display strings, remain fragile at future limits, and change otherwise valid identities unnecessarily.

### Apply offsets around the persisted source parent's coordinate frame

The common expansion helper resolves the persisted source item and composes `parent * O^i * parent_inverse` around each already evaluated base occurrence. It intersects the base interval/clip with the repeater interval and multiplies inherited opacity by the existing clamped copy factor. This preserves the specified origin for groups and component instances without reconstructing descendant transforms.

Applying the offset around each descendant's parent was rejected because it would deform a group rather than transform the copied subtree as a unit.

### Extend existing migration fault matrices

Schema 16 is added to both the pre-journal byte-preservation matrix and every post-commit publication/recovery phase. Assertions cover schema 17 current state, undo/redo, one complete generation, cleanup of managed transaction files, and unchanged bytes when failure precedes the commit point. No migration production code or schema version changes are expected.

A separate happy-path-only schema-16 test was rejected because it would continue to leave the interruption guarantee untested.

### Recurse through group sources only inside source-closure counting

`count_source(Group)` recursively invokes source-closure counting for each direct child. Ordinary `count_tracks` continues to count each persisted item once, so nested items are not double-counted merely because they have ancestors. Component children retain their existing memoized definition expansion and repeaters retain checked multiplication. This makes `outer -> inner -> descendants` contribute the complete source size to every generated copy before allocation.

Adding a generic group branch to `count_item` was rejected because flat track iteration already visits every persisted group and descendant and would count ordinary nested items multiple times.

### Apply rich-text documents before freezing a component scope

Each component-instance recursion resolves its effective rich-text slot documents and passes only the target-layer documents into the child evaluation scope. The child applies those documents to its directly owned ordinary evaluated text layers before component recursion completes and before the immutable repeater source snapshot is created. Local repeaters therefore clone final `text` and `rich_runs`, and outer group/component repeaters clone those already-correct occurrences. The existing post-recursion direct-depth rewrite is removed.

Broadly matching target IDs anywhere in generated `InstanceOrder` was rejected because nested component scopes may legally reuse local IDs and would receive another scope's slot override.

### Preserve archived chronology with a dated completion note

The prior proposal's approval section will state both the initial pre-implementation approval and the later completed contract review, referencing the detailed evidence in its tasks file. The correction does not rewrite the dates or claim the review happened earlier.

### Validate graphs before recursive preflight

Evaluation first runs the canonical project validation needed to establish item indexing, parent/reference invariants, and complete group/component acyclicity. Only after those invariants pass may SVG, component-occurrence, repeater, or audio-closure preflights recurse. `validate_scope` is organized into safe passes: build indexes and validate intrinsic invariants, validate all parent links and cycles, then resolve repeater sources and audio closures. Every recursive group/component walker additionally maintains an active-node set and returns `INVALID_ARGUMENT` on re-entry, so a later call-order regression still cannot overflow the process stack. Missing references continue to return `ITEM_NOT_FOUND` with their existing precedence.

Validating only the currently used root closure was rejected because hidden or unused component definitions are part of the canonical project and must be rejected deterministically too.

### Project complete scope work before materialization

The ordinary portion of a scope is frozen as bounded indices plus source/order metadata rather than a deep-cloned `Vec` of layers. Repeater expansion then has two passes. The projection pass resolves every same-scope repeater exclusively against that ordinary range and uses checked arithmetic to accumulate effective intervals, final parent-conjugated matrices, visual layers, vector segments, raster surfaces, and aggregate bytes for all copies. Only after the full scope projection succeeds does the materialization pass clone occurrences and publish them in canonical order. A failure therefore leaves the result without any generated layers.

Checking each repeater immediately before its own append was rejected because earlier repeaters could already have allocated substantial generated work before a later one proves the scope invalid.

### Share occurrence measurement with scene refinement

The segment, raster-bound, surface, and byte calculation currently embedded in final scene refinement is extracted into a pure, non-publishing occurrence measurement used by both projection and refinement. Projection measures each source occurrence with its final `parent * O^i * parent_inverse` transform and intersected interval; it does not accept a finite `O^i` when the composed matrix or derived bounds are non-finite. Exact inclusive limits remain valid, and the first one-over result returns `INVALID_ARGUMENT` before materialization.

Maintaining a second approximate estimator was rejected because drift from the final renderer measurement would make boundary acceptance nondeterministic.

## Risks / Trade-offs

- [Risk] Moving all scope repeaters to a common late pass could accidentally include generated occurrences from another repeater. → Build every same-scope source set from the immutable ordinary snapshot; multiple repeaters remain independent.
- [Risk] Nested components and local repeaters can multiply occurrence work. → Retain the existing preflight and layer/resource limits and verify exact-boundary tests still pass.
- [Risk] New order keys could alter valid simple cases. → Assert existing stable IDs/order plus mixed, greater-than-nine, and equal-z cases; keep identity formatting unchanged where already emitted.
- [Risk] Native golden tests can silently skip without tools. → Run the focused and full native gates with required-tool mode and the pinned FFmpeg/font configuration recorded in evidence.
- [Risk] Recursive source counting could double-count ordinary nested items. → Recurse only from `count_source(Group)` and retain flat `count_tracks` semantics; prove the exact 65,536/65,537 boundary.
- [Risk] Rich-text propagation could affect a nested scope reusing the same target ID. → Apply overrides only to raw direct layers inside the owning child scope before identities and generated paths are assembled.
- [Risk] Reordering validation could change which error is observed first. → Preserve the canonical missing-reference and invalid-argument precedence in focused tests while moving only recursion-dependent work after graph validation.
- [Risk] The projection pass and final refinement could disagree at numeric boundaries. → Use one pure measurement routine and checked accumulators in both paths; assert exact and one-over limits.
- [Risk] Lightweight indices can become stale while layers are appended. → Store an immutable ordinary range and source-relative metadata, and resolve every same-scope repeater before publishing any generated layer.

## Migration Plan

No data migration or deployment sequencing is required. Implement behind the unchanged schema-17 representation, run compatibility and golden gates, and roll back by reverting the evaluator/test change if regressions appear; persisted projects remain readable by the existing schema-17 implementation.

## Open Questions

None. Public compatibility, ordering, source selection, fault coverage, and approval requirements are fixed by the living specifications and this design.
