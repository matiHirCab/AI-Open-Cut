# Atomic component lifecycle

Protocol 1 runtimes advertising `component_lifecycle` support this workflow:

1. `component_create` stores a reusable definition with local tracks and optional slots.
2. `component_define_slots` replaces its typed slot bindings.
3. `add_component_instance` places the template on a root overlay track with `slotValues`.
4. `component_instance_duplicate` copies an instance with independent overrides.

These are standalone MCP tools with `projectId` and `expectedRevision`, headless `edit` payloads, and operations inside `timeline_batch_edit` (headless `edit_batch`). Creation operations accept `resultAlias` in batches. References resolve only to earlier creations; local slot names and string values remain literal.

## Duplication

```json
{
  "operation": "component_instance_duplicate",
  "itemId": "@original",
  "offsetMs": 1000,
  "slotValues": {
    "heading": { "type": "text", "value": "Second card" }
  },
  "resultAlias": "copy"
}
```

`itemId` names one root component instance. `offsetMs` is required, nonnegative and a JavaScript-safe integer; the copied start and end must remain safe integers. The new item appends to the same unlocked overlay track with a fresh ID and stack order. It retains the shared component ID, parent, transforms, visibility, z-index, duration, source trim and time scale. The result reports the new ID; later batch edits can use `@copy`. Moving or reparenting uses existing explicit edits.

Omitting `slotValues` copies stored overrides. An explicit map **replaces the entire map**. An empty map clears overrides: defaults apply, absent optional slots retain base properties, and required values without defaults must still be supplied. All eight slot kinds and legal special IDs such as `__proto__`, `constructor` and `toString` retain their typed meaning. Neither the source instance nor the shared definition is modified or materialized into copied tracks.

Coordinates, half-open local timing, ordering, visibility, audio and fallback remain those of [component evaluation](component-evaluation.md). Duplicates render like equivalent explicit placements through frame/range/draft preview and export. [Slot constraints](template-slots.md), finite values, managed assets and existing graph/text limits apply before saving. Expanded occurrence and scene limits remain renderer preflight checks and fail before render artifact preparation or backend execution.

## Atomicity and compatibility

An ordered batch of 1–100 edits commits once and adds one undo step. Undo, redo and reopen preserve IDs, definitions, overrides and order. A stale revision fails with retryable `REVISION_CONFLICT`. Missing source/slot IDs use `ITEM_NOT_FOUND`, absent managed assets use `ASSET_NOT_FOUND`, locked tracks use `TRACK_LOCKED`, and invalid source types, timing, values or aliases use existing invalid-argument errors. Failed edits and batches preserve project/history bytes and revision, including failure after an earlier duplication in the same batch.

Schema 13 and protocol 1 are unchanged; no migration is introduced. Older schema-13 runtimes can read saved duplicates. Existing `duplicate_items`, component creation, slot replacement and instance-update requests retain their meanings. `contracts/component-lifecycle-v1.json` governs the additive API; existing component and slot catalogs continue to own underlying semantics.
