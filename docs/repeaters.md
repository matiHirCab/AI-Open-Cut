# Lazy repeaters

Schema 17 adds compact `repeater` timeline items and protocol-1 operations `add_repeater` and `timeline_add_repeater`. Editing support is advertised as `repeater_items`; complete local rendering is advertised as `repeater_rendering`.

A repeater occupies an overlay track and contains `source`, `copies`, `transformOffset`, and `opacityOffset`. The source scope is `root` for root items or `component:<componentId>` inside a definition, and its ID must name a shape, group, or component instance in that scope. Group and component sources must have visual-only transitive content. Copy counts are integers from 1 through 256 and count additional copies: the source continues to render normally.

`transformOffset` contains a pixel or normalized position and finite `scaleX`, `scaleY`, `rotationDeg`, `skewXDeg`, and `skewYDeg`. For one-based copy index `i`, core applies the offset matrix power `O^i` in source-parent coordinates. Opacity is `sourceOpacity * clamp(1 + i * opacityOffset, 0, 1)`. Copies use the source clock intersected with the repeater's half-open interval; this milestone does not apply time offsets.

Generated occurrences are lazy and never appear as persisted timeline items. They form a deterministic block at the repeater's track/z-index/stack-order position, with ascending copy index and preserved source-subtree order. Core preflights the 256-copy bound, the existing 65,536 expanded-occurrence bound, 4,096 visual-layer bound, finite matrices, and existing vector/surface/memory limits before renderer work.

`add_repeater` participates in `timeline_batch_edit`; earlier aliases resolve in `trackId` and `repeater.source.id`, and the repeater's `resultAlias` can be used by later operations. Repeater descriptors are replaced as a whole through `update_item.repeater`. Timing, move, trim, split, duplicate, delete, visibility, z-index, and reorder operations are supported. Parenting, common transforms, keyframes, audio, and transition endpoints are rejected.

Batch descriptor replacement resolves `itemId` first and `repeater.source.id` second; `scope` stays literal. Missing/forward aliases retain `VALIDATION_FAILED` and full rollback. Drafts continue using literal IDs, including IDs returned by earlier committed batches; they do not declare batch aliases.

Validation is independent of publication: hidden tracks/items, clipped instances and unused definitions consume complete expanded budgets. The root and each definition have independent budgets; unrelated definitions are not added to the root total. Effective overrides, local repeaters and final parent-conjugated matrices are checked before any generated layer is cloned. Only visible, nonempty occurrences are published with their source clocks, bindings and deterministic order.

The inclusive 4,096 transition-fact budget counts each endpoint role on every ordinary and generated occurrence, including nested copies and retained hidden transitions. Self endpoints contribute both roles. Hidden transitions count for validation but are not published. Any excess fails with `INVALID_ARGUMENT` before generated-layer materialization or backend work.

Visual-only source validation resolves asset defaults and overrides before classifying group/component descendants, including local repeaters. An effective asset with audio is rejected even when hidden, clipped or muted; repeaters never copy audio. Standalone definitions use defaults and actual instances use overrides, with independent results for distinct values. Invalid edits and drafts preserve revision/history and invalid reopened snapshots are rejected without rewriting files.

Opening schemas 1–16 migrates current state and retained undo/redo history atomically to 17. Repeater records under earlier declared versions and unknown future versions fail closed without rewrite. Older binaries cannot decode schema-17 repeater projects; no downgrade exists. Canonical fields, limits, examples, and failures live in `contracts/repeaters-v1.json`.
